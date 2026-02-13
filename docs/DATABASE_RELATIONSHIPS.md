# Database Relationships

This document describes the relationships between the tables in the `orc-rust-ator` database and includes a diagram to visualize these relationships.

## Relationships

-   A `job_definition` can have multiple `task_definitions`. This is a one-to-many relationship, with the `job_id` in the `task_definitions` table being a foreign key that references the `job_id` in the `job_definitions` table.
-   A `job_definition` can have multiple `job_runs`. This is a one-to-many relationship, with the `job_id` in the `job_runs` table being a foreign key that references the `job_id` in the `job_definitions` table.
-   The `users` table is not directly related to any of the other tables. It is used for API authentication and is managed independently.

## Diagram

The following diagram illustrates the relationships between the tables:

```mermaid
erDiagram
    job_definitions ||--o{ task_definitions : "has"
    job_definitions ||--o{ job_runs : "has"

    job_definitions {
        UUID job_id PK
        VARCHAR(255) job_name
        TEXT description
        VARCHAR(255) schedule
        BOOLEAN is_active
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
    }

    task_definitions {
        UUID task_id PK
        UUID job_id FK
        INT task_order
        JSONB extractor_config
        JSONB loader_config
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
    }

    job_runs {
        UUID run_id PK
        UUID job_id FK
        VARCHAR(50) status
        VARCHAR(50) triggered_by
        TIMESTAMPTZ started_at
        TIMESTAMPTZ finished_at
        TEXT error_message
        TIMESTAMPTZ created_at
    }

    users {
        UUID user_id PK
        VARCHAR(255) username
        TEXT password_hash
        TIMESTAMPTZ created_at
        TIMESTAMPTZ updated_at
    }
```
