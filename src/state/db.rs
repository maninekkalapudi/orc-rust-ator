//! Provides database access and defines data models for the application.
//! 
//! This module handles database connection pooling, migrations, and CRUD operations
//! for `JobDefinition`, `TaskDefinition`, and `JobRun` entities.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

// --- Data Structures ---

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct JobDefinition {
    pub job_id: String,
    pub job_name: String,
    pub description: Option<String>,
    pub schedule: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct TaskDefinition {
    pub task_id: String,
    pub job_id: String,
    pub task_order: i32,
    pub extractor_config: Value,
    pub loader_config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct JobRun {
    pub run_id: String,
    pub job_id: String,
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
}

#[derive(Clone)]
pub struct Db {
    pool: DbPool,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .context("Failed to create PostgreSQL connection pool ")?;
        Ok(Self { pool: DbPool::Pg(pool) })
    }

    pub async fn migrate(&self) -> Result<()> {
        match &self.pool {
            DbPool::Pg(pool) => {
                sqlx::migrate!("./migrations").run(pool).await?;
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
        let job = sqlx::query_as::<_, JobDefinition>(
            "INSERT INTO job_definitions (job_name, description, schedule, is_active) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(job_name)
        .bind(description)
        .bind(schedule)
        .bind(is_active)
        .fetch_one(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(job)
    }

    pub async fn get_job_definition(&self, job_id: String) -> Result<Option<JobDefinition>> {
        let job = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(match &self.pool { DbPool::Pg(pool) => pool })
            .await?;
        Ok(job)
    }

    pub async fn get_all_job_definitions(&self) -> Result<Vec<JobDefinition>> {
        let jobs = sqlx::query_as::<_, JobDefinition>("SELECT * FROM job_definitions ")
            .fetch_all(match &self.pool { DbPool::Pg(pool) => pool })
            .await?;
        Ok(jobs)
    }

    // --- Task Definitions ---

    pub async fn create_task_definition(
        &self,
        job_id: String,
        task_order: i32,
        extractor_config: &Value,
        loader_config: &Value,
    ) -> Result<TaskDefinition> {
        let task = sqlx::query_as::<_, TaskDefinition>(
            "INSERT INTO task_definitions (job_id, task_order, extractor_config, loader_config) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(job_id)
        .bind(task_order)
        .bind(extractor_config)
        .bind(loader_config)
        .fetch_one(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(task)
    }

    pub async fn get_task_definitions_for_job(&self, job_id: String) -> Result<Vec<TaskDefinition>> {
        let tasks = sqlx::query_as::<_, TaskDefinition>(
            "SELECT * FROM task_definitions WHERE job_id = $1 ORDER BY task_order ASC "
        )
        .bind(job_id)
        .fetch_all(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(tasks)
    }

    // --- Job Runs ---

    pub async fn create_job_run(
        &self,
        job_id: String,
        status: &str,
        triggered_by: &str,
    ) -> Result<JobRun> {
        let run = sqlx::query_as::<_, JobRun>(
            "INSERT INTO job_runs (job_id, status, triggered_by) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(job_id)
        .bind(status)
        .bind(triggered_by)
        .fetch_one(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(run)
    }

    pub async fn update_job_run_status(&self, run_id: String, status: &str) -> Result<()> {
        sqlx::query("UPDATE job_runs SET status = $1, updated_at = NOW() WHERE run_id = $2")
            .bind(status)
            .bind(run_id)
            .execute(match &self.pool { DbPool::Pg(pool) => pool })
            .await?;
        Ok(())
    }

    pub async fn get_queued_job_run(&self) -> Result<Option<JobRun>> {
        let run = sqlx::query_as::<_, JobRun>(
            "SELECT * FROM job_runs WHERE status = \"queued\" ORDER BY created_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED "
        )
        .fetch_optional(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(run)
    }

    pub async fn get_last_job_run(&self, job_id: String) -> Result<Option<JobRun>> {
        let run = sqlx::query_as::<_, JobRun>(
            "SELECT * FROM job_runs WHERE job_id = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(job_id)
        .fetch_optional(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(run)
    }

    pub async fn update_job_run_status_with_error(
        &self,
        run_id: String,
        status: &str,
        error_message: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE job_runs SET status = $1, error_message = $2, finished_at = NOW() WHERE run_id = $3",
        )
        .bind(status)
        .bind(error_message)
        .bind(run_id)
        .execute(match &self.pool { DbPool::Pg(pool) => pool })
        .await?;
        Ok(())
    }

    pub async fn get_all_job_runs(&self) -> Result<Vec<JobRun>> {
        let runs = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs ")
            .fetch_all(match &self.pool { DbPool::Pg(pool) => pool })
            .await?;
        Ok(runs)
    }

    pub async fn get_job_run(&self, run_id: String) -> Result<Option<JobRun>> {
        let run = sqlx::query_as::<_, JobRun>("SELECT * FROM job_runs WHERE run_id = $1")
            .bind(run_id)
            .fetch_optional(match &self.pool { DbPool::Pg(pool) => pool })
            .await?;
        Ok(run)
    }
}
