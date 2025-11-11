#!/bin/bash
# Script to create the first admin user

echo "Music Upload Service - Create Admin User"
echo "========================================="
echo ""

read -p "Enter admin username: " USERNAME
read -sp "Enter admin password: " PASSWORD
echo ""
read -sp "Confirm password: " PASSWORD_CONFIRM
echo ""

if [ "$PASSWORD" != "$PASSWORD_CONFIRM" ]; then
    echo "Error: Passwords do not match"
    exit 1
fi

if [ ${#PASSWORD} -lt 8 ]; then
    echo "Error: Password must be at least 8 characters"
    exit 1
fi

# Generate UUID
USER_ID=$(uuidgen)

echo ""
echo "Creating admin user..."
echo ""
echo "To complete the setup, you need to:"
echo "1. Generate a password hash for: $PASSWORD"
echo "2. Run this SQL command:"
echo ""
echo "INSERT INTO users (id, username, password_hash, is_admin)"
echo "VALUES ('$USER_ID', '$USERNAME', '<HASH_HERE>', true);"
echo ""
echo "You can generate the hash by:"
echo "1. Using the application's API after one admin exists"
echo "2. Using an online Argon2 hash generator"
echo "3. Adding a CLI command to the application"
echo ""
echo "For now, you can use this temporary method:"
echo "Run: cargo run --example hash_password '$PASSWORD'"
