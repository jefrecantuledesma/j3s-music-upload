#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     J3S Music Upload Service - Setup Script           ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo ""

# Function to generate random password
generate_password() {
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-25
}

# Function to generate JWT secret
generate_jwt_secret() {
    openssl rand -base64 32
}

# Function to prompt for yes/no
prompt_yes_no() {
    local prompt="$1"
    local response
    while true; do
        read -p "$prompt (y/n): " response
        case "$response" in
            [Yy]* ) return 0;;
            [Nn]* ) return 1;;
            * ) echo "Please answer yes or no.";;
        esac
    done
}

# Check if script is run from project root
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Please run this script from the project root directory${NC}"
    exit 1
fi

echo -e "${GREEN}Welcome to the J3S Music Upload Service setup!${NC}"
echo ""
echo "This script will help you configure the service for either:"
echo "  1. Docker deployment (recommended for production)"
echo "  2. Local development (running directly with cargo)"
echo ""

# Ask for deployment mode
echo -e "${YELLOW}Choose your deployment mode:${NC}"
echo "  1) Docker (recommended - easiest setup)"
echo "  2) Local development (requires MariaDB installed)"
echo ""
read -p "Enter choice (1 or 2): " MODE

if [ "$MODE" != "1" ] && [ "$MODE" != "2" ]; then
    echo -e "${RED}Invalid choice. Please run the script again and choose 1 or 2.${NC}"
    exit 1
fi

# Generate secrets
echo ""
echo -e "${BLUE}Generating secure secrets...${NC}"
DB_PASSWORD=$(generate_password)
JWT_SECRET=$(generate_jwt_secret)
echo -e "${GREEN}✓ Secrets generated${NC}"

if [ "$MODE" = "1" ]; then
    # Docker mode
    echo ""
    echo -e "${YELLOW}=== Docker Setup ===${NC}"
    echo ""

    # Get music directory
    read -p "Enter the path to your Navidrome music directory [/srv/navidrome/music]: " MUSIC_DIR
    MUSIC_DIR=${MUSIC_DIR:-/srv/navidrome/music}

    # Get server port
    read -p "Enter the port for the web server [8080]: " SERVER_PORT
    SERVER_PORT=${SERVER_PORT:-8080}

    # Get Ferric path
    read -p "Enter the path to Ferric executable [/usr/local/bin/ferric]: " FERRIC_PATH
    FERRIC_PATH=${FERRIC_PATH:-/usr/local/bin/ferric}

    echo ""
    echo -e "${BLUE}Creating configuration files...${NC}"

    # Create config.toml for Docker
    cat > config.toml <<EOF
# Music Upload Service Configuration (Docker Mode)

[server]
host = "0.0.0.0"
port = $SERVER_PORT

[database]
url = "mysql://music_upload:$DB_PASSWORD@mariadb:3306/music_upload"
max_connections = 5

[paths]
# Main Navidrome music directory
music_dir = "$MUSIC_DIR"
# Temporary directory for uploads and processing
temp_dir = "$MUSIC_DIR/tmp"
# Path to the Ferric executable
ferric_path = "$FERRIC_PATH"

[security]
# JWT secret for token signing (auto-generated)
jwt_secret = "$JWT_SECRET"
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
EOF

    # Create docker-compose.yml with matching password
    cat > docker-compose.yml <<EOF
version: '3.8'

services:
  mariadb:
    image: mariadb:latest
    container_name: music_upload_db
    environment:
      MYSQL_ROOT_PASSWORD: $DB_PASSWORD
      MYSQL_DATABASE: music_upload
      MYSQL_USER: music_upload
      MYSQL_PASSWORD: $DB_PASSWORD
    volumes:
      - music_upload_db:/var/lib/mysql
    networks:
      - music_upload_net
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
      interval: 10s
      timeout: 5s
      retries: 5

  music-upload:
    build: .
    container_name: music_upload_app
    ports:
      - "$SERVER_PORT:$SERVER_PORT"
    volumes:
      - ./config.toml:/app/config.toml:ro
      - $MUSIC_DIR:$MUSIC_DIR
    depends_on:
      mariadb:
        condition: service_healthy
    networks:
      - music_upload_net
    restart: unless-stopped
    environment:
      - RUST_LOG=info
      - CONFIG_PATH=/app/config.toml

volumes:
  music_upload_db:

networks:
  music_upload_net:
EOF

    echo -e "${GREEN}✓ Docker configuration created${NC}"
    echo ""
    echo -e "${GREEN}Setup complete!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Ensure Docker and Docker Compose are installed"
    echo "  2. Start the services: ${BLUE}docker-compose up -d${NC}"
    echo "  3. View logs: ${BLUE}docker-compose logs -f music-upload${NC}"
    echo "  4. Access the web interface at ${GREEN}http://localhost:$SERVER_PORT${NC}"
    echo ""
    echo -e "${RED}IMPORTANT:${NC}"
    echo "  Default login credentials:"
    echo "    Username: ${GREEN}admin${NC}"
    echo "    Password: ${GREEN}admin${NC}"
    echo ""
    echo "  ${RED}PLEASE CHANGE THE DEFAULT PASSWORD IMMEDIATELY AFTER FIRST LOGIN!${NC}"
    echo "  Use the 'Change Password' feature in the web interface."
    echo ""
    echo -e "${YELLOW}Database credentials (saved in docker-compose.yml):${NC}"
    echo "  Database: music_upload"
    echo "  Username: music_upload"
    echo "  Password: $DB_PASSWORD"

