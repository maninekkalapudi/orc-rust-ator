# orc-rust-ator

A robust and scalable Rust-based ELT (Extract, Load, Transform) job orchestrator designed for high-performance data processing. `orc-rust-ator` provides a flexible architecture for defining, scheduling, executing, and monitoring data pipelines.

## Overview

`orc-rust-ator` enables users to define data extraction and loading tasks as "jobs," which can be scheduled or triggered manually. It leverages a pluggable architecture for various data sources (extractors) and destinations (loaders), ensuring adaptability to diverse data ecosystems. The system is managed via a RESTful API, with an internal scheduler and worker manager handling job execution.

## Key Features

*   **Pluggable Extractors:** Easily integrate with different data sources (e.g., API, CSV, Parquet).
*   **Pluggable Loaders:** Support various data destinations (e.g., DuckDB).
*   **Flexible Job Scheduling:** Define jobs with cron-like schedules or trigger them manually via the API.
*   **Robust Orchestration:** Dedicated components for managing jobs, scheduling runs, and executing tasks.
*   **RESTful API:** Programmatic control and monitoring of jobs and their execution.
*   **Database-Backed State:** Persistent storage for job definitions, task configurations, and run history.
*   **Error Handling & Retries:** Built-in mechanisms for resilient task execution.

## Architecture

The `orc-rust-ator` system is composed of several key components:

*   **API (`src/api`):** Provides a RESTful interface for interacting with the orchestrator, allowing users to create, manage, and monitor jobs and their runs.
*   **Orchestrator (`src/orchestrator`):**
    *   **Job Manager:** Handles CRUD operations for job definitions and their associated tasks.
    *   **Scheduler:** Periodically checks for due jobs based on their schedules and queues them for execution.
    *   **Worker Manager:** Dispatches queued job runs to available workers and monitors their progress.
*   **Worker (`src/worker`):** Executes individual job tasks, coordinating the extraction and loading of data.
*   **State Store (`src/state`):** Manages the application's persistent state, including job definitions, task configurations, and job run history, using a relational database (PostgreSQL/SQLite).
*   **Plugins (`src/plugins`):**
    *   **Extractors:** Modules responsible for fetching data from various sources.
    *   **Loaders:** Modules responsible for writing data to various destinations.

## Getting Started

### Prerequisites

*   **Rust:** Install Rust and Cargo using `rustup`.
*   **Database:** PostgreSQL or SQLite. The application can be configured to use either. For development, SQLite is often convenient.

### Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-repo/orc-rust-ator.git
    cd orc-rust-ator
    ```
2.  **Database Migrations:**
    Ensure your database is running and accessible.
    For PostgreSQL, set the `DATABASE_URL` environment variable (e.g., `postgres://user:password@host:port/database_name`).
    For SQLite, you can use a file path (e.g., `sqlite:data_warehouse.db`).
    The application will run migrations on startup.

3.  **Build the project:**
    ```bash
    cargo build --release
    ```

### Running the Application

`orc-rust-ator` runs as a single application that can start its various components.

```bash
# Example: Run the application (this will start API, Scheduler, and WorkerManager)
# Ensure DATABASE_URL is set in your environment
DATABASE_URL="sqlite:data_warehouse.db" cargo run --release
```

## API Reference (Examples)

The API server typically runs on `http://127.0.0.1:8080`.

### Health Check

```bash
curl http://127.0.0.1:8080/health
```

### Create a New Job

```bash
curl -X POST -H "Content-Type: application/json" -d '{
    "job_name": "My Daily Report",
    "description": "Extracts sales data and loads into DuckDB",
    "schedule": "0 0 9 * * *",
    "is_active": true,
    "tasks": [
        {
            "extractor_config": {
                "type": "api",
                "url": "https://api.example.com/sales"
            },
            "loader_config": {
                "type": "duckdb",
                "db_path": "data_warehouse.db",
                "table_name": "daily_sales"
            }
        }
    ]
}' http://127.0.0.1:8080/jobs
```

### Manually Trigger a Job

Replace `{job_id}` with the actual ID of the job.

```bash
curl -X POST http://127.0.0.1:8080/jobs/{job_id}/run
```

### Get All Jobs

```bash
curl http://127.0.0.1:8080/jobs
```

### Get All Job Runs

```bash
curl http://127.0.0.1:8080/runs
```

## Extensibility

New data sources and destinations can be added by implementing the `Extractor` and `Loader` traits respectively within the `src/plugins` directory.

## Development

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
