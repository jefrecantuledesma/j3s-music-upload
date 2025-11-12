# Quick Start Guide

Get J3S Music Upload Service running in under 5 minutes!

## 🚀 Super Quick Setup (Recommended)

### Option 1: Automated Setup Script (Easiest!)

1. **Run the setup script:**
   ```bash
   chmod +x scripts/setup.sh
   ./scripts/setup.sh
   ```

   The script will:
   - Create `config.toml` with auto-generated JWT secret
   - Set up SQLite database (no server needed!)
   - Build the application
   - Create an admin user with your chosen password

2. **Start the service:**
   ```bash
   ./target/release/j3s_music_upload
   ```

   Or use cargo:
   ```bash
   cargo run --release
   ```

3. **Access the web interface:**
   - Open your browser to `http://localhost:8080`
   - Login with the admin credentials you created

That's it! No database server, no complex setup. Just run the script and go!

---

### Option 2: Docker Setup (Production Ready)

1. **Create shared network for Caddy (if using reverse proxy):**
   ```bash
   docker network create shared_web
   ```

2. **Copy and configure:**
   ```bash
   cp config.toml.example config.toml
   # Edit config.toml with your settings
   ```

3. **Start the service:**
   ```bash
   docker-compose up -d
   ```

4. **Check logs:**
   ```bash
   docker-compose logs -f music-upload
   ```

5. **Access the web interface:**
   - Direct: `http://localhost:8080`
   - Via Caddy: `https://music-upload.yourdomain.com`
   - Login with **admin/admin** (change immediately!)

---

## 🔐 First Login & Security

### Default Credentials
On first startup, if no users exist, the application creates:
- **Username:** `admin`
- **Password:** `admin`

**🔒 IMPORTANT: Change this immediately after first login!**

### Change Password (API)

```bash
# First, login to get your JWT token
TOKEN=$(curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.token')

# Change your password
curl -X POST http://localhost:8080/api/user/change-password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "old_password": "admin",
    "new_password": "your_new_secure_password"
  }'
```

---

## 📤 Uploading Music

### File Upload
1. Navigate to the upload page
2. Select audio files (MP3, FLAC, OGG, OPUS, M4A, WAV, AAC)
3. Click Upload
4. Ferric will process and organize files automatically

### YouTube Download
1. Navigate to the upload page
2. Enter a YouTube URL
3. Service downloads audio and processes it with Ferric

---

## 🛠️ Useful Commands

### Local Development

```bash
# Build
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run

# Run setup script
./scripts/setup.sh

# Check database
sqlite3 data/music_upload.db ".tables"
sqlite3 data/music_upload.db "SELECT * FROM users;"
```

### Docker

```bash
# View logs
docker-compose logs -f music-upload

# Restart service
docker-compose restart

# Stop service
docker-compose down

# Rebuild and restart
docker-compose build && docker-compose up -d

# Check database
docker exec -it music_upload_service sqlite3 /app/data/music_upload.db ".tables"
```

---

## 🔧 Troubleshooting

### "Failed to connect to database"
- **SQLite:** Ensure the `data/` directory exists and is writable
- **Permissions:** Check that the application can write to the data directory
- The database file will be created automatically on first run

### "Ferric command not found"
- Install Ferric from the ferric repository
- Update `ferric_path` in `config.toml` to the correct location
- Ferric is optional - uploads will work without it (no processing)

### "Permission denied" on directories
- Ensure music and temp directories are writable
- The app will try to create directories automatically
- For Docker: check volume mounts in `docker-compose.yml`

### Upload fails
- Check file size limits in `config.toml` (default 500MB)
- Ensure file extension is in `allowed_extensions` list
- Check disk space in music and temp directories
- Review logs for specific errors

### Docker: GLIBC version errors
- Make sure you rebuild after pulling latest changes
- The Dockerfile now uses matching Debian versions

### Caddy reverse proxy not working
- Ensure shared_web network exists: `docker network create shared_web`
- Check that docker-compose.yml includes the shared_web network
- Verify Caddy can reach the container: `docker network inspect shared_web`

---

## 📋 Setup Checklist

**Automated Setup:**
- [ ] Ran `./scripts/setup.sh`
- [ ] Created admin user with secure password
- [ ] Started service with `cargo run --release` or `./target/release/j3s_music_upload`
- [ ] Accessed web interface at http://localhost:8080
- [ ] Logged in successfully
- [ ] Uploaded test file (optional)

**Docker Setup:**
- [ ] Created shared_web network (if using Caddy)
- [ ] Copied and configured config.toml
- [ ] Started services with `docker-compose up -d`
- [ ] Checked logs with `docker-compose logs`
- [ ] Accessed web interface
- [ ] Changed default admin password
- [ ] Updated Caddyfile (if using reverse proxy)

---

## 🎯 What Makes This Simple?

### No Database Server Required!
- Uses SQLite - single file database
- No MySQL/MariaDB/PostgreSQL to install
- No database users or permissions to configure
- Automatic migrations on startup
- Just run and go!

### Automatic Setup
- Auto-generates JWT secrets
- Creates admin user on first run
- Initializes database automatically
- Creates necessary directories

### Easy Configuration
- Single `config.toml` file
- Sensible defaults
- Clear examples provided

---

## 📁 Directory Structure

```
j3s_music_upload/
├── data/
│   └── music_upload.db        # SQLite database (auto-created)
├── config.toml                # Your configuration
├── src/                       # Rust source code
├── migrations/                # Database migrations (auto-applied)
├── templates/                 # HTML templates
├── scripts/
│   └── setup.sh              # Setup wizard
└── target/release/
    └── j3s_music_upload      # Compiled binary
```

---

## 🔗 API Endpoints

### Authentication
- `POST /api/login` - Login and get JWT token
- `POST /api/logout` - Logout

### User Management (Admin Only)
- `GET /api/admin/users` - List all users
- `POST /api/admin/users` - Create new user
- `DELETE /api/admin/users/:id` - Delete user
- `POST /api/admin/users/:id/password` - Change user password

### Upload
- `POST /api/upload` - Upload audio files
- `POST /api/youtube` - Download from YouTube URL

### Logs & Config (Admin Only)
- `GET /api/admin/logs` - View upload logs
- `GET /api/admin/config` - List config values
- `POST /api/admin/config` - Update config value

---

## 🎉 Success!

Once you're logged in:
- Upload audio files via the web interface
- Download from YouTube URLs
- Ferric automatically processes and organizes music
- Files appear in your Navidrome music library

**Enjoy your music upload service!**

---

## 📚 More Information

- **QUICKSTART.md** (this file): Get started quickly
- **SETUP.md**: Detailed configuration options
- **README.md**: Architecture and development details
- **CLAUDE.md**: Development guidelines
- **PROJECT_SUMMARY.md**: Project overview

---

## 🆘 Getting Help

1. Check the logs (console output or `docker-compose logs`)
2. Verify configuration in `config.toml`
3. Check database: `sqlite3 data/music_upload.db ".tables"`
4. Review troubleshooting section above
5. Check file permissions on data/ and music directories
