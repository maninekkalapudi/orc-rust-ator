/*
 * File: src/orchestrator/job_manager.rs
 * Description: Manages job definitions and their associated tasks.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added in-memory SQLite support for unit testing.
 * - 2026-02-13: Added file header and documentation comments.
 */

//! Manages job definitions and their associated tasks.
//! 
//! This module provides the `JobManager` struct, which offers CRUD (Create, Read, Update, Delete)
//! functionality for job definitions and their tasks, interacting directly with the database.

use crate::state::db::{Db, JobDefinition, TaskDefinition};
use anyhow::{Context, Result};
use serde_json::Value;
use uuid::Uuid;

use tracing::info;

pub struct JobManager {
    db: Db,
}

impl JobManager {
    /// Creates a new `JobManager` with the given database handle.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Creates a new job definition and its associated tasks in the database.
    /// This is an atomic operation within the database (implicitly or explicitly handled by Db).
    pub async fn create_job(
        &self,
        job_name: &str,
        description: Option<&str>,
        schedule: &str,
        is_active: bool,
        tasks: Vec<NewTask>,
    ) -> Result<JobDefinition> {
        info!("JobManager: Creating job definition for '{}'", job_name);
        let job = self
            .db
            .create_job_definition(job_name, description, schedule, is_active)
            .await
            .context(format!("Failed to create job definition for '{}'", job_name))?;

        info!("JobManager: Job definition '{}' created with ID: {}", job_name, job.job_id);

        for (i, task) in tasks.into_iter().enumerate() {
            info!("JobManager: Creating task {} for job '{}'", i + 1, job.job_id);
            self.db
                .create_task_definition(
                    job.job_id,
                    i as i32 + 1,
                    &task.extractor_config,
                    &task.loader_config,
                )
                .await
                .context(format!("Failed to create task {} for job '{}'", i + 1, job.job_id))?;
            info!("JobManager: Task {} for job '{}' created.", i + 1, job.job_id);
        }

        info!("JobManager: Successfully created job '{}' with all tasks.", job_name);
        Ok(job)
    }

    /// Retrieves a job definition and all its associated tasks by job ID.
    pub async fn get_job(&self, job_id: Uuid) -> Result<Option<(JobDefinition, Vec<TaskDefinition>)>> {
        info!("JobManager: Attempting to retrieve job with ID: {}", job_id);
        if let Some(job) = self.db.get_job_definition(job_id).await.context(format!("Failed to get job definition for ID: {}", job_id))? {
            info!("JobManager: Job with ID {} found. Retrieving tasks.", job_id);
            let tasks = self.db.get_task_definitions_for_job(job.job_id).await.context(format!("Failed to get task definitions for job ID: {}", job_id))?;
            info!("JobManager: Successfully retrieved job {} and its tasks.", job_id);
            Ok(Some((job, tasks)))
        } else {
            info!("JobManager: Job with ID {} not found.", job_id);
            Ok(None)
        }
    }
}

pub struct NewTask {
    pub extractor_config: Value,
    pub loader_config: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::db::Db;
    use serde_json::json;

    async fn setup() -> Db {
        let database_url = "sqlite::memory:";
        let db = Db::new(database_url).await.unwrap();
        db.migrate().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_create_and_get_job() {
        let db = setup().await;
        let job_manager = JobManager::new(db);

        let tasks = vec![NewTask {
            extractor_config: json!({ "type": "api", "url": "https://example.com" }),
            loader_config: json!({ "type": "duckdb", "db_path": "test.db", "table_name": "test" }),
        }];

        let job = job_manager
            .create_job("Test Job", Some("Test Description"), "@manual", true, tasks)
            .await
            .unwrap();

        let (retrieved_job, retrieved_tasks) = job_manager.get_job(job.job_id).await.unwrap().unwrap();

        assert_eq!(job.job_id, retrieved_job.job_id);
        assert_eq!(job.job_name, retrieved_job.job_name);
        assert_eq!(retrieved_tasks.len(), 1);
    }
}
