#!/bin/bash

# This script automates the PostgreSQL database setup for the orc-rust-ator project.
# It assumes PostgreSQL is already installed and running on your system.
#
# IMPORTANT: Hardcoding passwords in scripts is not recommended for production environments.
# Consider using environment variables or a secure secrets management solution.

DB_NAME="orc_rust_ator_db"
DB_USER="postgres"
DB_PASSWORD="password" # Change this to a strong, unique password in production!

echo "Starting PostgreSQL database setup..."

# Check if psql is available
if ! command -v psql &> /dev/null
then
    echo "Error: psql command not found. Please ensure PostgreSQL is installed and in your PATH."
    exit 1
fi

# Create database and user
# Using PGPASSWORD for non-interactive password input.
# This assumes the 'postgres' superuser can connect without a password initially,
# or that the user running the script has appropriate permissions.
# If you have a password set for the 'postgres' superuser, you might need to
# modify this to include PGPASSWORD for the initial connection as well.

echo "Creating database '$DB_NAME' and user '$DB_USER'..."

# Create database
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME;" &> /dev/null
if [ $? -ne 0 ]; then
    echo "Warning: Database '$DB_NAME' might already exist or there was an issue creating it. Continuing..."
fi

# Create user
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" &> /dev/null
if [ $? -ne 0 ]; then
    echo "Warning: User '$DB_USER' might already exist or there was an issue creating it. Continuing..."
fi

# Grant privileges
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"
if [ $? -ne 0 ]; then
    echo "Error: Failed to grant privileges on database '$DB_NAME' to user '$DB_USER'."
    exit 1
fi

echo "PostgreSQL database setup complete for '$DB_NAME'."
echo "You can now set your DATABASE_URL environment variable:"
echo "export DATABASE_URL=\"postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME\""