elif [ "$MODE" = "2" ]; then
    # Local development mode
    echo ""
    echo -e "${YELLOW}=== Local Development Setup ===${NC}"
    echo ""

    # Database configuration
    echo "Please provide your MariaDB/MySQL database connection details:"
    read -p "Database host [localhost]: " DB_HOST
    DB_HOST=${DB_HOST:-localhost}

    read -p "Database port [3306]: " DB_PORT
    DB_PORT=${DB_PORT:-3306}

    read -p "Database name [music_upload]: " DB_NAME
    DB_NAME=${DB_NAME:-music_upload}

    read -p "Database username [music_upload]: " DB_USER
    DB_USER=${DB_USER:-music_upload}

    read -sp "Database password (leave empty to generate random): " DB_PASSWORD_INPUT
    echo ""
    if [ -z "$DB_PASSWORD_INPUT" ]; then
        DB_PASSWORD=$(generate_password)
        echo -e "${GREEN}Generated random password: $DB_PASSWORD${NC}"
        echo -e "${YELLOW}Please create the database user with this password${NC}"
    else
        DB_PASSWORD="$DB_PASSWORD_INPUT"
    fi

    # Get music directory
    read -p "Enter the path to your Navidrome music directory [/srv/navidrome/music]: " MUSIC_DIR
    MUSIC_DIR=${MUSIC_DIR:-/srv/navidrome/music}

    # Get server port
    read -p "Enter the port for the web server [8080]: " SERVER_PORT
    SERVER_PORT=${SERVER_PORT:-8080}

    # Get Ferric path
    read -p "Enter the path to Ferric executable [/usr/local/bin/ferric]: " FERRIC_PATH
    FERRIC_PATH=${FERRIC_PATH:-/usr/local/bin/ferric}

    echo ""
    echo -e "${BLUE}Creating configuration files...${NC}"

    # Create config.toml for local development
    cat > config.toml <<EOF
# Music Upload Service Configuration (Local Development)

[server]
host = "127.0.0.1"
port = $SERVER_PORT

[database]
url = "mysql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
max_connections = 5

[paths]
# Main Navidrome music directory
music_dir = "$MUSIC_DIR"
# Temporary directory for uploads and processing
temp_dir = "$MUSIC_DIR/tmp"
# Path to the Ferric executable
ferric_path = "$FERRIC_PATH"

[security]
# JWT secret for token signing (auto-generated)
jwt_secret = "$JWT_SECRET"
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
EOF

    echo -e "${GREEN}✓ Configuration created${NC}"
    echo ""

    # Offer to create database
    if prompt_yes_no "Would you like to see the SQL commands to create the database?"; then
        echo ""
        echo -e "${BLUE}Run these SQL commands in your MariaDB/MySQL:${NC}"
        echo ""
        echo "CREATE DATABASE IF NOT EXISTS $DB_NAME;"
        echo "CREATE USER IF NOT EXISTS '$DB_USER'@'$DB_HOST' IDENTIFIED BY '$DB_PASSWORD';"
        echo "GRANT ALL PRIVILEGES ON $DB_NAME.* TO '$DB_USER'@'$DB_HOST';"
        echo "FLUSH PRIVILEGES;"
        echo ""
    fi

    # Create directories
    echo -e "${BLUE}Creating directories...${NC}"
    mkdir -p "$MUSIC_DIR/tmp"
    echo -e "${GREEN}✓ Directories created${NC}"

    echo ""
    echo -e "${GREEN}Setup complete!${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Ensure MariaDB is running and the database is created"
    echo "  2. Build the project: ${BLUE}cargo build --release${NC}"
    echo "  3. Run the application: ${BLUE}cargo run --release${NC}"
    echo "  4. Access the web interface at ${GREEN}http://localhost:$SERVER_PORT${NC}"
    echo ""
    echo -e "${RED}IMPORTANT:${NC}"
    echo "  Default login credentials:"
    echo "    Username: ${GREEN}admin${NC}"
    echo "    Password: ${GREEN}admin${NC}"
    echo ""
    echo "  ${RED}PLEASE CHANGE THE DEFAULT PASSWORD IMMEDIATELY AFTER FIRST LOGIN!${NC}"
    echo "  Use the API endpoint: POST /api/user/change-password"
fi

echo ""
echo -e "${GREEN}Configuration files have been created:${NC}"
echo "  - config.toml (main configuration)"
if [ "$MODE" = "1" ]; then
    echo "  - docker-compose.yml (Docker services)"
fi
echo ""
echo -e "${BLUE}For more information, see:${NC}"
echo "  - README.md"
echo "  - SETUP.md"
echo "  - QUICKSTART.md"
echo ""
