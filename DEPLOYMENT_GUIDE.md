# Deployment Guide for orc-rust-ator

This guide provides instructions for deploying and running the `orc-rust-ator` application using Docker.

## Prerequisites

*   **Docker:** Ensure Docker is installed and running on your system.
*   **Docker Compose (Recommended for local/dev):** Install Docker Compose for easier multi-service management.

## Building the Docker Image

First, navigate to the root of the `orc-rust-ator` project and build the Docker image. This process will compile the Rust application and create a lightweight Docker image.

```bash
docker build -t orc-rust-ator:latest .
```

## Running with Docker

You can run the `orc-rust-ator` container directly using `docker run`. Remember to configure the `DATABASE_URL` environment variable and map the API port.

### Using SQLite (for simplicity or local development)

This example uses an SQLite database file within the container. For persistent storage, you should mount a volume.

```bash
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL="sqlite:/app/data_warehouse.db" \
  --name orc-rust-ator-sqlite \
  orc-rust-ator:latest
```

To persist the SQLite database, mount a volume:

```bash
mkdir -p ./data
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/data:/app \
  -e DATABASE_URL="sqlite:/app/data_warehouse.db" \
  --name orc-rust-ator-sqlite-persistent \
  orc-rust-ator:latest
```

### Using PostgreSQL

For production or more robust setups, it's recommended to use an external PostgreSQL database. Ensure your PostgreSQL instance is accessible from where the Docker container will run.

```bash
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL="postgres://user:password@host:port/database_name" \
  --name orc-rust-ator-postgres \
  orc-rust-ator:latest
```

Replace `user`, `password`, `host`, `port`, and `database_name` with your PostgreSQL connection details.

## Running with Docker Compose (Recommended for Development)

Docker Compose allows you to define and run multi-container Docker applications. A `docker-compose.yaml` file can simplify the setup, especially when running with a local PostgreSQL instance.

Create a `docker-compose.yaml` file in your project root:

```yaml
version: '3.8'

services:
  db:
    image: postgres:16-alpine
    restart: always
    environment:
      POSTGRES_DB: orc_rust_ator_db
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - db_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  orc-rust-ator:
    build:
      context: .
      dockerfile: Dockerfile
    restart: always
    environment:
      DATABASE_URL: postgres://user:password@db:5432/orc_rust_ator_db
    ports:
      - "8080:8080"
    depends_on:
      - db

volumes:
  db_data:
```

To start the services:

```bash
docker compose up -d
```

To stop the services:

```bash
docker compose down
```

## Environment Variables

*   `DATABASE_URL`: (Required) The connection string for your database. Examples:
    *   `sqlite:/app/data_warehouse.db`
    *   `postgres://user:password@host:port/database_name`

## Monitoring and Logs

To view the logs of a running container:

```bash
docker logs <container_name_or_id>
```

If using Docker Compose:

```bash
docker compose logs orc-rust-ator
```

## Production Considerations

For production deployments, consider the following:

*   **Persistent Storage:** Ensure your database (especially SQLite) uses persistent volumes.
*   **Secrets Management:** Do not hardcode database credentials. Use Docker secrets or a dedicated secrets management solution.
*   **Scalability:** For high availability and scalability, consider deploying `orc-rust-ator` in an orchestration platform like Kubernetes.
*   **Monitoring:** Integrate with a robust monitoring system to track application performance and health.
*   **Backups:** Implement a strategy for regular database backups.
