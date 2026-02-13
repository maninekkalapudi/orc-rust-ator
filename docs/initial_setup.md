# Initial Setup Guide

This document provides instructions on how to set up and run the `orc-rust-ator` project for the first time.

## 1. Prerequisites

Before you begin, ensure you have the following software installed on your system:

- **Rust:** The programming language used for this project. You can install it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
- **Docker:** Used to run the PostgreSQL database in a container. You can download it from [https://www.docker.com/products/docker-desktop](https://www.docker.com/products/docker-desktop).
- **PostgreSQL Client:** Required for connecting to the database. You can download it from [https://www.postgresql.org/download/](https://www.postgresql.org/download/).

## 2. Configuration

1.  **Create a `.env` file:** In the root of the project, create a file named `.env`. This file will store the environment variables for the application.

2.  **Set the `DATABASE_URL`:** Add the following line to your `.env` file:

    ```
    DATABASE_URL=postgres://user:password@localhost:5432/orc-rust-ator
    ```

    Replace `user` and `password` with the desired credentials for your PostgreSQL database.

## 3. Database Setup

1.  **Start the PostgreSQL database:** You can use the provided `docker-compose.yaml` file to start a PostgreSQL container. Open a terminal in the root of the project and run the following command:

    ```bash
    docker-compose up -d
    ```

2.  **Run database migrations:** The project uses `sqlx` for database migrations. You will need to have `sqlx-cli` installed. If you don't have it, you can install it with:

    ```bash
    cargo install sqlx-cli
    ```

    Once `sqlx-cli` is installed, run the following command to apply the database migrations:

    ```bash
    sqlx database create
    sqlx migrate run
    ```

## 4. Running the Application

1.  **Build and run in development mode:** To run the application in development mode, use the following command:

    ```bash
    cargo run
    ```

2.  **Build and run in release mode:** For a production build, use the `--release` flag:

    ```bash
    cargo run --release
    ```

    The application will be available at `http://localhost:8080`.

## 5. Testing

To run the test suite, use the following command:

```bash
cargo test
```

## 6. Database Schema

This section provides a detailed explanation of the database schema used in the `orc-rust-ator` project. The schema is defined through a series of SQL migration files located in the `migrations` directory.

### `job_definitions`

This table stores the definitions of the jobs that the orchestrator can run.

| Column | Data Type | Constraints | Description |
|---|---|---|---|
| `job_id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() | The unique identifier for the job. |
| `job_name` | VARCHAR(255) | NOT NULL | The name of the job. |
| `description` | TEXT | | A description of the job. |
| `schedule` | VARCHAR(255) | NOT NULL | A cron string that defines when the job should run. |
| `is_active` | BOOLEAN | NOT NULL, DEFAULT true | A flag to indicate if the job is active and should be scheduled. |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the job was created. |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the job was last updated. |

**Indexes:**
- `idx_job_definitions_is_active`: An index on the `is_active` column to quickly find active jobs.

### `task_definitions`

This table stores the definitions of the tasks that make up a job. Each job can have one or more tasks.

| Column | Data Type | Constraints | Description |
|---|---|---|---|
| `task_id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() | The unique identifier for the task. |
| `job_id` | UUID | NOT NULL, REFERENCES job_definitions(job_id) ON DELETE CASCADE | The foreign key to the `job_definitions` table, linking the task to a job. |
| `task_order` | INT | NOT NULL | The order in which the task should be executed within the job. |
| `extractor_config` | JSONB | NOT NULL | A JSON object that contains the configuration for the extractor plugin. |
| `loader_config` | JSONB | NOT NULL | A JSON object that contains the configuration for the loader plugin. |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the task was created. |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the task was last updated. |

**Constraints:**
- `UNIQUE (job_id, task_order)`: Ensures that the task order is unique for each job.

**Indexes:**
- `idx_task_definitions_job_id`: An index on the `job_id` column to quickly find all the tasks for a given job.

### `job_runs`

This table stores the history of the job runs.

| Column | Data Type | Constraints | Description |
|---|---|---|---|
| `run_id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() | The unique identifier for the job run. |
| `job_id` | UUID | NOT NULL, REFERENCES job_definitions(job_id) ON DELETE CASCADE | The foreign key to the `job_definitions` table, linking the run to a job. |
| `status` | VARCHAR(50) | NOT NULL | The status of the job run (e.g., 'queued', 'running', 'success', 'failed'). |
| `triggered_by` | VARCHAR(50) | NOT NULL | How the job was triggered (e.g., 'scheduled', 'manual'). |
| `started_at` | TIMESTAMPTZ | | The timestamp when the job run started. |
| `finished_at` | TIMESTAMPTZ | | The timestamp when the job run finished. |
| `error_message` | TEXT | | Any error message if the job run failed. |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the job run was created. |

**Indexes:**
- `idx_job_runs_status`: An index on the `status` column to quickly find job runs with a specific status.
- `idx_job_runs_job_id`: An index on the `job_id` column to quickly find all the runs for a given job.

### `users`

This table stores the user accounts for authentication.

| Column | Data Type | Constraints | Description |
|---|---|---|---|
| `user_id` | UUID | PRIMARY KEY, DEFAULT uuid_generate_v4() | The unique identifier for the user. |
| `username` | VARCHAR(255) | NOT NULL, UNIQUE | The username of the user. |
| `password_hash` | TEXT | NOT NULL | The hashed password of the user. |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the user was created. |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | The timestamp when the user was last updated. |

**Indexes:**
- `idx_users_username`: An index on the `username` column to quickly find a user by their username.