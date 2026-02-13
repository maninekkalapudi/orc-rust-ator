# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https.keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https.semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **2025-08-07:** Added support for extracting data from Parquet files.
- **2025-10-20:** Completed end-to-end testing of the system, verifying the full job lifecycle from API creation to successful data loading.
- **2025-10-20:** Implemented initial integration test (`test_create_and_run_job_lifecycle`) covering job creation, execution, and status verification.
- **2025-10-20:** Resolved complex dependency conflicts between `polars`, `duckdb`, and `arrow` crates by pinning `chrono` to `=0.4.39` and implementing a CSV-to-tempfile-to-DuckDB loading strategy in `DuckDBLoader`.
- **2025-10-20:** Cleaned up `DuckDBLoader` and `WorkerManager` debugging code and reverted temporary test-specific changes.
- **2025-10-20:** Initial `CHANGELOG.md` to document the re-architecture of the project.
- **2025-10-20:** Began Phase 1: Discovery and High-Level Design.
- **2025-10-20:** Completed analysis of the existing codebase (`README.md`, `Cargo.toml`, `src/main.rs`, `jobs.yaml`).
- **2025-10-20:** Defined the target architecture, focusing on decoupling, scalability, and extensibility.

### Removed
- **2025-08-07:** Removed temporary data warehouse files from the repository.
