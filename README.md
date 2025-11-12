# J3S Music Upload Service

A web-based music upload service for Navidrome with file upload and YouTube download capabilities. Built with Rust, Axum, and MariaDB.

## ✨ Features

- **User Authentication**: Secure JWT-based authentication with Argon2 password hashing
- **Default Admin User**: Automatically created on first startup (username: `admin`, password: `admin`)
- **Password Management**: Self-service password changes and admin user management
- **File Upload**: Upload audio files (MP3, FLAC, OGG, OPUS, M4A, WAV, AAC)
- **YouTube Download**: Download audio from YouTube videos using yt-dlp
- **Ferric Integration**: Automatic audio processing, conversion, and organization
- **Admin Panel**: User management, password changes, and configuration editing
- **Upload History**: Track all uploads with status and error logging
- **Docker Support**: Easy deployment with Docker and docker-compose
- **Interactive Setup**: Automated setup script for both Docker and local deployments

## 🚀 Quick Start (2 Minutes!)

**For the absolute quickest setup, see [QUICKSTART.md](QUICKSTART.md)**

### Easy Installation (Recommended)

1. **Run the interactive setup script:**
   ```bash
   ./scripts/setup.sh
   ```
   - Choose Docker or Local mode
   - Answer the prompts (press Enter for defaults)
   - Script auto-generates secure passwords and creates all config files

2. **Start the service:**
   ```bash
   docker-compose up -d  # For Docker mode
   # OR
   cargo run --release   # For local mode
   ```

3. **Login and change the default password:**
   - Open `http://localhost:8080`
   - Login with username `admin` and password `admin`
   - **Change the password immediately!**

That's it! The application automatically creates a default admin user on first startup.

### Manual Installation (Advanced)

<details>
<summary>Click to expand manual installation instructions</summary>

#### Prerequisites

- Docker and docker-compose (for Docker mode)
- Rust (latest stable) and MariaDB (for local mode)
- Ferric binary (optional, for audio processing)

#### Steps

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd j3s_music_upload
   ```

2. Copy the example configuration:
   ```bash
   cp config.toml.example config.toml
   ```

3. Generate a JWT secret and update `config.toml`:
   ```bash
   openssl rand -base64 32
   # Paste the output into config.toml's jwt_secret field
   ```

4. Update database credentials in both `config.toml` and `docker-compose.yml`

5. Start the services:
   ```bash
   docker-compose up -d
   ```

6. Access the web interface at `http://localhost:8080`
   - Login with default credentials: `admin` / `admin`
   - **Change the password immediately!**

</details>

### Default Admin User

On first startup, if no users exist in the database, the application automatically creates:
- **Username:** `admin`
- **Password:** `admin`

**🔐 CRITICAL: Change this password immediately after first login!**

## Usage

1. Access the web interface at `http://localhost:8080` or `http://music-upload.jcledesma.xyz`
2. Login with your admin credentials
3. Create additional users via the Admin panel
4. Upload music files or download from YouTube
5. Files are automatically processed by Ferric and merged into Navidrome

## Architecture

### Workflow

1. **Upload/Download**: User uploads files or provides YouTube URL
2. **Temporary Storage**: Files saved to `/srv/navidrome/music/tmp`
3. **Ferric Processing**:
   - Converts FLAC to OPUS
   - Organizes files by artist/album
   - Normalizes filenames (lowercase, remove special chars)
4. **Merge**: Processed files moved to `/srv/navidrome/music`
5. **Cleanup**: Temporary files removed

### Components

- **Web Server**: Axum-based async web server
- **Database**: MariaDB for user and config storage
- **Authentication**: JWT tokens with argon2 password hashing
- **File Processing**: Integration with Ferric CLI tool
- **YouTube**: yt-dlp for audio extraction

### API Endpoints

#### Public
- `POST /api/login` - User authentication

#### Protected (Require JWT)
- `POST /api/upload` - Upload audio files
- `POST /api/youtube` - Download from YouTube
- `POST /api/user/change-password` - Change own password
- `POST /api/logout` - Logout (client-side token removal)

#### Admin Only
- `GET /api/admin/users` - List all users
- `POST /api/admin/users` - Create new user
- `DELETE /api/admin/users/:id` - Delete user
- `POST /api/admin/users/:id/password` - Change any user's password
- `GET /api/admin/config` - List config
- `POST /api/admin/config` - Update config
- `GET /api/admin/logs` - Get upload logs

## Configuration

### Runtime Configuration

Some settings can be modified at runtime through the admin panel. These are stored in the database `config` table and override file-based configuration.

### File-based Configuration

See `config.toml.example` for all available options.

## Security Considerations

- **JWT Tokens**: Secure, stateless authentication
- **Password Hashing**: Argon2 with salt
- **File Validation**: Extension and size checks
- **SQL Injection**: Protected by sqlx prepared statements
- **YouTube Downloads**: User warned to only download permitted content

## Development

### Building

```bash
cargo build --release
```

### Running locally

```bash
# Start MariaDB
docker-compose up -d mariadb

# Update config.toml with localhost database URL
# Then run
cargo run
```

### Database Migrations

Migrations are automatically run on startup using sqlx migrate. Manual migration:

```bash
sqlx migrate run
```

## Troubleshooting

### Database Connection Errors
- Check database credentials in `config.toml`
- Ensure MariaDB container is running
- Verify network connectivity

### Upload Failures
- Check file size limits in config
- Verify temp directory permissions
- Check Ferric path and permissions

### YouTube Download Issues
- Ensure yt-dlp is installed in container
- Check YouTube URL format
- Verify network access from container
- Keep `youtube.player_client = "web"` (or a working client) in `config.toml` to bypass recent YouTube "Precondition check failed" responses

## License

[Your License Here]

## Contributing

[Your Contributing Guidelines Here]
