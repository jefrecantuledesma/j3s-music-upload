# Security Guide

## Overview

J3S Music Upload Service is designed for world-facing deployments. This document outlines the security measures implemented and best practices for secure deployment.

## Built-in Security Features

### Authentication & Authorization
- **JWT-based authentication** with configurable expiration
- **Argon2 password hashing** with random salts (industry standard)
- **Role-based access control** (admin vs regular users)
- **Protected routes** - All sensitive endpoints require authentication
- **Default admin password warning** - System warns on first startup

### Input Validation & Sanitization
- **Path traversal protection** - Filenames sanitized to prevent directory traversal attacks
- **File type whitelisting** - Only allowed audio extensions accepted
- **File size limits** - Configurable maximum file size (default 500MB)
- **YouTube URL validation** - Strict HTTPS-only URL patterns, command injection prevention
- **Request body size limits** - Enforced at the HTTP layer

### SQL Injection Protection
- **Parameterized queries** - All database queries use sqlx prepared statements
- **No raw SQL** - Zero user input directly concatenated into queries

### Session Management
- **Configurable timeouts** - JWT tokens expire based on `session_timeout_hours`
- **Secure secret generation** - Auto-generates cryptographically random JWT secrets
- **Token invalidation** - Logout endpoint for client-side token removal

### Rate Limiting & Resource Protection
- **Request body limits** - Enforced by tower-http middleware
- **File size validation** - Checked before processing begins
- **Upload logging** - All uploads tracked with user ID and status

## Deployment Security Checklist

### Essential Steps

- [ ] **Change default admin password immediately** after first login
- [ ] **Use HTTPS only** - Deploy behind a reverse proxy (Caddy/Nginx) with valid SSL/TLS certificates
- [ ] **Restrict network access** - Don't expose port 8080 directly; use reverse proxy
- [ ] **Generate unique JWT secret** - Use `openssl rand -base64 32` or let setup script generate
- [ ] **Set secure file permissions** on config.toml (contains JWT secret)
- [ ] **Keep SQLite database secure** - Ensure `data/` directory has proper permissions
- [ ] **Review Caddyfile/Nginx config** - Ensure security headers are set

### Recommended Steps

- [ ] **Configure firewall** - Only allow ports 80/443 for reverse proxy
- [ ] **Enable fail2ban** or similar for brute force protection
- [ ] **Regular backups** - Backup SQLite database periodically
- [ ] **Monitor logs** - Review upload logs and auth attempts regularly
- [ ] **Update dependencies** - Keep Rust dependencies up-to-date (`cargo update`)
- [ ] **Limit user creation** - Only admins should create accounts
- [ ] **Use strong passwords** - Enforce password policies for all users

### Advanced Security (Production)

- [ ] **Implement rate limiting** - Use Caddy/Nginx rate limiting for API endpoints
- [ ] **Add IP whitelisting** if service is for private use
- [ ] **Set up monitoring/alerts** for suspicious activity
- [ ] **Consider 2FA** for admin accounts (future feature)
- [ ] **Restrict CORS** - Specify exact allowed origins in `main.rs` instead of `Any`
- [ ] **Review Ferric security** - Ensure Ferric path is not user-modifiable

## Configuration Security

### JWT Secret

The JWT secret is critical for session security:

```toml
[security]
jwt_secret = "CHANGE_THIS_TO_A_RANDOM_SECRET"  # CRITICAL!
session_timeout_hours = 24
```

**Generate a secure secret:**
```bash
openssl rand -base64 32
```

### File Permissions

```bash
# Lock down config file (contains JWT secret)
chmod 600 config.toml

# Secure database
chmod 700 data/
chmod 600 data/music_upload.db

# Ensure music directories are writable by service only
chown -R app:app /srv/navidrome/music
chmod 755 /srv/navidrome/music
```

### Reverse Proxy Configuration

**Example Caddy configuration with security headers:**

