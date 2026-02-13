# This script automates the PostgreSQL database setup for the orc-rust-ator project.
# It assumes PostgreSQL is already installed and running on your system.
#
# IMPORTANT: Hardcoding passwords in scripts is not recommended for production environments.
# Consider using environment variables or a secure secrets management solution.

$DB_NAME = "orc_rust_ator_db"
$DB_USER = "postgres"
$DB_PASSWORD = "password" # Change this to a strong, unique password in production!

Write-Host "Starting PostgreSQL database setup..."

# Check if psql is available
# This assumes psql is in the system's PATH or accessible.
# If not, you might need to provide the full path to psql.exe
$psqlPath = (Get-Command psql -ErrorAction SilentlyContinue).Source
if (-not $psqlPath) {
    Write-Host "Error: psql command not found. Please ensure PostgreSQL is installed and in your PATH."
    Exit 1
}

Write-Host "Creating database '$DB_NAME' and user '$DB_USER'..."

# Create database
# Using -c for command execution and -U for user.
# This assumes the 'postgres' superuser can connect.
try {
    & $psqlPath -U postgres -c "CREATE DATABASE $DB_NAME;" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: Database '$DB_NAME' might already exist or there was an issue creating it. Continuing..."
    }
} catch {
    Write-Host "Warning: Database '$DB_NAME' might already exist or there was an issue creating it. Continuing..."
}

# Create user
try {
    & $psqlPath -U postgres -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Warning: User '$DB_USER' might already exist or there was an issue creating it. Continuing..."
    }
} catch {
    Write-Host "Warning: User '$DB_USER' might already exist or there was an issue creating it. Continuing..."
}

# Grant privileges
try {
    & $psqlPath -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to grant privileges on database '$DB_NAME' to user '$DB_USER'."
        Exit 1
    }
} catch {
    Write-Host "Error: Failed to grant privileges on database '$DB_NAME' to user '$DB_USER'."
    Exit 1
}

Write-Host "PostgreSQL database setup complete for '$DB_NAME'."
Write-Host "You can now set your DATABASE_URL environment variable:"
Write-Host "`$env:DATABASE_URL=`"postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME`"`"
