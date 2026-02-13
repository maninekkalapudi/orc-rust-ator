/*
 * File: src/state/db.rs
 * Description: Provides database access and defines data models for the application.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added SQLite support for in-memory testing.
 * - 2026-02-13: Normalized UUID generation in Rust for cross-db compatibility.
 * - 2026-02-13: Added file header and documentation comments.
 */

//! Provides database access and defines data models for the application.
//! 
//! This module handles database connection pooling, migrations, and CRUD operations
//! for `JobDefinition`, `TaskDefinition`, and `JobRun` entities.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use serde::Serialize;
use sqlx::{FromRow, PgPool, SqlitePool};
use uuid::Uuid;

// --- Data Structures ---

use crate::models::user::User;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct JobDefinition {
    pub job_id: Uuid,
    pub job_name: String,
    pub description: Option<String>,
    pub schedule: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct TaskDefinition {
    pub task_id: Uuid,
    pub job_id: Uuid,
    pub task_order: i32,
    pub extractor_config: Value,
    pub loader_config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct JobRun {
    pub run_id: Uuid,
    pub job_id: Uuid,
    pub status: String,
    pub triggered_by: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Database Connection ---

#[derive(Clone)]
pub enum DbPool {
    Pg(PgPool),
    Sqlite(SqlitePool),
}

/// Database interface providing access to the underlying storage (Postgres or SQLite).
#[derive(Clone)]
pub struct Db {
    pool: DbPool,
}

impl Db {
    /// Creates a new `Db` instance from a connection URL.
    /// Supports `postgresql://` and `sqlite://` protocols.
    pub async fn new(database_url: &str) -> Result<Self> {
        if database_url.starts_with("sqlite") {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
                .context("Failed to create SQLite connection pool")?;
            Ok(Self { pool: DbPool::Sqlite(pool) })
        } else {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
                .context("Failed to create PostgreSQL connection pool")?;
            Ok(Self { pool: DbPool::Pg(pool) })
        }
    }

    pub async fn migrate(&self) -> Result<()> {
        match &self.pool {
            DbPool::Pg(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
            }
            DbPool::Sqlite(pool) => {
                let schema = r#"
CREATE TABLE IF NOT EXISTS job_definitions (
    job_id TEXT PRIMARY KEY,
    job_name TEXT NOT NULL,
    description TEXT,
    schedule TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS task_definitions (
    task_id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES job_definitions(job_id) ON DELETE CASCADE,
    task_order INT NOT NULL,
    extractor_config TEXT NOT NULL,
    loader_config TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (job_id, task_order)
);

CREATE TABLE IF NOT EXISTS job_runs (
    run_id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES job_definitions(job_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    triggered_by TEXT NOT NULL,
    started_at DATETIME,
    finished_at DATETIME,
    error_message TEXT,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS job_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES job_definitions(job_id),
    warehouse_table TEXT NOT NULL,
    file_path TEXT NOT NULL,
    row_count BIGINT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_job FOREIGN KEY (job_id) REFERENCES job_definitions(job_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_job_definitions_is_active ON job_definitions(is_active);
CREATE INDEX IF NOT EXISTS idx_job_runs_status ON job_runs(status);
CREATE INDEX IF NOT EXISTS idx_job_runs_job_id ON job_runs(job_id);
CREATE INDEX IF NOT EXISTS idx_task_definitions_job_id ON task_definitions(job_id);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_job_results_job_id ON job_results(job_id);
CREATE INDEX IF NOT EXISTS idx_job_results_created_at ON job_results(created_at);
"#;
                for statement in schema.split(';') {
                    let trimmed = statement.trim();
                    if !trimmed.is_empty() {
                        sqlx::query(trimmed).execute(pool).await?;
                    }
                }
            }
        }
        Ok(())
    }

    // --- Job Definitions ---

    pub async fn create_job_definition(
        &self,
        job_name: &str,
        description: Option<&str>,
        schedule: &str,
        is_active: bool,
    ) -> Result<JobDefinition> {
        let job_id = Uuid::new_v4();
        match &self.pool {
            DbPool::Pg(pool) => {
                let job = sqlx::query_as::<_, JobDefinition>(
                    "INSERT INTO job_definitions (job_id, job_name, description, schedule, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING *"
                )
                .bind(job_id)
                .bind(job_name)
                .bind(description)
                .bind(schedule)
                .bind(is_active)
                .fetch_one(pool)
                .await?;
                Ok(job)
            }
            DbPool::Sqlite(pool) => {
                let job = sqlx::query_as::<_, JobDefinition>(
                    "INSERT INTO job_definitions (job_id, job_name, description, schedule, is_active) VALUES (?, ?, ?, ?, ?) RETURNING *"
                )
                .bind(job_id)
                .bind(job_name)
                .bind(description)
                .bind(schedule)
                .bind(is_active)
                .fetch_one(pool)
                .await?;
                Ok(job)
            }
        }
    }

    pub async fn get_job_definition(&self, job_id: Uuid) -> Result<Option<JobDefinition>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let job = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions WHERE job_id = $1")
                    .bind(job_id)
                    .fetch_optional(pool)
                    .await?;
                Ok(job)
            }
            DbPool::Sqlite(pool) => {
                let job = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions WHERE job_id = ?")
                    .bind(job_id)
                    .fetch_optional(pool)
                    .await?;
                Ok(job)
            }
        }
    }

    pub async fn get_all_job_definitions(&self) -> Result<Vec<JobDefinition>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let jobs = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions ")
                    .fetch_all(pool)
                    .await?;
                Ok(jobs)
            }
            DbPool::Sqlite(pool) => {
                let jobs = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions ")
                    .fetch_all(pool)
                    .await?;
                Ok(jobs)
            }
        }
    }

    // --- Task Definitions ---

    pub async fn create_task_definition(
        &self,
        job_id: Uuid,
        task_order: i32,
        extractor_config: &Value,
        loader_config: &Value,
    ) -> Result<TaskDefinition> {
        let task_id = Uuid::new_v4();
        match &self.pool {
            DbPool::Pg(pool) => {
                let task = sqlx::query_as::<_, TaskDefinition>(
                    "INSERT INTO task_definitions (task_id, job_id, task_order, extractor_config, loader_config) VALUES ($1, $2, $3, $4, $5) RETURNING *"
                )
                .bind(task_id)
                .bind(job_id)
                .bind(task_order)
                .bind(extractor_config)
                .bind(loader_config)
                .fetch_one(pool)
                .await?;
                Ok(task)
            }
            DbPool::Sqlite(pool) => {
                let task = sqlx::query_as::<_, TaskDefinition>(
                    "INSERT INTO task_definitions (task_id, job_id, task_order, extractor_config, loader_config) VALUES (?, ?, ?, ?, ?) RETURNING *"
                )
                .bind(task_id)
                .bind(job_id)
                .bind(task_order)
                .bind(extractor_config)
                .bind(loader_config)
                .fetch_one(pool)
                .await?;
                Ok(task)
            }
        }
    }

    pub async fn get_task_definitions_for_job(&self, job_id: Uuid) -> Result<Vec<TaskDefinition>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let tasks = sqlx::query_as::<_, TaskDefinition>(
                    "SELECT * FROM task_definitions WHERE job_id = $1 ORDER BY task_order ASC "
                )
                .bind(job_id)
                .fetch_all(pool)
                .await?;
                Ok(tasks)
            }
            DbPool::Sqlite(pool) => {
                let tasks = sqlx::query_as::<_, TaskDefinition>(
                    "SELECT * FROM task_definitions WHERE job_id = ? ORDER BY task_order ASC "
                )
                .bind(job_id)
                .fetch_all(pool)
                .await?;
                Ok(tasks)
            }
        }
    }

    // --- Job Runs ---

    pub async fn create_job_run(
        &self,
        job_id: Uuid,
        status: &str,
        triggered_by: &str,
    ) -> Result<JobRun> {
        let run_id = Uuid::new_v4();
        match &self.pool {
            DbPool::Pg(pool) => {
                let run = sqlx::query_as::<_, JobRun>(
                    "INSERT INTO job_runs (run_id, job_id, status, triggered_by) VALUES ($1, $2, $3, $4) RETURNING *"
                )
                .bind(run_id)
                .bind(job_id)
                .bind(status)
                .bind(triggered_by)
                .fetch_one(pool)
                .await?;
                Ok(run)
            }
            DbPool::Sqlite(pool) => {
                let run = sqlx::query_as::<_, JobRun>(
                    "INSERT INTO job_runs (run_id, job_id, status, triggered_by) VALUES (?, ?, ?, ?) RETURNING *"
                )
                .bind(run_id)
                .bind(job_id)
                .bind(status)
                .bind(triggered_by)
                .fetch_one(pool)
                .await?;
                Ok(run)
            }
        }
    }

    pub async fn update_job_run_status(&self, run_id: Uuid, status: &str) -> Result<()> {
        match &self.pool {
            DbPool::Pg(pool) => {
                sqlx::query("UPDATE job_runs SET status = $1, updated_at = NOW() WHERE run_id = $2")
                    .bind(status)
                    .bind(run_id)
                    .execute(pool)
                    .await?;
            }
            DbPool::Sqlite(pool) => {
                sqlx::query("UPDATE job_runs SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE run_id = ?")
                    .bind(status)
                    .bind(run_id)
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_queued_job_run(&self) -> Result<Option<JobRun>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let run = sqlx::query_as::<_, JobRun>(
                    "SELECT * FROM job_runs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED "
                )
                .fetch_optional(pool)
                .await?;
                Ok(run)
            }
            DbPool::Sqlite(pool) => {
                // SQLite doesn't support FOR UPDATE SKIP LOCKED.
                // For test environments, a simple select is usually sufficient.
                let run = sqlx::query_as::<_, JobRun>(
                    "SELECT * FROM job_runs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1"
                )
                .fetch_optional(pool)
                .await?;
                Ok(run)
            }
        }
    }

    pub async fn get_last_job_run(&self, job_id: Uuid) -> Result<Option<JobRun>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let run = sqlx::query_as::<_, JobRun>(
                    "SELECT * FROM job_runs WHERE job_id = $1 ORDER BY created_at DESC LIMIT 1"
                )
                .bind(job_id)
                .fetch_optional(pool)
                .await?;
                Ok(run)
            }
            DbPool::Sqlite(pool) => {
                let run = sqlx::query_as::<_, JobRun>(
                    "SELECT * FROM job_runs WHERE job_id = ? ORDER BY created_at DESC LIMIT 1"
                )
                .bind(job_id)
                .fetch_optional(pool)
                .await?;
                Ok(run)
            }
        }
    }

    pub async fn update_job_run_status_with_error(
        &self,
        run_id: Uuid,
        status: &str,
        error_message: &str,
    ) -> Result<()> {
        match &self.pool {
            DbPool::Pg(pool) => {
                sqlx::query(
                    "UPDATE job_runs SET status = $1, error_message = $2, finished_at = NOW() WHERE run_id = $3",
                )
                .bind(status)
                .bind(error_message)
                .bind(run_id)
                .execute(pool)
                .await?;
            }
            DbPool::Sqlite(pool) => {
                sqlx::query(
                    "UPDATE job_runs SET status = ?, error_message = ?, finished_at = CURRENT_TIMESTAMP WHERE run_id = ?",
                )
                .bind(status)
                .bind(error_message)
                .bind(run_id)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }

    pub async fn get_all_job_runs(&self) -> Result<Vec<JobRun>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let runs = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs ")
                    .fetch_all(pool)
                    .await?;
                Ok(runs)
            }
            DbPool::Sqlite(pool) => {
                let runs = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs ")
                    .fetch_all(pool)
                    .await?;
                Ok(runs)
            }
        }
    }

    pub async fn get_job_run(&self, run_id: Uuid) -> Result<Option<JobRun>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let run = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs WHERE run_id = $1")
                    .bind(run_id)
                    .fetch_optional(pool)
                    .await?;
                Ok(run)
            }
            DbPool::Sqlite(pool) => {
                let run = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs WHERE run_id = ?")
                    .bind(run_id)
                    .fetch_optional(pool)
                    .await?;
                Ok(run)
            }
        }
    }

    // --- User Management ---

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        let user_id = Uuid::new_v4();
        match &self.pool {
            DbPool::Pg(pool) => {
                let user = sqlx::query_as::<_, User>(
                    "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3) RETURNING *"
                )
                .bind(user_id)
                .bind(username)
                .bind(password_hash)
                .fetch_one(pool)
                .await?;
                Ok(user)
            }
            DbPool::Sqlite(pool) => {
                let user = sqlx::query_as::<_, User>(
                    "INSERT INTO users (user_id, username, password_hash) VALUES (?, ?, ?) RETURNING *"
                )
                .bind(user_id)
                .bind(username)
                .bind(password_hash)
                .fetch_one(pool)
                .await?;
                Ok(user)
            }
        }
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        match &self.pool {
            DbPool::Pg(pool) => {
                let user = sqlx::query_as::<_, User>(
                    "SELECT * FROM users WHERE username = $1"
                )
                .bind(username)
                .fetch_optional(pool)
                .await?;
                Ok(user)
            }
            DbPool::Sqlite(pool) => {
                let user = sqlx::query_as::<_, User>(
                    "SELECT * FROM users WHERE username = ?"
                )
                .bind(username)
                .fetch_optional(pool)
                .await?;
                Ok(user)
            }
        }
    }
}

