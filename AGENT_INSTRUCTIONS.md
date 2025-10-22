# Agent Instructions for `orc-rust-ator` Project

This document outlines best practices and project-specific considerations for Dockerizing and building the `orc-rust-ator` Rust project, particularly focusing on optimizing compilation times and avoiding common pitfalls. Adhering to these guidelines will ensure efficient and successful builds.

## General Rust Docker Best Practices

1.  **Always Use Multi-Stage Builds:**
    *   Separate the build environment (with full Rust toolchain and build dependencies) from the runtime environment (containing only the compiled binary and essential runtime dependencies). This significantly reduces final image size and improves security.
    *   The `Dockerfile` is already configured with `chef`, `builder`, and `runtime` stages.

2.  **Leverage `cargo-chef` for Dependency Caching:**
    *   `cargo-chef` is crucial for optimizing Docker layer caching. It pre-compiles dependencies, ensuring that changes to application source code do not trigger a full recompilation of all dependencies.
    *   The `chef` stage in the `Dockerfile` is dedicated to this purpose.

3.  **Optimize Dockerfile Layer Caching Order:**
    *   Ensure `Cargo.toml` and `Cargo.lock` are copied early in the `chef` stage, followed by `cargo chef prepare` and `cargo chef cook`. This maximizes cache hits for dependencies.
    *   Application source code (`COPY . .`) should be copied *after* dependencies are cached.

4.  **Utilize `.dockerignore` Effectively:**
    *   Maintain a comprehensive `.dockerignore` file to exclude unnecessary files and directories (e.g., `.git/`, `target/`, local development assets) from the Docker build context. This speeds up `COPY` operations and reduces context size.

5.  **Consider `default-features=false` for Large Crates:**
    *   For large crates like `polars`, disabling default features and explicitly enabling only required features can drastically reduce compilation times and binary size.
    *   The `polars` dependency in `Cargo.toml` is configured with `default-features=false` and specific features (`lazy`, `csv`, `parquet`, `json`, `ipc`). If new `polars` functionality is required, consult the `polars` documentation to identify and enable the necessary features.

## Project-Specific Considerations

1.  **`chef` Stage Build Dependencies:**
    *   The `chef` stage *must* include `pkg-config`, `libssl-dev`, and `g++` in its `apt-get install` command. These are essential for compiling various Rust crates, especially those with C/C++ dependencies like `openssl-sys` and `libduckdb-sys`.
    *   Current `Dockerfile` line: `RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ && rm -rf /var/lib/apt/lists/*`

2.  **`builder` Stage Build Dependencies:**
    *   Similar to the `chef` stage, the `builder` stage also requires `pkg-config`, `libssl-dev`, and `g++` for the main application compilation.
    *   Current `Dockerfile` line: `RUN apt-get update && apt-get install -y pkg-config libssl-dev g++ && rm -rf /var/lib/apt/lists/*`

3.  **`sqlx::migrate!` Macro Usage:**
    *   The `migrations` directory *must* be copied into the `builder` stage *before* the `cargo build` command. The `sqlx::migrate!` macro requires access to these files at compile time.
    *   Ensure the path provided to `sqlx::migrate!` (e.g., `"./migrations"`) is correct and **does not contain any trailing spaces**. A trailing space can lead to a "No such file or directory" error during compilation.
    *   Current `Dockerfile` line: `COPY migrations ./migrations` in the `builder` stage.
    *   Current `src/state/db.rs` line: `sqlx::migrate!("./migrations").run(pool).await?;`

4.  **`docker-compose.yaml` Version Warning:**
    *   The `version` attribute in `docker-compose.yaml` is considered obsolete. While it doesn't cause errors, it's good practice to remove it to avoid potential confusion in future Docker Compose versions.

By following these instructions, future agents and developers can efficiently manage and build the `orc-rust-ator` project within Docker, avoiding common compilation issues and maintaining optimized build times.