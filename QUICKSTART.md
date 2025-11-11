# Quick Start Guide

## 🚀 Get Running in 5 Minutes

### Step 1: Configure (2 minutes)

```bash
# Copy example config
cp config.toml.example config.toml

# Generate a secure JWT secret
openssl rand -base64 32

# Edit config.toml and:
# 1. Paste the JWT secret into the jwt_secret field
# 2. Update database password (choose a strong password)
# 3. Verify paths are correct for your system
nano config.toml
```

### Step 2: Update Docker Compose (1 minute)

```bash
# Edit docker-compose.yml
nano docker-compose.yml

# Update MYSQL_PASSWORD and MYSQL_ROOT_PASSWORD to match your config.toml
# Make sure the password matches what you put in config.toml!
```

### Step 3: Start Services (1 minute)

```bash
# Build and start everything
docker-compose up -d

# Wait for database to initialize (30 seconds)
sleep 30

# Check if services are running
docker-compose ps
```

### Step 4: Create Admin User (1 minute)

```bash
# Easy way - use the initialization script
./scripts/init_admin.sh

# Or manual way:
# 1. Generate password hash
cargo run --example hash_password "YourAdminPassword"

# 2. Copy the INSERT statement from output
# 3. Connect to database and run it
docker exec -it music_upload_db mysql -u music_upload -p music_upload
# Paste the INSERT statement
```

### Step 5: Login! (30 seconds)

Open your browser: **http://localhost:8080**

Login with your admin credentials and start uploading music!

## 📋 Checklist

- [ ] Copied config.toml.example to config.toml
- [ ] Generated and set JWT secret
- [ ] Updated database passwords in both config.toml and docker-compose.yml
- [ ] Verified Navidrome music directory exists: /srv/navidrome/music
- [ ] Started services: docker-compose up -d
- [ ] Created admin user
- [ ] Logged in successfully

## 🔧 Common Issues

### "Database connection failed"
- Check if mariadb container is running: `docker-compose ps`
- Verify password in config.toml matches docker-compose.yml
- Wait 30 seconds after first start for database initialization

### "Cannot create admin user"
- Username might already exist
- Check database logs: `docker-compose logs mariadb`
- Verify database is initialized: `docker exec music_upload_db mysql -u music_upload -p -e "SHOW TABLES;"`

### "Upload fails"
- Check if Ferric is installed (you may need to build/install it separately)
- Verify temp directory exists and has correct permissions
- Check application logs: `docker-compose logs music-upload`

### "YouTube download doesn't work"
- Verify yt-dlp is installed in container: `docker exec music-upload yt-dlp --version`
- Check if YouTube URL is valid
- Review logs for specific errors

## 📁 Project Structure

```
.
├── config.toml          # Your configuration (create this)
├── docker-compose.yml   # Docker orchestration
├── Dockerfile           # Container definition
├── src/                 # Rust source code
├── templates/           # HTML templates
├── migrations/          # Database migrations
├── scripts/             # Helper scripts
└── examples/            # Utility programs
```

## 🎯 What's Next?

1. **Add More Users**: Use the Admin panel to create user accounts
2. **Upload Music**: Try uploading a test file
3. **Configure Ferric**: Make sure your Ferric installation is working
4. **Set Up SSL**: Configure reverse proxy with Let's Encrypt
5. **Backup**: Set up automated database backups

## 📚 More Information

- **Full Setup Guide**: See SETUP.md
- **Project Details**: See PROJECT_SUMMARY.md
- **User Manual**: See README.md

## 🆘 Getting Help

1. Check the logs: `docker-compose logs music-upload`
2. Verify configuration: review config.toml
3. Test database: `docker exec music_upload_db mysql -u music_upload -p`
4. Review documentation in this repository

## 🎉 Success!

Once you're logged in:
- Upload audio files via the web interface
- Download from YouTube (CC0/open source content only!)
- Ferric will automatically process and organize your music
- Files appear in your Navidrome library

**Enjoy your music upload service!**
