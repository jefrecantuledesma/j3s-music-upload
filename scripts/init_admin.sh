#!/bin/bash
# Script to initialize the first admin user for the Music Upload Service

set -e

echo "==============================================="
echo "Music Upload Service - Admin Initialization"
echo "==============================================="
echo ""
echo "This script will help you create the first admin user."
echo ""

# Check if docker-compose is running
if ! docker-compose ps | grep -q "music_upload_db.*Up"; then
    echo "Error: Database container is not running!"
    echo "Please start the services first: docker-compose up -d"
    exit 1
fi

# Prompt for username
read -p "Enter admin username [admin]: " USERNAME
USERNAME=${USERNAME:-admin}

# Prompt for password
while true; do
    read -sp "Enter admin password (min 8 characters): " PASSWORD
    echo ""
    if [ ${#PASSWORD} -lt 8 ]; then
        echo "Error: Password must be at least 8 characters long"
        continue
    fi

    read -sp "Confirm password: " PASSWORD_CONFIRM
    echo ""

    if [ "$PASSWORD" = "$PASSWORD_CONFIRM" ]; then
        break
    else
        echo "Error: Passwords do not match. Please try again."
    fi
done

echo ""
echo "Creating admin user..."

# Generate password hash using the example program
HASH_OUTPUT=$(cargo run --example hash_password "$PASSWORD" 2>/dev/null | grep "^\$argon2")

if [ -z "$HASH_OUTPUT" ]; then
    echo "Error: Failed to generate password hash"
    exit 1
fi

# Get database credentials from docker-compose
DB_PASSWORD=$(grep "MYSQL_PASSWORD:" docker-compose.yml | head -1 | awk '{print $3}')

# Insert user into database
docker exec -i music_upload_db mysql -u music_upload -p"$DB_PASSWORD" music_upload <<EOF
INSERT INTO users (id, username, password_hash, is_admin)
VALUES (UUID(), '$USERNAME', '$HASH_OUTPUT', true);
EOF

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Admin user created successfully!"
    echo ""
    echo "You can now login at http://localhost:8080 with:"
    echo "  Username: $USERNAME"
    echo "  Password: (the password you just entered)"
    echo ""
else
    echo ""
    echo "✗ Failed to create admin user."
    echo "  The username might already exist, or there's a database issue."
    echo "  Check the logs: docker-compose logs mariadb"
    exit 1
fi
