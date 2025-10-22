//! Manages worker processes and the execution of individual job runs.
//! 
//! This module provides the `WorkerManager` struct, which periodically polls the database
//! for queued job runs, dispatches them to worker tasks, and handles their completion or failure.

use crate::state::db::Db;
use crate::worker::run_worker;
use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, debug};

pub struct WorkerManager {
    db: Db,
}

impl WorkerManager {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> Result<()> {
        info!("WorkerManager started.");
        loop {
            debug!("WorkerManager: Checking for queued job runs...");
            if let Some(job_run) = self.db.get_queued_job_run().await? {
                info!("WorkerManager: Found queued job run: {}", job_run.run_id);
                self.db
                    .update_job_run_status(job_run.run_id.clone(), "running")
                    .await?;
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

            sleep(Duration::from_secs(10)).await;
        }
    }
}
