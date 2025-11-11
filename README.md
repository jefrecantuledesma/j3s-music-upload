# J3S Music Upload Service

A web-based music upload service for Navidrome with file upload and YouTube download capabilities. Built with Rust, Axum, and MariaDB.

## Features

- **User Authentication**: Secure JWT-based authentication with password hashing
- **File Upload**: Upload audio files (MP3, FLAC, OGG, OPUS, M4A, WAV, AAC)
- **YouTube Download**: Download audio from YouTube videos using yt-dlp
- **Ferric Integration**: Automatic audio processing, conversion, and organization
- **Admin Panel**: User management and configuration editing
- **Upload History**: Track all uploads with status and error logging
- **Docker Support**: Easy deployment with Docker and docker-compose

## Quick Start

### Prerequisites

- Docker and docker-compose
- Ferric binary (place in `/usr/local/bin/ferric` or specify path in config)
- Navidrome music directory at `/srv/navidrome/music`

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd j3s_music_upload
```

2. Copy the example configuration:
```bash
cp config.toml.example config.toml
```

3. Edit `config.toml` with your settings:
```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "mysql://music_upload:change_this_password@mariadb:3306/music_upload"
max_connections = 5

[paths]
music_dir = "/srv/navidrome/music"
temp_dir = "/srv/navidrome/music/tmp"
ferric_path = "/usr/local/bin/ferric"

[security]
# Generate with: openssl rand -base64 32
jwt_secret = "your-secret-key-here"
session_timeout_hours = 24

[upload]
max_file_size_mb = 500
allowed_extensions = ["mp3", "flac", "ogg", "opus", "m4a", "wav", "aac"]

[youtube]
enabled = true
ytdlp_path = "yt-dlp"
audio_format = "best"
```

4. Update database credentials in `docker-compose.yml`:
```yaml
environment:
  - MYSQL_ROOT_PASSWORD=change_this_root_password
  - MYSQL_DATABASE=music_upload
  - MYSQL_USER=music_upload
  - MYSQL_PASSWORD=change_this_password
```

5. Start the services:
```bash
docker-compose up -d
```

6. Create the first admin user (after the database is initialized):
```bash
docker exec -it music_upload_service /app/j3s_music_upload create-admin
```

Or manually using SQL:
```bash
docker exec -it music_upload_db mysql -u music_upload -p music_upload
```

Then run:
```sql
-- Generate a password hash using argon2
-- You can use an online tool or create a user via the admin panel once you have one admin
INSERT INTO users (id, username, password_hash, is_admin)
VALUES (UUID(), 'admin', '$argon2id$...', true);
```

### Creating the First Admin User

To bootstrap the first admin user, you can use a simple SQL script after the database is initialized:

```bash
# Connect to the database
docker exec -it music_upload_db mysql -u music_upload -p music_upload

# Insert admin user (you'll need to hash the password first)
# For development, you can temporarily modify the code to print a hash
# or use an online argon2 hash generator
```

For easier setup, here's a Rust snippet to generate a password hash:

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

let password = "your_admin_password";
let salt = SaltString::generate(&mut OsRng);
let hash = Argon2::default().hash_password(password.as_bytes(), &salt).unwrap();
println!("Password hash: {}", hash);
```

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

#### Protected
- `POST /api/upload` - Upload audio files
- `POST /api/youtube` - Download from YouTube
- `GET /api/admin/users` - List users
- `POST /api/admin/users` - Create user
- `DELETE /api/admin/users/:id` - Delete user
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

## License

[Your License Here]

## Contributing

[Your Contributing Guidelines Here]
