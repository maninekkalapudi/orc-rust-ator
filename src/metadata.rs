// src/metadata.rs

// --- Imports ---
// External Crate Imports
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use tracing::{debug, error, info}; // Correctly import all needed log levels
use uuid::{Timestamp, Uuid};

// Local Module Imports
use crate::config::{ExtractorConfig, LoaderConfig};
use crate::job_def::JobDefinition;


// --- Data Structures (FIXED) ---

/// Represents a job that is ready to be checked for scheduling.
#[derive(Debug)]
pub struct Job {
    pub job_id: String,
    pub schedule: String,
}

/// Represents a specific execution of a job that has been queued.
#[derive(Debug)]
pub struct JobRun {
    pub run_id: Uuid,
    pub job_id: String,
}

/// Represents a single, executable task with its configuration deserialized.
#[derive(Debug)]
pub struct RunnableTask {
    pub task_id: String,
    pub extractor_config: ExtractorConfig,
    pub loader_config: LoaderConfig,
}


// --- Database Functions ---

/// Creates the necessary metadata tables in the DuckDB database if they don't exist.
pub fn setup_metadata_db(conn: &Connection) -> Result<()> {
    info!("Setting up metadata database schema...");
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS jobs (
            job_id          VARCHAR PRIMARY KEY,
            description     VARCHAR,
            schedule        VARCHAR NOT NULL,
            is_active       BOOLEAN DEFAULT TRUE,
            created_at      TIMESTAMPTZ DEFAULT current_timestamp
        );
        CREATE TABLE IF NOT EXISTS job_runs (
            run_id          VARCHAR PRIMARY KEY,
            job_id          VARCHAR NOT NULL,
            status          VARCHAR NOT NULL,
            run_type        VARCHAR,
            created_at      TIMESTAMPTZ DEFAULT current_timestamp,
            started_at      TIMESTAMPTZ,
            ended_at        TIMESTAMPTZ,
            logs            VARCHAR
        );
        CREATE TABLE IF NOT EXISTS tasks (
            task_id         VARCHAR PRIMARY KEY,
            job_id          VARCHAR NOT NULL,
            task_order      INTEGER,
            extractor_config JSON,
            loader_config   JSON
        );
        "
    ).with_context(|| "Failed to execute schema setup batch")?;
    info!("Metadata schema is up-to-date.");
    Ok(())
}

/// Reads job definitions from a YAML file and seeds the `jobs` and `tasks` tables.
pub fn initialize_jobs_from_file(conn: &mut Connection, file_path: &str) -> Result<()> {
    info!(file_path, "Initializing metadata from configuration file.");

    let file = std::fs::File::open(file_path)
        .with_context(|| format!("Failed to open jobs definition file: {file_path}"))?;
    let job_defs: Vec<JobDefinition> = serde_yaml::from_reader(file)
        .context("Failed to parse YAML. Check for syntax errors or structural issues.")?;

    let tx = conn.transaction()
        .context("Failed to begin database transaction")?;

    info!("Clearing existing data from 'jobs' and 'tasks' tables.");
    tx.execute("DELETE FROM tasks", [])?;
    tx.execute("DELETE FROM jobs", [])?;

    for job_def in &job_defs {
        info!(job_id = %job_def.job_id, "Seeding job definition.");
        tx.execute(
            "INSERT INTO jobs (job_id, description, schedule, is_active) VALUES (?, ?, ?, ?)",
            params![
                job_def.job_id,
                job_def.description,
                job_def.schedule,
                job_def.is_active
            ],
        ).with_context(|| format!("Failed to insert job: {}", job_def.job_id))?;

        for task_def in &job_def.tasks {
            let extractor_json = serde_json::to_string(&task_def.extractor_config)?;
            let loader_json = serde_json::to_string(&task_def.loader_config)?;

            tx.execute(
                "INSERT INTO tasks (task_id, job_id, task_order, extractor_config, loader_config) VALUES (?, ?, ?, ?, ?)",
                // FIX: Use the JSON variables in the query parameters
                params![
                    task_def.task_id,
                    job_def.job_id,
                    task_def.task_order,
                    extractor_json,
                    loader_json
                ],
            ).with_context(|| format!("Failed to insert task: {}", task_def.task_id))?;
        }
    }

    tx.commit().context("Failed to commit transaction")?;
    info!(job_count = job_defs.len(), "Metadata initialization complete.");
    Ok(())
}

