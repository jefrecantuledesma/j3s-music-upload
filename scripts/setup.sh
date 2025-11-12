#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}J3S Music Upload - Setup Script${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Change to project root
cd "$(dirname "$0")/.."

# Check if config.toml already exists
if [ -f "config.toml" ]; then
    echo -e "${YELLOW}Warning: config.toml already exists!${NC}"
    read -p "Do you want to overwrite it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}Setup cancelled.${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Step 1: Creating config.toml from template...${NC}"

# Generate a random JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Ask user for configuration
read -p "Server port [8080]: " SERVER_PORT
SERVER_PORT=${SERVER_PORT:-8080}

read -p "Music directory [/srv/navidrome/music]: " MUSIC_DIR
MUSIC_DIR=${MUSIC_DIR:-/srv/navidrome/music}

read -p "Ferric path [/usr/local/bin/ferric]: " FERRIC_PATH
FERRIC_PATH=${FERRIC_PATH:-/usr/local/bin/ferric}

# Create config.toml
cat > config.toml << EOF
# Music Upload Service Configuration

[server]
host = "0.0.0.0"
port = ${SERVER_PORT}

[database]
# SQLite database file (will be created automatically)
url = "sqlite:./data/music_upload.db"
max_connections = 5

[paths]
# Main Navidrome music directory
music_dir = "${MUSIC_DIR}"
# Temporary directory for uploads and processing
temp_dir = "${MUSIC_DIR}/tmp"
# Path to the Ferric executable
ferric_path = "${FERRIC_PATH}"

[security]
# JWT secret for token signing (auto-generated)
jwt_secret = "${JWT_SECRET}"
# Session timeout in hours
session_timeout_hours = 24

[upload]
# Maximum file size in MB
max_file_size_mb = 500
# Allowed file extensions
allowed_extensions = ["mp3", "flac", "ogg", "opus", "m4a", "wav", "aac"]

[youtube]
# Enable YouTube download feature
enabled = true
# yt-dlp path (or just "yt-dlp" if in PATH)
ytdlp_path = "yt-dlp"
# Audio format preference
audio_format = "best"
# Format selector passed to yt-dlp (--format)
format_selector = "bestaudio/best"
# Player client hint for yt-dlp extractor (web/android/ios). Set empty to disable.
player_client = "web"
# Extra raw arguments appended before the URL
extra_args = []
EOF

echo -e "${GREEN}✓ config.toml created with auto-generated JWT secret${NC}"
echo ""

echo -e "${GREEN}Step 2: Creating necessary directories...${NC}"
mkdir -p data
mkdir -p "${MUSIC_DIR}" 2>/dev/null || echo -e "${YELLOW}Note: ${MUSIC_DIR} may require sudo. You may need to create it manually.${NC}"
mkdir -p "${MUSIC_DIR}/tmp" 2>/dev/null || true
echo -e "${GREEN}✓ Directories created${NC}"
echo ""

echo -e "${GREEN}Step 3: Building the application...${NC}"
cargo build --release
echo -e "${GREEN}✓ Application built successfully${NC}"
echo ""

echo -e "${GREEN}Step 4: Creating admin user...${NC}"
echo ""
read -p "Enter admin username [admin]: " ADMIN_USER
ADMIN_USER=${ADMIN_USER:-admin}
read -s -p "Enter admin password: " ADMIN_PASS
echo ""

# Check if hash_password example exists
if [ ! -f "examples/hash_password.rs" ]; then
    echo -e "${YELLOW}Creating hash_password utility...${NC}"
    mkdir -p examples
    cat > examples/hash_password.rs << 'RUST'
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <password>", args[0]);
        std::process::exit(1);
    }

    let password = &args[1];
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    println!("{}", hash);
}
RUST
fi

# Hash the password
PASS_HASH=$(cargo run --quiet --example hash_password "$ADMIN_PASS" 2>&1 | tail -1)

# Generate UUID for admin user
if command -v uuidgen &> /dev/null; then
    ADMIN_ID=$(uuidgen)
elif [ -f /proc/sys/kernel/random/uuid ]; then
    ADMIN_ID=$(cat /proc/sys/kernel/random/uuid)
else
    # Fallback: generate a simple UUID-like string
    ADMIN_ID=$(cat /dev/urandom | tr -dc 'a-f0-9' | fold -w 32 | head -n 1 | sed 's/\(........\)\(....\)\(....\)\(....\)\(............\)/\1-\2-\3-\4-\5/')
fi

# Wait for database to be initialized
echo -e "${YELLOW}Initializing database (this may take a moment)...${NC}"
timeout 3s ./target/release/j3s_music_upload &>/dev/null || true
sleep 1

# Insert admin user into database
if command -v sqlite3 &> /dev/null; then
    sqlite3 data/music_upload.db << SQL
INSERT INTO users (id, username, password_hash, is_admin, created_at, updated_at)
VALUES ('${ADMIN_ID}', '${ADMIN_USER}', '${PASS_HASH}', 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
SQL
    echo -e "${GREEN}✓ Admin user created successfully${NC}"
else
    echo -e "${YELLOW}Warning: sqlite3 command not found. You'll need to create the admin user manually.${NC}"
    echo -e "${YELLOW}Install sqlite3 and run:${NC}"
    echo "sqlite3 data/music_upload.db << SQL"
    echo "INSERT INTO users (id, username, password_hash, is_admin, created_at, updated_at)"
    echo "VALUES ('${ADMIN_ID}', '${ADMIN_USER}', '${PASS_HASH}', 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);"
    echo "SQL"
fi

echo ""

echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}Setup complete!${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo -e "You can now start the application with:"
echo -e "  ${YELLOW}./target/release/j3s_music_upload${NC}"
echo ""
echo -e "Or run in development mode with:"
echo -e "  ${YELLOW}cargo run${NC}"
echo ""
echo -e "Admin credentials:"
echo -e "  Username: ${GREEN}${ADMIN_USER}${NC}"
echo -e "  Password: ${GREEN}[hidden]${NC}"
echo ""
echo -e "The web interface will be available at:"
echo -e "  ${BLUE}http://localhost:${SERVER_PORT}${NC}"
echo ""
echo -e "${YELLOW}Note:${NC} Database file location: ${GREEN}./data/music_upload.db${NC}"
echo ""
