// In src/lib.rs

//! Core library for the `orc-rust-ator` application.
//! 
//! This crate provides the main application logic, including API handling, job orchestration,
//! plugin management, state management, and worker execution. It also sets up the global
//! logger and initializes the database and background services.

// --- Imports ---
use anyhow::{Context, Result};
use std::env;
use tracing::info;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::api::handlers::{create_job, get_job, get_jobs, get_run, get_runs, health_check, run_job};

// --- Module Declarations ---
pub mod api;
pub mod orchestrator;
pub mod plugins;
pub mod state;
pub mod worker;
pub mod logger;

// --- Local Imports ---
use crate::orchestrator::job_manager::JobManager;
use crate::orchestrator::scheduler::Scheduler;
use crate::orchestrator::worker_manager::WorkerManager;
use crate::state::db::Db;

pub async fn run_app() -> Result<()> {
    // Initialize logging
    logger::initialize_logger();
    print_banner();
    info!("orc-rust-ator application starting...");

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let db = Db::new(&database_url).await?;
    db.migrate().await?;

    let job_manager = JobManager::new(db.clone());
    let scheduler = Scheduler::new(db.clone());
    let worker_manager = WorkerManager::new(db.clone());

    // Start the scheduler and worker manager in the background
    tokio::spawn(async move { scheduler.run().await });
    tokio::spawn(async move { worker_manager.run().await });

    // Build the API routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/jobs", post(create_job).get(get_jobs))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/:job_id/run", post(run_job))
        .route("/runs", get(get_runs))
        .route("/runs/:run_id", get(get_run))
        .with_state(db);

    // Run the API server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("API server listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(&addr).await?, app)
        .await
        .context("API server failed to start")?;

    Ok(())
}

fn print_banner() {
    println!(r"  ___  ____  ____     ____  _   _  ____  ____  ____  ____  ____  ____");
    println!(r" / _ \|  _ \|  _ \   |  _ \| | | |/ ___||  _ \|  _ \|  _ \|  _ \|  _ \");
    println!(r"| | | | |_) | |_) |  | |_) | |_| | |    | |_) | |_) | |_) | |_) | |_) |");
    println!(r"| |_| |  _ <|  _ <   |  _ <|  _  | |___ |  __/|  __/|  __/|  __/|  __/");
    println!(r" \___/|_| \_|_| \_\  |_| \_|_| |_|\____||_|   |_|   |_|   |_|   |_|   ");
    println!(r"\n");
}
