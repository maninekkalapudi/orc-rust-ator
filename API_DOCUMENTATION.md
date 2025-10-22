# API Documentation for orc-rust-ator

This document provides a detailed reference for the `orc-rust-ator` RESTful API, allowing programmatic interaction with the job orchestration system.

## Base URL

The API server typically runs on `http://127.0.0.1:8080`.

## Authentication

Currently, the API does not implement any authentication mechanisms. All endpoints are publicly accessible.

## Error Handling

API errors are indicated by standard HTTP status codes. Common error responses include:

*   `400 Bad Request`: The request body or parameters are invalid.
*   `404 Not Found`: The requested resource does not exist.
*   `500 Internal Server Error`: An unexpected error occurred on the server.

## Endpoints

---

### 1. Health Check

Checks the health and availability of the API server.

*   **URL:** `/health`
*   **Method:** `GET`
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: The server is running and healthy.

**Example Request:**

```bash
curl http://127.0.0.1:8080/health
```

---

### 2. Create a New Job

Creates a new job definition with associated tasks.

*   **URL:** `/jobs`
*   **Method:** `POST`
*   **Request Body:** `application/json`
    ```json
    {
        "job_name": "string",
        "description": "string" | null,
        "schedule": "string", // Cron expression (e.g., "0 0 9 * * *") or "@manual"
        "is_active": boolean,
        "tasks": [
            {
                "extractor_config": { /* JSON object based on extractor type */ },
                "loader_config": { /* JSON object based on loader type */ }
            }
        ]
    }
    ```
    **Extractor Config Examples:**
    *   **API Extractor:** `{"type": "api", "url": "https://api.example.com/data"}`
    *   **CSV Extractor:** `{"type": "csv", "path": "/path/to/data.csv"}`
    *   **Parquet Extractor:** `{"type": "parquet", "path": "/path/to/data.parquet"}`

    **Loader Config Examples:**
    *   **DuckDB Loader:** `{"type": "duckdb", "db_path": "data.db", "table_name": "my_table"}`

*   **Responses:**
    *   `200 OK`: Job created successfully. Returns the created `JobDefinition` object.
    *   `500 Internal Server Error`: Failed to create job.

**Example Request:**

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

---

### 3. Get All Jobs

Retrieves a list of all defined jobs.

*   **URL:** `/jobs`
*   **Method:** `GET`
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: Returns an array of `JobDefinition` objects.
    *   `500 Internal Server Error`: Failed to retrieve jobs.

**Example Request:**

```bash
curl http://127.0.0.1:8080/jobs
```

---

### 4. Get a Specific Job

Retrieves details for a single job definition, including its associated tasks.

*   **URL:** `/jobs/{job_id}`
*   **Method:** `GET`
*   **URL Parameters:**
    *   `job_id` (string, UUID): The unique identifier of the job.
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: Returns a `JobDefinition` object along with its `TaskDefinition` array.
    *   `404 Not Found`: Job with the given `job_id` not found.
    *   `500 Internal Server Error`: Failed to retrieve job.

**Example Request:**

```bash
curl http://127.0.0.1:8080/jobs/a1b2c3d4-e5f6-7890-1234-567890abcdef
```

---

### 5. Manually Trigger a Job

Queues a specific job for immediate execution.

*   **URL:** `/jobs/{job_id}/run`
*   **Method:** `POST`
*   **URL Parameters:**
    *   `job_id` (string, UUID): The unique identifier of the job to trigger.
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: Job successfully queued for execution.
    *   `404 Not Found`: Job with the given `job_id` not found.
    *   `500 Internal Server Error`: Failed to queue job run.

**Example Request:**

```bash
curl -X POST http://127.0.0.1:8080/jobs/a1b2c3d4-e5f6-7890-1234-567890abcdef/run
```

---

### 6. Get All Job Runs

Retrieves a list of all historical job runs.

*   **URL:** `/runs`
*   **Method:** `GET`
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: Returns an array of `JobRun` objects.
    *   `500 Internal Server Error`: Failed to retrieve job runs.

**Example Request:**

```bash
curl http://127.0.0.1:8080/runs
```

---

### 7. Get a Specific Job Run

Retrieves details for a single job run.

*   **URL:** `/runs/{run_id}`
*   **Method:** `GET`
*   **URL Parameters:**
    *   `run_id` (string, UUID): The unique identifier of the job run.
*   **Request Body:** None
*   **Responses:**
    *   `200 OK`: Returns a `JobRun` object.
    *   `404 Not Found`: Job run with the given `run_id` not found.
    *   `500 Internal Server Error`: Failed to retrieve job run.

**Example Request:**

```bash
curl http://127.0.0.1:8080/runs/f1e2d3c4-b5a6-9876-5432-10fedcba9876
```
