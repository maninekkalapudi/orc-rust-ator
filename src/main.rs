// In src/main.rs


// --- Imports ---
use anyhow::{Context, Result};
use chrono::Utc;
use cron::Schedule;
use duckdb::Connection;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};
use std::process;

// --- Module Declarations ---
mod config;
mod job_def;
mod logger;
mod metadata;
mod pipelines;
mod extractors;
mod loaders;

// --- Local Imports ---
use crate::config::{ExtractorConfig, LoaderConfig};
// Cleaned up imports for better style
use crate::extractors::prelude::{ApiExtractor, CsvExtractor};
use crate::loaders::prelude::DuckDBLoader;
use crate::pipelines::{Extractor, Loader};


fn format_error_chain(e: &anyhow::Error) -> String {
    e.chain()
        .map(|cause| cause.to_string())
        .collect::<Vec<String>>()
        .join(" -> ") // Join causes with an arrow for readability
}


// --- Main Application Entry Point ---

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the global logger. This one line sets up everything.
    logger::initialize_logger();

    info!("Application starting up.");

    // 1. Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    info!(r"args: {:?}", args);
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    // 2. Establish a connection to the metadata database.
    let mut conn = Connection::open("metadata.db")
        .context("Failed to open metadata database 'metadata.db'")?;

    // 3. Route to the correct logic based on the command.
    match command {
        "init" => {
            info!("--- Running Initialization ---");
            metadata::setup_metadata_db(&conn)?;

            // --- CORRECTED LOGIC: Call the initialization function only ONCE ---
            if let Err(e) = metadata::initialize_jobs_from_file(&mut conn, "jobs.yaml") {
                let error_string = format_error_chain(&e);
                error!(error = %error_string, "A fatal error occurred during database initialization.");
                process::exit(1);
            }

            info!("--- Initialization Complete ---");
        }
        "run_job" => {
            info!("--- Running Manual Job Trigger ---");
            if let Some(job_id) = args.get(2) {
                info!(job_id, "Attempting to run job directly with metadata tracking.");

                // --- FULL IMPLEMENTATION FOR MANUAL RUNS ---

                // 1. Create a "MANUAL" run record in the database.
                let run_id = metadata::queue_job_run(&conn, job_id, "MANUAL")
                    .with_context(|| format!("Failed to create a manual run record for job '{job_id}'"))?;
                info!(%run_id, job_id, "Created manual run record.");

                // 2. Immediately update the status to 'RUNNING'.
                metadata::update_run_status(&conn, run_id, "RUNNING")?;

                // 3. Execute the job and capture the result.
                let job_result = execute_job(&conn, job_id).await;

                // 4. Match on the result to set the final status.
                match job_result {
                    Ok(_) => {
                        metadata::update_run_status(&conn, run_id, "SUCCESS")?;
                        info!(%run_id, job_id, "Manual job run finished successfully.");
                    }
                    Err(e) => {
                        metadata::update_run_status(&conn, run_id, "FAILED")?;
                        let error_string = format_error_chain(&e);
                        error!(%run_id, job_id, error = %error_string, "Manual job run failed.");
                        process::exit(1);
                    }
                }
            } else {
                error!("The 'run_job' command requires a job_id.");
                println!(r"\nUsage: orc_rust_ator run_job <job_id>");
                process::exit(1);
            }
        }
        "scheduler" => {
            info!("--- Running Scheduler Cycle ---");
            if let Err(e) = run_scheduler(&conn).await {
                let error_string = format_error_chain(&e);
                error!(error = %error_string, "A fatal error occurred during the scheduler cycle.");
                process::exit(1);
            }
            info!("--- Scheduler Cycle Complete ---");
        }
        "run_worker" => {
            info!("--- Running Worker Cycle ---");
            if let Err(e) = run_worker(&conn).await {
                let error_string = format_error_chain(&e);
                error!(error = %error_string, "Manual job run failed.");
                process::exit(1);
            }
            info!("--- Worker Cycle Complete ---");
        }
        "help" => {
            // We need to update the help text to include the new command
            println!(r"Orc_Rust_Ator: A Rust-based ELT Orchestrator");
            println!(r"\nUsage: orc_rust_ator <command>");
            println!("\nCommands:");
            println!("  init                - Initialize the metadata database from jobs.yaml. Run this first.");
            println!("  run_job <job_id>    - Manually run a specific job by its ID, bypassing the scheduler."); // <-- NEW HELP TEXT
            println!("  scheduler           - Checks for due jobs and queues them to run.");
            println!("  run_worker          - Picks up and executes one queued job from the queue.");
            println!("  help                - Shows this help message.");
        }
        _ => {
            // Use structured logging for better machine-readability
            error!(command, "Unknown command. Use 'help' for a list of commands.");
        }
    }

    Ok(())
}


