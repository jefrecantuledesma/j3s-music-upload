# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

J3S Music Upload Service is a Rust web application for uploading music to Navidrome. It provides file upload, YouTube download capabilities, and integrates with Ferric for audio processing.

**Tech Stack:**
- Rust (latest stable via Docker)
- Axum 0.7 (web framework)
- SQLx with SQLite (database)
- JWT + Argon2 (authentication)
- Askama (templates)
- Docker + docker-compose (deployment)

## Development Commands

### Quick Setup

```bash
# Interactive setup script (generates config, creates directories)
./scripts/setup.sh

# Start the service
docker-compose up -d
```

### Building & Running

```bash
# Build the project
cargo build --release

# Run locally (SQLite database auto-created)
cargo run

# Run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f music-upload

# Rebuild and restart
docker-compose build && docker-compose up -d
```

### Database

```bash
# Run migrations (automatically runs on startup)
sqlx migrate run

# Create a new migration
sqlx migrate add <migration_name>

# Access SQLite database directly
sqlite3 ./data/music_upload.db
```

### Testing & Utilities

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Generate password hash for admin user (optional - auto admin created on first run)
cargo run --example hash_password "your_password"

# Check code (fast check without full build)
cargo check
```

### Docker Operations

```bash
# Stop all services
docker-compose down

# View container status
docker-compose ps

# Execute commands in container
docker-compose exec music-upload /app/j3s_music_upload

# Access container shell
docker-compose exec music-upload /bin/bash
```

## Architecture

### Application Structure

```
src/
├── main.rs              # Entry point, router setup, AppState
├── config.rs            # Config loading from config.toml
├── db.rs                # Database operations layer
├── auth.rs              # JWT middleware and token management
├── models.rs            # Data models and serialization
├── templates.rs         # Askama template definitions
└── handlers/
    ├── auth_handlers.rs # Login/logout endpoints
    ├── upload.rs        # File upload logic + Ferric integration
    ├── youtube.rs       # YouTube download via yt-dlp
    └── admin.rs         # User & config management
```

### Key Architectural Patterns

**AppState**: Shared application state containing `Database`, `Config`, and `AuthState`. Passed to handlers via `State<Arc<AppState>>`.

**Authentication Flow**:
1. User credentials → `POST /api/login` in `auth_handlers.rs`
2. Password verified via Argon2 in `db.rs:verify_password()`
3. JWT token created in `auth.rs:create_token()`
4. Token validated by `auth_middleware()` for protected routes
5. `AuthUser` extension added to request for handlers

**Upload Flow**:
1. Files saved to `temp_dir` (from config)
2. `process_temp_dir()` calls Ferric CLI tool
3. Ferric processes audio (FLAC→OPUS conversion, metadata tagging)
4. Files moved to `music_dir` and organized
5. Temp directory cleaned up

**Database Layer**:
- All DB operations in `db.rs` use sqlx prepared statements (SQL injection protection)
- SQLite database with auto-creation via `create_if_missing(true)`
- Migrations in `migrations/` run automatically on startup via `sqlx::migrate!()`
- Three tables: `users`, `config` (runtime settings), `upload_logs`

**Default Admin Creation**:
- On first startup, if no users exist, a default admin user is automatically created
- Credentials: username `admin`, password `admin`
- User is warned to change the password immediately (logged at WARN level)

### Configuration System

Two-tier configuration:
1. **File-based** (`config.toml`): Server settings, paths, security secrets, upload limits
2. **Database-stored** (`config` table): Runtime-editable settings accessible via admin panel

Config loaded in `main.rs` via `Config::load()` which reads `config.toml` (or `CONFIG_PATH` env var).

### Route Protection

Routes split into two groups in `main.rs`:
- **Public**: Login page (`/`, `/api/login`) - no auth required
- **Protected**: All other routes wrapped with `auth_middleware` layer (requires valid JWT in `Authorization: Bearer <token>` header)

Admin-only operations (user management, config editing) check `AuthUser.is_admin` in handlers.

### Security Features

**File Upload Security** (`handlers/upload.rs`):
- Sanitized filenames to prevent path traversal attacks
- Rejects files with `..`, `/`, or `\` in filenames
- Extension whitelist validation
- File size limit enforcement

**YouTube Download Security** (`handlers/youtube.rs`):
- Strict URL validation (only HTTPS YouTube URLs)
- Blocks command injection characters (`;`, `|`, `` ` ``, `$`, `&&`, `||`)
- URL length limit (200 chars)
- Uses args array (not shell strings) for yt-dlp execution

