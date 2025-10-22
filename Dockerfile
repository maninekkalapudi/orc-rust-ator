# Stage 1: Install cargo-chef and cache dependencies
FROM rust:slim-trixie as chef
WORKDIR /app

# Install build dependencies for chef stage
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ && rm -rf /var/lib/apt/lists/*

# Install cargo-chef
RUN cargo install cargo-chef --locked

# Copy Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to allow cargo-chef to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Cache dependencies
RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 2: Build the application
FROM rust:slim-trixie as builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ && rm -rf /var/lib/apt/lists/*

# Copy cached dependencies from the chef stage
COPY --from=chef /app/recipe.json recipe.json
COPY --from=chef /usr/local/cargo /usr/local/cargo
COPY --from=chef /app/target target
COPY migrations ./migrations

# Copy project files
COPY . .

# Build the application in release mode
RUN cargo build --release --bin orc-rust-ator

# Stage 3: Final runtime image
FROM debian:trixie-slim

# Install runtime dependencies for the application
RUN apt-get update && apt-get install -y \
    libpq5 \
    libzstd1 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage (release version)
COPY --from=builder /app/target/release/orc-rust-ator ./orc-rust-ator
# Copy necessary runtime assets
COPY migrations ./migrations
COPY jobs.yaml .
COPY test_data.csv .
COPY docker-entrypoint.sh .

EXPOSE 8080

# Create a non-root user for security
RUN useradd -m -u 1000 appuser
USER appuser

# Entrypoint will be handled by a separate script
ENTRYPOINT ["./docker-entrypoint.sh"]
