# Quick Start Guide

Get J3S Music Upload Service running in under 5 minutes!

## 🚀 Super Quick Setup (Recommended)

### Option 1: Docker Setup (Easiest - 2 Minutes!)

1. **Run the interactive setup script:**
   ```bash
   ./scripts/setup.sh
   ```
   - Choose option `1` for Docker
   - Answer the prompts (or press Enter to use defaults)
   - The script will auto-generate secure passwords and create all config files

2. **Start the services:**
   ```bash
   docker-compose up -d
   ```

3. **Access the web interface:**
   - Open your browser to `http://localhost:8080`
   - Login with default credentials:
     - **Username:** `admin`
     - **Password:** `admin`

4. **🔐 IMPORTANT: Change the default password immediately!**
   - The application creates a default admin user on first startup
   - Change your password using the API or web interface

That's it! You're ready to upload music.

---

### Option 2: Local Development Setup

1. **Prerequisites:**
   - Rust (latest stable)
   - MariaDB/MySQL running locally
   - Ferric (optional, for audio processing)

2. **Run the interactive setup script:**
   ```bash
   ./scripts/setup.sh
   ```
   - Choose option `2` for Local development
   - Enter your database credentials
   - The script will create your config file

3. **Create the database (if needed):**
   ```bash
   # The setup script will show you the SQL commands
   mysql -u root -p
   ```
   ```sql
   CREATE DATABASE music_upload;
   CREATE USER 'music_upload'@'localhost' IDENTIFIED BY 'your_password';
   GRANT ALL PRIVILEGES ON music_upload.* TO 'music_upload'@'localhost';
   FLUSH PRIVILEGES;
   ```

4. **Build and run:**
   ```bash
   cargo build --release
   cargo run --release
   ```

5. **Access the web interface:**
   - Open your browser to `http://localhost:8080`
   - Login with default credentials:
     - **Username:** `admin`
     - **Password:** `admin`

6. **🔐 IMPORTANT: Change the default password immediately!**

---

## 🔐 First Login & Security

### Default Credentials
- **Username:** `admin`
- **Password:** `admin`

The application automatically creates this default admin user on first startup if no users exist.

### Change Password (API)

**For yourself:**
```bash
curl -X POST http://localhost:8080/api/user/change-password \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "old_password": "admin",
    "new_password": "your_new_secure_password"
  }'
```

