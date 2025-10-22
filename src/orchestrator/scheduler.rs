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
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Scheduler started.");
        loop {
            debug!("Scheduler: Checking for due jobs...");
            match self.check_and_schedule_jobs().await {
                Ok(_) => {},
                Err(e) => error!("Scheduler error: {:?}", e),
            }
            sleep(Duration::from_secs(60)).await;
        }
    }

    async fn check_and_schedule_jobs(&self) -> Result<()> {
        let jobs = self.db.get_all_job_definitions().await.context("Scheduler: Failed to get all job definitions")?;

        for job in jobs {
            if !job.is_active {
                debug!("Scheduler: Job {} is inactive, skipping.", job.job_id);
                continue;
            }

            let schedule = match Schedule::from_str(&job.schedule) {
                Ok(s) => s,
                Err(e) => {
                    error!("Scheduler: Failed to parse schedule for job {}: {:?}", job.job_id, e);
                    continue; // Skip this job if schedule is invalid
                }
            };
            let now = chrono::Utc::now();

            // Check if the job is due
            if let Some(next_due) = schedule.upcoming(Utc).next() {
                if next_due <= now {
                    let last_run = self.db.get_last_job_run(job.job_id.clone()).await.context(format!("Scheduler: Failed to get last run for job {}", job.job_id))?;
                    let should_schedule = match last_run {
                        Some(run) => {
                            if run.created_at < next_due {
                                true
                            } else {
                                debug!("Scheduler: Job {} is due but already has a recent run. Skipping.", job.job_id);
                                false
                            }
                        }
                        None => true, // No previous runs, so schedule it
                    };

                    if should_schedule {
                        info!("Scheduler: Scheduling job: {}", job.job_name);
                        self.db
                            .create_job_run(job.job_id.clone(), "queued", "scheduler")
                            .await
                            .context(format!("Scheduler: Failed to create job run for job {}", job.job_id))?;
                        info!("Scheduler: Job {} scheduled successfully.", job.job_name);
                    }
                }
            }
        }
        Ok(())
    }
}
