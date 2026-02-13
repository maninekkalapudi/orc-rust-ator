/*
 * File: src/lib.rs
 * Description: Core library for the `orc-rust-ator` application.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Refactored database initialization to support SQLite.
 * - 2026-02-13: Added file header and documentation comments.
 */

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

use axum::Router;

use std::net::SocketAddr;



// --- Module Declarations ---

pub mod api;
pub mod orchestrator;
pub mod plugins;
pub mod state;
pub mod auth;
pub mod worker;
pub mod utils;
pub mod logger;
pub mod models;


// --- Local Imports ---

use crate::orchestrator::scheduler::Scheduler;

use crate::orchestrator::worker_manager::WorkerManager;
use crate::state::db::Db;



use crate::api::grpc_service::MyJobService;
use crate::api::grpc_service::proto::job_service_server::JobServiceServer;

pub async fn run_app() -> Result<()> {
    logger::initialize_logger();
    print_banner();
    info!("orc-rust-ator application starting...");

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let db = Db::new(&database_url).await?;
    db.migrate().await?;

    let scheduler = Scheduler::new(db.clone());
    let worker_manager = WorkerManager::new(db.clone());

    tokio::spawn(async move { scheduler.run().await });
    tokio::spawn(async move { worker_manager.run().await });

    let rest_api = api::app(db.clone());

    let grpc_service = MyJobService { db: db.clone() };
    let grpc_server = JobServiceServer::new(grpc_service);

    let grpc_router = tonic::service::Routes::new(grpc_server).into_axum_router();

    let app = Router::new()
        .merge(rest_api)
        .merge(grpc_router);


    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await.context("Server failed to start")?;

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
