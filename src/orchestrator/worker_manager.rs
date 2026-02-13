/*
 * File: src/orchestrator/worker_manager.rs
 * Description: Manages worker processes and the execution of individual job runs.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Reduced polling interval for better responsiveness.
 * - 2026-02-13: Added file header and documentation comments.
 */

//! Manages worker processes and the execution of individual job runs.
//! 
//! This module provides the `WorkerManager` struct, which periodically polls the database
//! for queued job runs, dispatches them to worker tasks, and handles their completion or failure.

use crate::state::db::Db;
use crate::worker::run_worker;
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, debug};

pub struct WorkerManager {
    db: Db,
}

impl WorkerManager {
    /// Creates a new `WorkerManager` with the given database handle.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Starts the worker manager loop. It polls the database for queued job runs
    /// and spawns a worker task for each one found.
    pub async fn run(&self) -> Result<()> {
        info!("WorkerManager started.");
        loop {
            debug!("WorkerManager: Checking for queued job runs...");
            let job_run_option = self.db.get_queued_job_run().await.context("WorkerManager: Failed to get queued job run")?;
            if let Some(job_run) = job_run_option {
                info!("WorkerManager: Found queued job run: {}", job_run.run_id);
                self.db
                    .update_job_run_status(job_run.run_id.clone(), "running")
                    .await
                    .context(format!("WorkerManager: Failed to update job run {} status to 'running'", job_run.run_id))?;
                info!("WorkerManager: Job run {} status set to 'running'.", job_run.run_id);

                let db_clone = self.db.clone();
                let job_run_clone = job_run.clone();
                tokio::spawn(async move {
                    debug!("WorkerManager: Spawning worker for job run: {}", job_run_clone.run_id);
                    if let Err(e) = run_worker(db_clone.clone(), job_run_clone.clone()).await {
                        error!("WorkerManager: Worker for run {} failed: {:?}", job_run_clone.run_id, e);
                        db_clone.update_job_run_status_with_error(job_run_clone.run_id, "failed", &e.to_string()).await.ok();
                    } else {
                        info!("WorkerManager: Worker for run {} completed successfully.", job_run_clone.run_id);
                    }
                });
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}