```caddy
music-upload.yourdomain.com {
    reverse_proxy music_upload_service:8080 {
        transport http {
            read_timeout 300s
            write_timeout 300s
        }
    }

    # Request body limits
    request_body {
        max_size 500MB
    }

    # Security headers
    header {
        # HTTPS enforcement
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"

        # Clickjacking protection
        X-Frame-Options "DENY"

        # MIME sniffing protection
        X-Content-Type-Options "nosniff"

        # XSS protection (legacy browsers)
        X-XSS-Protection "1; mode=block"

        # Referrer policy
        Referrer-Policy "strict-origin-when-cross-origin"

        # Content Security Policy (adjust as needed)
        Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';"

        # Remove server info
        -Server
    }

    # Rate limiting (Caddy Pro feature)
    # rate_limit {
    #     zone api {
    #         key {remote_host}
    #         events 100
    #         window 1m
    #     }
    # }

    # Compression
    encode gzip zstd

    # Logging
    log {
        output file /var/log/caddy/music-upload.log
        format json
    }
}
```

**Example Nginx configuration:**

```nginx
server {
    listen 443 ssl http2;
    server_name music-upload.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # File upload limits
    client_max_body_size 500M;

    # Timeouts for large uploads
    proxy_connect_timeout 300s;
    proxy_send_timeout 300s;
    proxy_read_timeout 300s;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    location / {
        proxy_pass http://music_upload_service:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Security Features by Layer

### Network Layer
- **No direct port exposure** in production
- **HTTPS enforcement** via reverse proxy
- **Firewall rules** restrict access

### Application Layer
- **Authentication required** for all sensitive operations
- **Input validation** on all user inputs
- **Path sanitization** prevents traversal attacks
- **Command injection prevention** in YouTube downloads
- **CORS configuration** limits cross-origin requests
- **Request size limits** prevent DoS via large uploads

### Database Layer
- **Parameterized queries** prevent SQL injection
- **Password hashing** with Argon2
- **SQLite WAL mode** for better concurrency (auto-enabled)

### File System Layer
- **Extension validation** prevents arbitrary file uploads
- **Size limits** prevent disk exhaustion
- **Temp directory cleanup** after processing
- **Ferric sandboxing** (depends on Ferric implementation)

## Threat Model

### Threats Mitigated

 **SQL Injection** - Parameterized queries
 **Path Traversal** - Filename sanitization
 **Command Injection** - URL validation, no shell expansion
 **XSS** - Content-Type headers, CSP via reverse proxy
 **CSRF** - JWT token required for state changes
 **Brute Force** - Rate limiting via reverse proxy
 **Session Hijacking** - HTTPS only, secure JWT secrets
 **Arbitrary File Upload** - Extension whitelisting
 **DoS via Large Files** - Size limits enforced

### Threats Requiring Additional Mitigation

  **Brute Force Login** - Add fail2ban or Caddy rate limiting
  **Account Enumeration** - Login errors are generic but timing may reveal users
  **DoS via API Spam** - Rate limiting recommended at reverse proxy
  **Malicious Audio Files** - Depends on Ferric/yt-dlp security

### Out of Scope

L **Physical Access** - Assumed server is physically secure
L **Compromised Dependencies** - Regular updates recommended
L **Social Engineering** - User education required

## Incident Response

If you suspect a security breach:

1. **Immediately** change all passwords
2. **Rotate JWT secret** in config.toml (invalidates all sessions)
3. **Review logs** in `docker-compose logs` or application logs
4. **Check database** for unauthorized users: `sqlite3 data/music_upload.db "SELECT * FROM users;"`
5. **Review upload logs**: `sqlite3 data/music_upload.db "SELECT * FROM upload_logs ORDER BY created_at DESC;"`
6. **Backup and restore** database if tampering suspected
7. **Update** all dependencies: `cargo update`
8. **Report issues** to the project maintainers

## Security Updates

To update dependencies for security patches:

```bash
# Update Rust dependencies
cargo update

# Check for outdated dependencies
cargo outdated

# Rebuild Docker image
docker-compose build --no-cache

# Restart service
docker-compose up -d
```

## Reporting Security Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email the maintainers directly (if contact info available)
3. Provide detailed information about the vulnerability
4. Allow reasonable time for a fix before public disclosure

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Caddy Security](https://caddyserver.com/docs/security)
- [SQLite Security](https://www.sqlite.org/security.html)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)

## License & Disclaimer

This software is provided as-is. While security best practices have been followed, no software is 100% secure. Use at your own risk and always follow defense-in-depth principles.
