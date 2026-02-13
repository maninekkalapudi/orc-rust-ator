/*
 * File: src/orchestrator/scheduler.rs
 * Description: Schedules jobs based on their defined schedules.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Fixed scheduling logic using cron::Schedule::after.
 * - 2026-02-13: Added file header and documentation comments.
 */

//! Schedules jobs based on their defined schedules.
//! 
//! This module provides the `Scheduler` struct, which periodically checks for due jobs
//! and creates `JobRun` entries in the database for the `WorkerManager` to pick up.

use crate::state::db::Db;
use anyhow::{Context, Result};
use cron::Schedule;
use chrono::Utc;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info};

pub struct Scheduler {
    db: Db,
}

impl Scheduler {
    /// Creates a new `Scheduler` instance with the given database handle.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Starts the scheduler loop. This will periodically check for due jobs.
    pub async fn run(&self) -> Result<()> {
        info!("Scheduler started.");
        loop {
            debug!("Scheduler: Checking for due jobs...");
            match self.check_and_schedule_jobs().await {
                Ok(_) => {}
                Err(e) => error!("Scheduler error: {:?}", e),
            }
            sleep(Duration::from_secs(10)).await;
        }
    }

    /// Checks all active job definitions and creates a `JobRun` for any that are due.
    async fn check_and_schedule_jobs(&self) -> Result<()> {
        let jobs = self.db
            .get_all_job_definitions()
            .await
            .context("Scheduler: Failed to get all job definitions")?;

        let now = Utc::now();

        for job in jobs {
            if !job.is_active {
                debug!("Scheduler: Job {} is inactive, skipping.", job.job_id);
                continue;
            }

            // Special case for manual-only jobs
            if job.schedule == "@manual" {
                continue;
            }

            let schedule = match Schedule::from_str(&job.schedule) {
                Ok(s) => s,
                Err(e) => {
                    error!(
                        "Scheduler: Failed to parse schedule for job {}: {:?}",
                        job.job_id, e
                    );
                    continue;
                }
            };

            let last_run = self.db
                .get_last_job_run(job.job_id)
                .await
                .context(format!("Scheduler: Failed to get last run for job {}", job.job_id))?;

            let last_time = match last_run {
                Some(run) => run.created_at,
                None => job.created_at,
            };

            if let Some(next_due) = schedule.after(&last_time).next() {
                if next_due <= now {
                    info!("Scheduler: Scheduling job: {}", job.job_name);
                    self.db
                        .create_job_run(job.job_id, "queued", "scheduler")
                        .await
                        .context(format!(
                            "Scheduler: Failed to create job run for job {}",
                            job.job_id
                        ))?;
                    info!("Scheduler: Job {} scheduled successfully.", job.job_name);
                }
            }
        }
        Ok(())
    }
}
