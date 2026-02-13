/*
 * File: src/utils/seeder.rs
 * Description: Utilities for seeding the database from YAML configuration files.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added file header and documentation comments.
 */

use anyhow::{Context, Result};
use crate::state::db::Db;
use crate::orchestrator::job_manager::{JobManager, NewTask};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct SeedJob {
    pub job_id: String,
    pub description: Option<String>,
    pub schedule: String,
    pub is_active: bool,
    pub tasks: Vec<SeedTask>,
}

#[derive(Debug, Deserialize)]
pub struct SeedTask {
    pub extractor_config: Value,
    pub loader_config: Value,
}

pub async fn seed_jobs(db: &Db, file_path: &str) -> Result<()> {
    info!("Seeding jobs from file: {}", file_path);
    let contents = fs::read_to_string(file_path).context(format!("Failed to read file: {}", file_path))?;
    let jobs: Vec<SeedJob> = serde_yaml::from_str(&contents).context("Failed to parse YAML")?;

    let job_manager = JobManager::new(db.clone());

    for job_data in jobs {
        // Check if job already exists (by name, which corresponds to YAML's job_id)
        // Since get_job takes a UUID, we can't easily check by name with current API.
        // But let's check all jobs and filter. This is slow but fine for seeding.
        let existing_jobs = db.get_all_job_definitions().await?;
        if existing_jobs.iter().any(|j| j.job_name == job_data.job_id) {
            warn!("Job '{}' already exists. Skipping.", job_data.job_id);
            continue;
        }

        let tasks: Vec<NewTask> = job_data.tasks.into_iter().map(|t| NewTask {
            extractor_config: t.extractor_config,
            loader_config: t.loader_config,
        }).collect();

        info!("Creating job: {}", job_data.job_id);
        job_manager.create_job(
            &job_data.job_id,
            job_data.description.as_deref(),
            &job_data.schedule,
            job_data.is_active,
            tasks,
        ).await?;
    }

    info!("Seeding completed successfully.");
    Ok(())
}
