// In src/worker/mod.rs

use crate::plugins::extractors::api_extractor::ApiExtractor;
use crate::plugins::extractors::csv_extractor::CsvExtractor;
use crate::plugins::extractors::parquet_extractor::ParquetExtractor;
use crate::plugins::loaders::duckdb_loader::DuckDBLoader;
use crate::plugins::{Extractor, Loader};
use crate::state::db::{Db, JobRun};
use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use tracing::{info, error, debug}; // Added tracing imports

pub async fn run_worker(db: Db, job_run: JobRun) -> Result<()> {
    info!("Worker: Starting worker for job run: {}", job_run.run_id);
    let result = execute_job_with_retries(&db, &job_run).await;

    match result {
        Ok(_) => {
            info!("Worker: Job run {} completed successfully. Updating status to 'success'.", job_run.run_id);
            db.update_job_run_status(job_run.run_id.clone(), "success").await.context(format!("Worker: Failed to update job run {} status to 'success'", job_run.run_id))?;
            info!("Worker: Job run {} status updated to 'success'.", job_run.run_id);
        }
        Err(e) => {
            error!("Worker: Job run {} failed: {:?}. Updating status to 'failed'.", job_run.run_id, e);
            db.update_job_run_status_with_error(job_run.run_id.clone(), "failed", &e.to_string()).await.context(format!("Worker: Failed to update job run {} status to 'failed'", job_run.run_id))?;
            error!("Worker: Job run {} status updated to 'failed'.", job_run.run_id);
        }
    }

    Ok(())
}

async fn execute_job_with_retries(db: &Db, job_run: &JobRun) -> Result<()> {
    let max_retries = 3;
    let mut attempts = 0;
    info!("Worker: Executing job run {} with max retries: {}", job_run.run_id, max_retries);

    loop {
        match execute_job(db, job_run).await {
            Ok(_) => {
                info!("Worker: Job run {} completed successfully after {} attempts.", job_run.run_id, attempts + 1);
                return Ok(());
            },
            Err(e) => {
                attempts += 1;
                error!("Worker: Job run {} failed on attempt {}/{}: {:?}", job_run.run_id, attempts, max_retries, e);
                if attempts >= max_retries {
                    error!("Worker: Job run {} failed after {} attempts. No more retries.", job_run.run_id, max_retries);
                    return Err(e);
                }
                debug!("Worker: Retrying job run {} in 5 seconds...", job_run.run_id);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn execute_job(db: &Db, job_run: &JobRun) -> Result<()> {
    info!("Worker: Executing job {} for run {}.", job_run.job_id, job_run.run_id);
    let tasks = db.get_task_definitions_for_job(job_run.job_id.clone()).await.context(format!("Worker: Failed to get task definitions for job {}", job_run.job_id))?;

    for (i, task) in tasks.into_iter().enumerate() {
        info!("Worker: Processing task {} for job {}.", i + 1, job_run.job_id);
        let extractor = get_extractor(&task.extractor_config).context(format!("Worker: Failed to get extractor for task {} in job {}", i + 1, job_run.job_id))?;
        let loader = get_loader(&task.loader_config).context(format!("Worker: Failed to get loader for task {} in job {}", i + 1, job_run.job_id))?;

        info!("Worker: Extracting data for task {} in job {}.", i + 1, job_run.job_id);
        let df = extractor.extract().await.context(format!("Worker: Extraction failed for task {} in job {}", i + 1, job_run.job_id))?;
        info!("Worker: Data extracted for task {} in job {}. Rows: {}", i + 1, job_run.job_id, df.height()); // Assuming df has a height() method

        info!("Worker: Loading data for task {} in job {}.", i + 1, job_run.job_id);
        loader.load(df).await.context(format!("Worker: Loading failed for task {} in job {}", i + 1, job_run.job_id))?;
        info!("Worker: Data loaded for task {} in job {}.", i + 1, job_run.job_id);
    }

    info!("Worker: All tasks for job {} in run {} completed.", job_run.job_id, job_run.run_id);
    Ok(())
}

fn get_extractor(config: &Value) -> Result<Arc<dyn Extractor + Send + Sync>> {
    let extractor_type = config["type"].as_str().context("Extractor type not specified")?;
    debug!("Worker: Getting extractor of type: {}", extractor_type);
    match extractor_type {
        "api" => {
            let url = config["url"].as_str().context("URL not specified for API extractor")?;
            debug!("Worker: Created API extractor for URL: {}", url);
            Ok(Arc::new(ApiExtractor { url: url.to_string() }))
        }
        "csv" => {
            let path = config["path"].as_str().context("Path not specified for CSV extractor")?;
            debug!("Worker: Created CSV extractor for path: {}", path);
            Ok(Arc::new(CsvExtractor { path: path.to_string() }))
        }
        "parquet" => {
            let path = config["path"].as_str().context("Path not specified for Parquet extractor")?;
            debug!("Worker: Created Parquet extractor for path: {}", path);
            Ok(Arc::new(ParquetExtractor { path: path.to_string() }))
        }
        _ => {
            error!("Worker: Unsupported extractor type: {}", extractor_type);
            Err(anyhow::anyhow!("Unsupported extractor type: {}", extractor_type))
        }
    }
}

fn get_loader(config: &Value) -> Result<Arc<dyn Loader + Send + Sync>> {
    let loader_type = config["type"].as_str().context("Loader type not specified")?;
    debug!("Worker: Getting loader of type: {}", loader_type);
    match loader_type {
        "duckdb" => {
            let db_path = config["db_path"].as_str().context("db_path not specified for DuckDB loader")?;
            let table_name = config["table_name"].as_str().context("table_name not specified for DuckDB loader")?;
            debug!("Worker: Created DuckDB loader for path: {} and table: {}", db_path, table_name);
            Ok(Arc::new(DuckDBLoader::new(db_path, table_name)))
        }
        _ => {
            error!("Worker: Unsupported loader type: {}", loader_type);
            Err(anyhow::anyhow!("Unsupported loader type: {}", loader_type))
        }
    }
}
