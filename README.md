# orc-rust-ator
Rust based ELT job orchestrator for high performance data processing with Rust tooling under the hood

# Rust Tools
  - **[tokio](https://github.com/tokio-rs/tokio)** - Core Async Runtime
  - **[polars](https://github.com/pola-rs/polars)** - Rust based DataFrame interface on top of an OLAP Query Engine implemented in Rust using Apache Arrow Columnar Format as the memory mode
  - **[reqwest](https://docs.rs/reqwest/latest/reqwest/)** - Rust based HTTP Client
  - **[anyhow](https://github.com/dtolnay/anyhow)** and **[thiserror](https://github.com/dtolnay/thiserror)** - Error handling in Rust

# Data Warehouse
  - **[duckdb](https://duckdb.org/)** - Open-source in-process column-oriented Relational Database Management System. Duckdb has first class Rust client

# Project Structure

    orc-rust-ator/
    ├── Cargo.toml         # Project dependencies and metadata
    ├── config.yaml        # Pipeline definitions
    ├── Dockerfile         # For containerizing the application
    └── src/
        ├── main.rs          # Main application entrypoint, orchestrator
        ├── config.rs        # Logic for loading and parsing config.yaml
        ├── pipeline.rs      # Defines the core traits (Extractor, Loader)
        ├── extractors/
        │   ├── mod.rs       # Makes the directory a module
        │   ├── csv_extractor.rs
        │   ├── api_extractor.rs
        │   └── db_extractor.rs  # (We'll sketch this one out)
        └── loaders/
            ├── mod.rs
            └── duckdb_loader.rs