/// Fetches all active jobs from the database to be checked by the scheduler.
pub fn get_due_jobs(conn: &Connection) -> Result<Vec<Job>> {
    debug!("Fetching active jobs for scheduling check.");
    let mut stmt = conn.prepare("SELECT job_id, schedule FROM jobs WHERE is_active = TRUE")?;
    let jobs = stmt.query_map([], |row| {
        Ok(Job {
            job_id: row.get(0)?,
            schedule: row.get(1)?,
        })
    })?.collect::<Result<Vec<_>, _>>()
        .with_context(|| "Failed to query and collect due jobs")?;
    debug!(count = jobs.len(), "Found active jobs.");
    Ok(jobs)
}

/// Fetches and deserializes all tasks for a specific job_id.
pub fn get_tasks_for_job(conn: &Connection, job_id: &str) -> Result<Vec<RunnableTask>> {
    info!(job_id, "Fetching tasks for job.");
    let mut stmt = conn.prepare(
        "SELECT task_id, extractor_config, loader_config FROM tasks WHERE job_id = ? ORDER BY task_order ASC"
    )?;

    let tasks = stmt.query_map(params![job_id], |row| {
        let task_id: String = row.get(0)?;
        let extractor_json: String = row.get(1)?;
        let loader_json: String = row.get(2)?;

        let extractor_config: ExtractorConfig = serde_json::from_str(&extractor_json)
            .map_err(|e| duckdb::Error::FromSqlConversionFailure(1, duckdb::types::Type::Text, Box::new(e)))?;
        let loader_config: LoaderConfig = serde_json::from_str(&loader_json)
            .map_err(|e| duckdb::Error::FromSqlConversionFailure(2, duckdb::types::Type::Text, Box::new(e)))?;

        Ok(RunnableTask { task_id, extractor_config, loader_config })
    })?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Failed to collect and parse tasks for job '{job_id}'"))?;

    info!(job_id, task_count = tasks.len(), "Successfully fetched tasks.");
    Ok(tasks)
}

/// Inserts a new record into `job_runs` with a 'QUEUED' status.
pub fn queue_job_run(conn: &Connection, job_id: &str, run_type: &str) -> Result<Uuid> {
    let now = Utc::now();
    let timestamp = Timestamp::from_unix(&uuid::NoContext, now.timestamp() as u64, now.timestamp_subsec_nanos());
    let run_id = Uuid::new_v7(timestamp);
    let run_id_str = run_id.to_string();

    info!(job_id, %run_id, run_type, "Queueing new run for job.");
    conn.execute(
        "INSERT INTO job_runs (run_id, job_id, status, run_type) VALUES (?, ?, 'QUEUED', ?)",
        params![run_id_str, job_id, run_type],
    ).with_context(|| "Failed to insert new job run into database")?;

    Ok(run_id)
}

/// Finds the oldest queued job run to be executed by a worker.
pub fn get_queued_run(conn: &Connection) -> Result<Option<JobRun>> {
    debug!("Polling for a queued job run...");
    let mut stmt = conn.prepare(
        "SELECT run_id, job_id FROM job_runs WHERE status = 'QUEUED' ORDER BY created_at ASC LIMIT 1"
    )?;
    let mut rows = stmt.query_map([], |row| {
        let run_id_str: String = row.get(0)?;
        let run_id = Uuid::parse_str(&run_id_str)
            .map_err(|e| duckdb::Error::FromSqlConversionFailure(0, duckdb::types::Type::Text, Box::new(e)))?;
        Ok(JobRun { run_id, job_id: row.get(1)? })
    })?;

    if let Some(run_result) = rows.next() {
        let job_run = run_result.with_context(|| "Failed to parse queued job run from database row")?;
        info!(run_id = %job_run.run_id, job_id = %job_run.job_id, "Claimed queued run for execution.");
        Ok(Some(job_run))
    } else {
        debug!("No queued jobs found.");
        Ok(None)
    }
}

/// Updates the status and timestamps of a specific job run.
pub fn update_run_status(conn: &Connection, run_id: Uuid, status: &str) -> Result<()> {
    let now: DateTime<Utc> = Utc::now();
    let run_id_str = run_id.to_string();
    let now_str = now.to_rfc3339();

    info!(%run_id, new_status = status, "Updating run status.");

    let rows_affected = match status {
        "RUNNING" => {
            conn.execute(
                "UPDATE job_runs SET status = 'RUNNING', started_at = ? WHERE run_id = ?",
                params![now_str, run_id_str],
            )?
        }
        "SUCCESS" | "FAILED" => {
            conn.execute(
                "UPDATE job_runs SET status = ?, ended_at = ? WHERE run_id = ?",
                params![status, now_str, run_id_str],
            )?
        }
        _ => {
            debug!(%run_id, status, "Ignoring unknown status for update.");
            0
        }
    };

    if rows_affected == 0 && (status == "RUNNING" || status == "SUCCESS" || status == "FAILED") {
        error!(%run_id, "Attempted to update status, but no matching run_id was found in the database.");
    }

    Ok(())
}