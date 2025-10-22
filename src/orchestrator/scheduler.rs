//! Schedules jobs based on their defined schedules.
//! 
//! This module provides the `Scheduler` struct, which periodically checks for due jobs
//! and creates `JobRun` entries in the database for the `WorkerManager` to pick up.

use crate::state::db::Db;
use anyhow::Result;
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
        let jobs = self.db.get_all_job_definitions().await?;

        for job in jobs {
            if !job.is_active {
                debug!("Job {} is inactive, skipping.", job.job_id);
                continue;
            }

            let schedule = Schedule::from_str(&job.schedule)?;
            let now = chrono::Utc::now();

            // Check if the job is due
            if let Some(next_due) = schedule.upcoming(Utc).next() {
                if next_due <= now {
                    // Check if a run for this schedule has already been created
                    // This is a simplified check. A more robust solution would involve
                    // storing the last scheduled time or a run ID associated with the schedule.
                    let last_run = self.db.get_last_job_run(job.job_id.clone()).await?;
                    let should_schedule = match last_run {
                        Some(run) => {
                            // Only schedule if the last run was before the current due time
                            // or if it failed and needs a retry (logic to be refined)
                            run.created_at < next_due
                        }
                        None => true, // No previous runs, so schedule it
                    };

                    if should_schedule {
                        info!("Scheduling job: {}", job.job_name);
                        self.db
                            .create_job_run(job.job_id.clone(), "queued", "scheduler")
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
