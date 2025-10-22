# Project Plan

This document outlines the tasks for the re-architecture of the orc-rust-ator project.

## Phase 2: Implementation

- **2.1. Set up the Project Structure:**
    - [x] Create a new directory structure that reflects the new architecture (e.g., `src/api`, `src/orchestrator`, `src/worker`, `src/state`, `src/plugins`).
    - [x] Update `Cargo.toml` with the necessary dependencies for the new architecture (e.g., a database driver for PostgreSQL, a web framework for the API).

- **2.2. Implement the State Store:**
    - [x] Define the database schema for the `job_definitions`, `job_runs`, and `tasks` tables.
    - [x] Implement the data access layer for the `State Store`, providing functions for creating, reading, updating, and deleting records in these tables.

- **2.3. Implement the Core Components:**
    - [x] Implement the `Job Manager` with CRUD functionality for jobs and tasks.
    - [x] Implement the `Scheduler` to queue due jobs.
    - [x] Implement the `Worker Manager` to launch and manage workers.

- **2.4. Implement the Worker:**
    - [x] Implement the `Task Runner` to execute the tasks of a job.
    - [x] Refactor the existing `Extractor` and `Loader` implementations into a pluggable architecture.
    - [x] Implement a robust error handling and retry mechanism for tasks.

- **2.5. Implement the API/CLI:**
    - [x] Design and implement a REST API for managing and monitoring the system.
    - [x] Update the CLI to interact with the new API.

## Phase 3: Testing and Deployment

- **3.1. Testing:**
    - [x] Write unit tests for all new components.
    - [x] Write integration tests to ensure that the different components work together correctly.
    - [x] Perform end-to-end testing of the entire system.

- **3.2. Documentation:**
    - [x] Update the `README.md` to reflect the new architecture.
    - [x] Add documentation for the API.

- **3.3. Deployment:**
    - [x] Create a `Dockerfile` for the new application.
    - [x] Write a deployment script or guide.