## Working with This Codebase

### Adding New API Endpoints

1. Define handler in appropriate `handlers/*.rs` file
2. Add route in `main.rs` to `protected_routes` or `public_routes` Router
3. Use `State<Arc<AppState>>` to access database/config
4. Use `Extension<AuthUser>` for authenticated user info

### Database Changes

1. Create migration: `sqlx migrate add <name>`
2. Write SQL in `migrations/<timestamp>_<name>.sql`
3. Add corresponding model in `models.rs` if needed
4. Add database methods in `db.rs`
5. Migrations run automatically on next startup

### Authentication Notes

- JWT tokens expire based on `session_timeout_hours` config
- Password hashing uses Argon2 with random salts (via `hash_password()` in `db.rs`)
- Default admin user created automatically on first startup (username: `admin`, password: `admin`)
- Additional admin users can be created via admin panel or by setting `is_admin = true` in database
- Password verification in `db.rs:verify_password()` uses constant-time comparison via Argon2

### Template Changes

- Templates in `templates/` directory
- Template structs defined in `src/templates.rs` using Askama
- Base template: `templates/base.html`
- Templates automatically rendered by Axum when returned from handlers

### Ferric Integration

The upload and YouTube handlers call Ferric for audio processing:
1. `process_with_ferric()` in `handlers/upload.rs` runs: `ferric --input-dir <temp_dir> --output-dir <music_dir>`
2. `process_temp_dir()` in `handlers/youtube.rs` runs the same command
3. Ferric processes audio files (FLAC→OPUS conversion, metadata tagging, organization)
4. Temp directory is cleaned up after successful processing
5. Upload logs track success/failure with error messages

Ferric path configured in `config.toml` under `paths.ferric_path`.

### Local Development Setup

1. Use setup script: `./scripts/setup.sh` (recommended) OR manual setup:
2. Copy `config.toml.example` to `config.toml`
3. Generate JWT secret: `openssl rand -base64 32` and update in config
4. Update paths in config (`music_dir`, `temp_dir`) as needed
5. Run app: `cargo run`
6. SQLite database auto-created at `./data/music_upload.db`
7. Default admin user auto-created on first run (admin/admin)

### YouTube Download Configuration

The `youtube` section in `config.toml` has several important settings:
- `player_client`: Set to `"web"` to avoid "Precondition check failed" errors (YouTube API changes)
- `extra_args`: Array of additional yt-dlp arguments (e.g., `["--throttled-rate=100K"]`)
- Arguments are tested in `handlers/youtube.rs` with unit tests

### Common Issues

**"Failed to connect to database"**: SQLite database should auto-create. Check write permissions on `./data/` directory.

**"Ferric command not found"**: Ferric must be installed separately and path set in config. The Dockerfile doesn't include Ferric by default.

**"Upload failed"**: Check `temp_dir` and `music_dir` exist and have correct permissions. Check Ferric is working: `ferric --version`.

**"YouTube download fails"**: Update `player_client` in config to `"web"` or another working client. Keep yt-dlp updated.

## Important Files

- `config.toml`: Main configuration (not in git, copy from `.example`)
- `config.toml.example`: Example configuration with all available options
- `docker-compose.yml`: Docker deployment configuration (uses external `web` network for reverse proxy)
- `migrations/`: Database schema versions (SQLite)
- `Cargo.toml`: Rust dependencies
- `Dockerfile`: Multi-stage build (Rust builder + Debian slim runtime with yt-dlp)
- `scripts/setup.sh`: Interactive setup script for generating config and initializing the service