// --- Orchestrator Logic: Scheduler ---

async fn run_scheduler(conn: &Connection) -> Result<()> {
    let jobs = metadata::get_due_jobs(conn)?;
    let now = Utc::now();

    if jobs.is_empty() {
        info!("No active jobs found in metadata.");
        return Ok(());
    }

    for job in jobs {
        if job.schedule == "@manual" {
            continue;
        }
        let schedule = Schedule::from_str(&job.schedule)
            .with_context(|| format!("Invalid cron schedule for job '{}': {}", job.job_id, job.schedule))?;

        if let Some(next_run) = schedule.upcoming(Utc).next() {
            if next_run <= now {
                info!(job_id = %job.job_id, "Job is due. Queuing run.");
                metadata::queue_job_run(conn, &job.job_id, "SCHEDULED")?;
            }
        }
    }
    Ok(())
}


// --- Orchestrator Logic: Worker ---

async fn run_worker(conn: &Connection) -> Result<()> {
    if let Some(run) = metadata::get_queued_run(conn)? {
        info!(run_id = ?run.run_id, job_id = %run.job_id, "Claimed run. Starting execution...");
        metadata::update_run_status(conn, run.run_id, "RUNNING")?;
        let job_result = execute_job(conn, &run.job_id).await;

        match job_result {
            Ok(_) => {
                metadata::update_run_status(conn, run.run_id, "SUCCESS")?;
                info!(run_id = ?run.run_id, job_id = %run.job_id, "Run finished successfully.");
            }
            Err(e) => {
                metadata::update_run_status(conn, run.run_id, "FAILED")?;
                error!(run_id = ?run.run_id, job_id = %run.job_id, error = ?e, "Run failed.");
            }
        }
    } else {
        info!("No queued jobs to run.");
    }

    Ok(())
}

async fn execute_job(conn: &Connection, job_id: &str) -> Result<()> {
    let tasks = metadata::get_tasks_for_job(conn, job_id)?;
    info!(job_id, "Found {} tasks for job.", tasks.len());

    for task in tasks {
        info!(task_id = %task.task_id, "-- Starting task --");

        let extractor: Arc<dyn Extractor + Send + Sync> = match task.extractor_config {
            // Pass the path and dtypes to CsvExtractor
            ExtractorConfig::Csv { path } => Arc::new(CsvExtractor { path }),
            // Pass the url to ApiExtractor
            ExtractorConfig::Api { url } => Arc::new(ApiExtractor { url }),
        };

        let loader: Arc<dyn Loader + Send + Sync> = match task.loader_config {
            LoaderConfig::DuckDB { db_path, table_name } => {
                Arc::new(DuckDBLoader { db_path, table_name })
            }
        };

        // 1. EXTRACT the data into a DataFrame.
        info!(task_id = %task.task_id, "Extracting data from source...");
        let df = extractor.extract().await
            .with_context(|| format!("Extraction failed for task '{}'", task.task_id))?;

        // 2. LOAD the DataFrame into the destination.
        info!(task_id = %task.task_id, "Loading data into destination...");
        loader.load(df).await
            .with_context(|| format!("Loading failed for task '{}'", task.task_id))?;

        info!(task_id = %task.task_id, "Task completed successfully.");
    }

    Ok(())
}
