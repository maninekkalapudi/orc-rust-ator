//! Defines the application's REST API and routing.
//! 
//! This module sets up the Axum router and defines the endpoints for managing
//! jobs and monitoring their execution. It integrates with the application's
//! state (database) and orchestrator components.

use axum::{routing::{get, post}, Router};
use crate::state::db::Db;

pub mod handlers;

pub fn app(db: Db) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/jobs", post(handlers::create_job).get(handlers::get_jobs))
        .route("/jobs/:job_id", get(handlers::get_job))
        .route("/jobs/:job_id/run", post(handlers::run_job))
        .route("/runs", get(handlers::get_runs))
        .route("/runs/:run_id", get(handlers::get_run))
        .with_state(db)
}
