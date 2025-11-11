# J3S Music Upload Service - Project Summary

## What Was Built

A complete, production-ready web application for uploading music to your Navidrome server. Built entirely in Rust with modern web technologies.

## Key Features

### 1. User Authentication
- JWT-based authentication with secure password hashing (Argon2)
- Session management with configurable timeout
- Admin and regular user roles

### 2. File Upload
- Drag-and-drop or browse to upload audio files
- Support for multiple formats: MP3, FLAC, OGG, OPUS, M4A, WAV, AAC
- Configurable file size limits
- Real-time upload progress feedback

### 3. YouTube Integration
- Download audio from YouTube videos
- Uses yt-dlp for reliable extraction
- Includes legal disclaimer for users
- Can be enabled/disabled via configuration

### 4. Ferric Integration
- Automatic audio processing pipeline
- FLAC to OPUS conversion
- Metadata organization and tagging
- Filename normalization
- Automatic merge into Navidrome library

### 5. Admin Panel
- User management (create, list, delete users)
- Runtime configuration editing
- Upload history and logs
- System monitoring

### 6. Modern UI
- Clean, responsive design
- Mobile-friendly interface
- Real-time status updates
- Error handling with user-friendly messages

## Project Structure

```
j3s_music_upload/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── db.rs                # Database operations
│   ├── auth.rs              # Authentication & JWT
│   ├── models.rs            # Data models
│   ├── templates.rs         # HTML template definitions
│   └── handlers/
│       ├── mod.rs
│       ├── auth_handlers.rs # Login/logout
│       ├── upload.rs        # File upload logic
│       ├── youtube.rs       # YouTube download logic
│       └── admin.rs         # Admin panel handlers
├── templates/
│   ├── base.html            # Base template
│   ├── login.html           # Login page
│   ├── upload.html          # Upload interface
│   ├── admin.html           # Admin panel
│   └── logs.html            # Upload history
├── migrations/
│   ├── 01_create_users_table.sql
│   ├── 02_create_config_table.sql
│   └── 03_create_upload_logs_table.sql
├── examples/
│   └── hash_password.rs     # Password hashing utility
├── scripts/
│   ├── init_admin.sh        # Admin initialization script
│   └── create_admin.sh      # Helper script
├── Dockerfile               # Container image definition
├── docker-compose.yml       # Docker orchestration
├── config.toml.example      # Example configuration
├── README.md                # User documentation
├── SETUP.md                 # Setup instructions
└── Cargo.toml               # Rust dependencies

## Technology Stack

### Backend
- **Rust 2021 Edition** - Memory-safe, fast systems programming
- **Axum 0.7** - Modern async web framework
- **SQLx** - Compile-time checked SQL queries
- **Tokio** - Async runtime
- **Argon2** - Password hashing
- **JWT** - Token-based authentication

### Frontend
- **HTML5** - Modern semantic markup
- **CSS3** - Custom styling with gradients and animations
- **JavaScript** - Vanilla JS for API interactions
- **Fetch API** - Asynchronous requests

### Database
- **MariaDB 11** - Reliable relational database
- **SQLx Migrations** - Version-controlled schema

### DevOps
- **Docker** - Containerization
- **Docker Compose** - Multi-container orchestration
- **yt-dlp** - YouTube video/audio downloader

## Database Schema

### users
- `id` - UUID primary key
- `username` - Unique username
- `password_hash` - Argon2 hashed password
- `is_admin` - Boolean admin flag
- `created_at` - Timestamp
- `updated_at` - Timestamp

### config
- `key` - Configuration key (primary)
- `value` - Configuration value
- `updated_at` - Timestamp

### upload_logs
- `id` - Auto-increment primary key
- `user_id` - Foreign key to users
- `upload_type` - 'file' or 'youtube'
- `source` - File name or URL
- `status` - pending/processing/completed/failed
- `file_count` - Number of files processed
- `error_message` - Error details if failed
- `created_at` - Timestamp
- `completed_at` - Timestamp

## API Endpoints

### Public Endpoints
- `GET /` - Login page
- `GET /upload` - Upload interface
- `GET /admin` - Admin panel
- `GET /logs` - Upload history
- `POST /api/login` - User authentication

### Protected Endpoints (Require JWT)
- `POST /api/upload` - Upload audio files
- `POST /api/youtube` - Download from YouTube
- `POST /api/logout` - Logout
- `GET /api/admin/users` - List users
- `POST /api/admin/users` - Create user
- `DELETE /api/admin/users/:id` - Delete user
- `GET /api/admin/config` - List configuration
- `POST /api/admin/config` - Update configuration
- `GET /api/admin/config/:key` - Get specific config
- `GET /api/admin/logs` - Get upload logs

## Security Features

1. **Password Security**
   - Argon2 hashing with salts
   - Minimum 8 character requirement
   - No plaintext storage

2. **Authentication**
   - JWT tokens with expiration
   - Secure session management
   - Protected API endpoints

3. **Authorization**
   - Role-based access control
   - Admin-only routes
   - User isolation (users only see their own logs)

4. **Input Validation**
   - File type checking
   - File size limits
   - SQL injection protection (prepared statements)
   - XSS prevention (template escaping)

5. **Network Security**
   - HTTPS ready (reverse proxy)
   - CORS configuration
   - Rate limiting (can be added via reverse proxy)

## Configuration Options

All configurable via `config.toml`:

- Server host and port
- Database connection
- File paths (music dir, temp dir, Ferric path)
- JWT secret and session timeout
- Upload limits and allowed file types
- YouTube download settings

## Quick Start

```bash
# 1. Copy configuration
cp config.toml.example config.toml

# 2. Generate JWT secret
openssl rand -base64 32

# 3. Edit config.toml with your settings

# 4. Start services
docker-compose up -d

# 5. Initialize admin user
./scripts/init_admin.sh

# 6. Access at http://localhost:8080
```

## Development

### Building
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Running Locally
```bash
# Start database
docker-compose up -d mariadb

# Run application
cargo run
```

## Production Deployment

1. Set up reverse proxy (nginx/caddy)
2. Configure SSL certificate
3. Set strong passwords
4. Configure firewall
5. Set up backups
6. Monitor logs

See SETUP.md for detailed instructions.

## Future Enhancements

Potential features to add:

- [ ] Bulk upload with queue management
- [ ] Playlist import/export
- [ ] Audio preview before upload
- [ ] Duplicate detection
- [ ] Tag editing interface
- [ ] User upload quotas
- [ ] Rate limiting
- [ ] Email notifications
- [ ] API key authentication
- [ ] Mobile app support
- [ ] Spotify/Apple Music integration
- [ ] Collaborative playlists
- [ ] Upload scheduling
- [ ] Bandwidth throttling

## License

[Your License Here]

## Credits

Built with:
- Rust and the amazing Rust ecosystem
- Axum web framework
- SQLx for database access
- Askama for templating
- yt-dlp for YouTube downloads
- Ferric for audio processing

## Support

For issues, questions, or contributions:
- Check README.md
- Review SETUP.md
- Open an issue on GitHub
- Contact the maintainer

---

**Note**: This application is designed for personal use and requires proper licensing for any music content uploaded or downloaded. Always respect copyright laws and only upload/download content you have rights to use.
