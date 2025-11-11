# Setup Guide

## Quick Start

### 1. Initial Setup

Copy the example configuration:
```bash
cp config.toml.example config.toml
```

### 2. Configure the Application

Edit `config.toml` with your settings:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
# Update this to match your docker-compose.yml database settings
url = "mysql://music_upload:your_password_here@mariadb:3306/music_upload"
max_connections = 5

[paths]
music_dir = "/srv/navidrome/music"
temp_dir = "/srv/navidrome/music/tmp"
ferric_path = "/usr/local/bin/ferric"

[security]
# Generate a secure secret: openssl rand -base64 32
jwt_secret = "REPLACE_WITH_SECURE_SECRET"
session_timeout_hours = 24

[upload]
max_file_size_mb = 500
allowed_extensions = ["mp3", "flac", "ogg", "opus", "m4a", "wav", "aac"]

[youtube]
enabled = true
ytdlp_path = "yt-dlp"
audio_format = "best"
```

### 3. Generate JWT Secret

```bash
openssl rand -base64 32
```

Copy the output and paste it into `config.toml` as the `jwt_secret` value.

### 4. Update Docker Compose

Edit `docker-compose.yml` and update the database passwords:

```yaml
environment:
  - MYSQL_ROOT_PASSWORD=your_strong_root_password
  - MYSQL_DATABASE=music_upload
  - MYSQL_USER=music_upload
  - MYSQL_PASSWORD=your_strong_password
```

**Important**: Make sure the password in `docker-compose.yml` matches the password in your `config.toml` database URL!

### 5. Build and Start Services

```bash
# Build the Docker image
docker-compose build

# Start the services
docker-compose up -d

# Check logs
docker-compose logs -f music-upload
```

### 6. Create First Admin User

Once the database is initialized (wait about 30 seconds after first start), create an admin user:

```bash
# Generate a password hash
cargo run --example hash_password "your_admin_password"
```

This will output SQL that you can run. Connect to the database:

```bash
docker exec -it music_upload_db mysql -u music_upload -p music_upload
```

Enter your database password, then paste the INSERT statement from the hash_password output.

Alternatively, use a quick command:
```bash
# Example (replace with your actual hash)
docker exec -i music_upload_db mysql -u music_upload -pyour_password music_upload <<EOF
INSERT INTO users (id, username, password_hash, is_admin)
VALUES (UUID(), 'admin', '\$argon2id\$v=19\$m=19456,t=2,p=1\$...', true);
EOF
```

### 7. Access the Application

Open your browser and navigate to:
- Local: `http://localhost:8080`
- Production: `http://music-upload.jcledesma.xyz`

Login with your admin credentials!

## Troubleshooting

### Database Connection Issues

```bash
# Check if database is running
docker-compose ps

# Check database logs
docker-compose logs mariadb

# Test database connection
docker exec -it music_upload_db mysql -u music_upload -p -e "SHOW DATABASES;"
```

### Application Won't Start

```bash
# Check application logs
docker-compose logs music-upload

# Verify config file exists
ls -la config.toml

# Check file permissions
docker-compose exec music-upload ls -la /app/
```

### Can't Upload Files

1. Verify Ferric is installed and accessible
2. Check temp directory permissions
3. Check music directory permissions
4. Review application logs for errors

```bash
# Check Ferric
docker-compose exec music-upload /usr/local/bin/ferric --version

# Check directory permissions
docker-compose exec music-upload ls -la /srv/navidrome/music/
```

### YouTube Downloads Failing

1. Ensure yt-dlp is installed in container
2. Check network connectivity from container
3. Verify YouTube URL format

```bash
# Test yt-dlp
docker-compose exec music-upload yt-dlp --version
```

## Production Deployment

### Reverse Proxy (nginx)

Example nginx configuration:

```nginx
server {
    listen 80;
    server_name music-upload.jcledesma.xyz;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Increase timeout for large uploads
        proxy_read_timeout 300;
        proxy_connect_timeout 300;
        proxy_send_timeout 300;

        # Allow large uploads
        client_max_body_size 500M;
    }
}
```

### SSL with Let's Encrypt

```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d music-upload.jcledesma.xyz

# Auto-renewal is configured automatically
```

### Backup Strategy

```bash
# Backup database
docker exec music_upload_db mysqldump -u music_upload -p music_upload > backup.sql

# Backup configuration
cp config.toml config.toml.backup

# Backup docker compose
cp docker-compose.yml docker-compose.yml.backup
```

### Updating the Application

```bash
# Pull latest changes
git pull

# Rebuild image
docker-compose build

# Restart services
docker-compose up -d

# Check logs
docker-compose logs -f music-upload
```

## Security Checklist

- [ ] Strong JWT secret generated (32+ characters)
- [ ] Strong database passwords set
- [ ] Firewall configured (only allow ports 80/443)
- [ ] SSL certificate installed (HTTPS)
- [ ] Regular backups configured
- [ ] Admin accounts use strong passwords
- [ ] File size limits configured appropriately
- [ ] Only trusted users have access

## Support

For issues or questions:
1. Check the logs: `docker-compose logs music-upload`
2. Review this setup guide
3. Check the main README.md
4. Open an issue on GitHub
