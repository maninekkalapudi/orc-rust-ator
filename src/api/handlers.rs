// In src/api/handlers.rs

//! Defines the API endpoint handlers for job management and monitoring.
//! 
//! This module contains functions that handle incoming HTTP requests, interact with the
//! `JobManager` and database, and return appropriate HTTP responses.

use crate::orchestrator::job_manager::{JobManager, NewTask};
use crate::state::db::Db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::Value;

use tracing::{info, error}; // Added tracing imports

// --- Job Handlers ---

#[derive(Deserialize)]
pub struct CreateJobRequest {
    pub job_name: String,
    pub description: Option<String>,
    pub schedule: String,
    pub is_active: bool,
    pub tasks: Vec<NewTaskRequest>,
}

#[derive(Deserialize)]
pub struct NewTaskRequest {
    pub extractor_config: Value,
    pub loader_config: Value,
}

pub async fn create_job(
    State(db): State<Db>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Received request to create job: {}", payload.job_name);

    let job_manager = JobManager::new(db);
    let tasks = payload
        .tasks
        .into_iter()
        .map(|t| NewTask {
            extractor_config: t.extractor_config,
            loader_config: t.loader_config,
        })
        .collect();

    let job = job_manager
        .create_job(
            &payload.job_name,
            payload.description.as_deref(),
            &payload.schedule,
            payload.is_active,
            tasks,
        )
        .await
        .map_err(|e| {
            error!("Failed to create job {}: {:?}", payload.job_name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Successfully created job: {}", job.job_name);
    Ok(Json(serde_json::to_value(job).unwrap()))
}

pub async fn get_jobs(State(db): State<Db>) -> Result<Json<Value>, StatusCode> {
    info!("Received request to get all jobs.");
    let jobs = db
        .get_all_job_definitions()
        .await
        .map_err(|e| {
            error!("Failed to get all jobs: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Successfully retrieved all jobs.");
    Ok(Json(serde_json::to_value(jobs).unwrap()))
}

pub async fn get_job(State(db): State<Db>, Path(job_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Received request to get job: {}", job_id);
    let job_manager = JobManager::new(db);
    let job = job_manager
        .get_job(job_id.clone()) // Clone job_id for logging
        .await
        .map_err(|e| {
            error!("Failed to get job {}: {:?}", job_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Successfully retrieved job: {}", job_id);
    Ok(Json(serde_json::to_value(job).unwrap()))
}

pub async fn run_job(State(db): State<Db>, Path(job_id): Path<String>) -> Result<StatusCode, StatusCode> {
    info!("Received request to run job: {}", job_id);
    db.create_job_run(job_id.clone(), "queued", "manual") // Clone job_id for logging
        .await
        .map_err(|e| {
            error!("Failed to run job {}: {:?}", job_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Successfully queued job run for job: {}", job_id);
    Ok(StatusCode::OK)
}

// --- Run Handlers ---

pub async fn get_runs(State(db): State<Db>) -> Result<Json<Value>, StatusCode> {
    info!("Received request to get all job runs.");
    // This is a simplified implementation. In a real application, you would want to add pagination.
    let runs = db
        .get_all_job_runs()
        .await
        .map_err(|e| {
            error!("Failed to get all job runs: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Successfully retrieved all job runs.");
    Ok(Json(serde_json::to_value(runs).unwrap()))
}

pub async fn get_run(State(db): State<Db>, Path(run_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Received request to get job run: {}", run_id);
    let run = db
        .get_job_run(run_id.clone()) // Clone run_id for logging
        .await
        .map_err(|e| {
            error!("Failed to get job run {}: {:?}", run_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Successfully retrieved job run: {}", run_id);
    Ok(Json(serde_json::to_value(run).unwrap()))
}

pub async fn health_check() -> Result<StatusCode, StatusCode> {
    tracing::info!("Health check requested.");
    Ok(StatusCode::OK)
}

