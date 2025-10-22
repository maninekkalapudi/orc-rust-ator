#!/bin/sh

# Exit immediately if a command exits with a non-zero status.
set -e

# Wait for PostgreSQL to be ready
# This uses a simple loop to check if the database is accepting connections.
# In a production environment, you might use a more robust solution like wait-for-it.sh or similar.
until pg_isready -h $DB_HOST -p $DB_PORT -U $DB_USER
do
  echo "Waiting for PostgreSQL at $DB_HOST:$DB_PORT..."
  sleep 2
done

echo "PostgreSQL is up - starting application"

# Execute the command passed to the entrypoint
exec "$@"