**As admin (change another user's password):**
```bash
curl -X POST http://localhost:8080/api/admin/users/USER_ID/password \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "new_password": "new_secure_password"
  }'
```

---

## 👥 User Management

### Create a New User (API)

```bash
curl -X POST http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "password": "secure_password",
    "is_admin": false
  }'
```

### List Users

```bash
curl -X GET http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Delete User

```bash
curl -X DELETE http://localhost:8080/api/admin/users/USER_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 📤 Uploading Music

### File Upload

1. Navigate to the Upload page
2. Select audio files (MP3, FLAC, OGG, OPUS, M4A, WAV, AAC)
3. Click Upload
4. Ferric will process and organize the files automatically

### YouTube Download

1. Navigate to the Upload page
2. Enter a YouTube URL
3. The service will download the audio and process it

---

## 🛠️ Useful Commands

### Docker

```bash
# View logs
docker-compose logs -f music-upload

# Restart services
docker-compose restart

# Stop services
docker-compose down

# Rebuild and restart
docker-compose build && docker-compose up -d

# Access database
docker exec -it music_upload_db mysql -u music_upload -p music_upload
```

### Local Development

```bash
# Build
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run

# Run migrations manually
sqlx migrate run

# Create migration
sqlx migrate add migration_name
```

---

## 📋 Checklist

**Docker Setup:**
- [ ] Ran `./scripts/setup.sh` and chose Docker mode
- [ ] Started services with `docker-compose up -d`
- [ ] Accessed web interface at http://localhost:8080
- [ ] Logged in with admin/admin
- [ ] Changed default password
- [ ] Created additional users (optional)

**Local Setup:**
- [ ] Ran `./scripts/setup.sh` and chose Local mode
- [ ] Created database with provided SQL commands
- [ ] Built project with `cargo build --release`
- [ ] Started service with `cargo run --release`
- [ ] Accessed web interface at http://localhost:8080
- [ ] Logged in with admin/admin
- [ ] Changed default password

## 🔧 Troubleshooting

### "Failed to connect to database"
- **Docker:** Ensure MariaDB is running with `docker-compose ps`
- **Local:** Check if MariaDB is running with `systemctl status mariadb`
- Verify credentials in `config.toml` match your database
- For Docker: ensure the service is healthy (wait 30 seconds after first start)

### "Ferric command not found"
- Install Ferric from the ferric repository
- Update `ferric_path` in `config.toml` to the correct location
- Or temporarily disable Ferric processing for testing

### "Permission denied" on directories
- Ensure the music and temp directories are writable
- For Docker: check volume mounts in `docker-compose.yml`
- The app will auto-create directories if they don't exist

### Upload fails
- Check file size limits in `config.toml` (default 500MB)
- Ensure file extension is in `allowed_extensions` list
- Check disk space in music and temp directories
- Review logs: `docker-compose logs music-upload` or check console output

### "YouTube download doesn't work"
- Verify yt-dlp is installed: `yt-dlp --version`
- Check if YouTube URL is valid
- Review logs for specific errors
- Ensure `youtube.enabled = true` in config.toml

## 📁 Directory Structure

```
j3s_music_upload/
├── src/                    # Rust source code
│   ├── handlers/          # API endpoint handlers
│   ├── main.rs            # Application entry point
│   ├── db.rs              # Database layer
│   ├── auth.rs            # Authentication
│   └── ...
├── migrations/            # Database migrations (auto-run on startup)
├── templates/             # Askama HTML templates
├── scripts/               # Utility scripts
│   └── setup.sh          # Interactive setup script
├── config.toml            # Configuration (created by setup script)
├── docker-compose.yml     # Docker services (created by setup script)
├── Dockerfile             # Docker build definition
└── Cargo.toml            # Rust dependencies
```

---

## 🔗 API Endpoints Reference

### Authentication
- `POST /api/login` - Login and get JWT token
- `POST /api/logout` - Logout (client-side token removal)

### User Management
- `GET /api/admin/users` - List all users (admin)
- `POST /api/admin/users` - Create new user (admin)
- `DELETE /api/admin/users/:id` - Delete user (admin)
- `POST /api/admin/users/:id/password` - Change user password (admin)
- `POST /api/user/change-password` - Change own password

### Upload
- `POST /api/upload` - Upload audio files
- `POST /api/youtube` - Download from YouTube URL

### Logs & Config
- `GET /api/admin/logs` - View upload logs
- `GET /api/admin/config` - List config values
- `POST /api/admin/config` - Update config value
- `GET /api/admin/config/:key` - Get specific config value

---

## 🎯 What's Next?

1. **Change Default Password** - This is critical for security!
2. **Add More Users** - Create accounts for other users
3. **Upload Music** - Try uploading a test file
4. **Configure Ferric** - Ensure Ferric is installed for audio processing
5. **Set Up SSL** - Configure reverse proxy with Let's Encrypt (for production)
6. **Backup** - Set up automated database backups

---

## 📚 More Information

- **This Guide**: Quick start and basic usage
- **SETUP.md**: Detailed configuration options
- **README.md**: Architecture and development details
- **CLAUDE.md**: Development guidelines for working with Claude Code
- **PROJECT_SUMMARY.md**: Project overview and technical details

---

## 🆘 Getting Help

1. Check the logs: `docker-compose logs music-upload` (Docker) or console output (local)
2. Verify configuration: review `config.toml`
3. Test database connection: `docker exec music_upload_db mysql -u music_upload -p`
4. Review documentation in this repository
5. Check the troubleshooting section above

---

## 🎉 Success!

Once you're logged in and have changed the default password:
- Upload audio files via the web interface
- Download from YouTube URLs
- Ferric will automatically process and organize your music
- Files appear in your Navidrome music library

**Enjoy your music upload service!**

### Default Admin Credentials (First Login Only)
- Username: `admin`
- Password: `admin`
- **🔐 CHANGE THIS IMMEDIATELY AFTER FIRST LOGIN!**
