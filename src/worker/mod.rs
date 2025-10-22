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

pub async fn run_worker(db: Db, job_run: JobRun) -> Result<()> {
    let result = execute_job_with_retries(&db, &job_run).await;

    match result {
        Ok(_) => {
            db.update_job_run_status(job_run.run_id.clone(), "success").await?;
        }
        Err(e) => {
            db.update_job_run_status_with_error(job_run.run_id.clone(), "failed", &e.to_string()).await?;
        }
    }

    Ok(())
}

async fn execute_job_with_retries(db: &Db, job_run: &JobRun) -> Result<()> {
    let max_retries = 3;
    let mut attempts = 0;

    loop {
        match execute_job(db, job_run).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn execute_job(db: &Db, job_run: &JobRun) -> Result<()> {
    let tasks = db.get_task_definitions_for_job(job_run.job_id.clone()).await?;

    for task in tasks {
        let extractor = get_extractor(&task.extractor_config)?;
        let loader = get_loader(&task.loader_config)?;

        let df = extractor.extract().await?;
        loader.load(df).await?;
    }

    Ok(())
}

fn get_extractor(config: &Value) -> Result<Arc<dyn Extractor + Send + Sync>> {
    let extractor_type = config["type"].as_str().context("Extractor type not specified")?;
    match extractor_type {
        "api" => {
            let url = config["url"].as_str().context("URL not specified for API extractor")?;
            Ok(Arc::new(ApiExtractor { url: url.to_string() }))
        }
        "csv" => {
            let path = config["path"].as_str().context("Path not specified for CSV extractor")?;
            Ok(Arc::new(CsvExtractor { path: path.to_string() }))
        }
        "parquet" => {
            let path = config["path"].as_str().context("Path not specified for Parquet extractor")?;
            Ok(Arc::new(ParquetExtractor { path: path.to_string() }))
        }
        _ => Err(anyhow::anyhow!("Unsupported extractor type: {}", extractor_type)),
    }
}

fn get_loader(config: &Value) -> Result<Arc<dyn Loader + Send + Sync>> {
    let loader_type = config["type"].as_str().context("Loader type not specified")?;
    match loader_type {
        "duckdb" => {
            let db_path = config["db_path"].as_str().context("db_path not specified for DuckDB loader")?;
            let table_name = config["table_name"].as_str().context("table_name not specified for DuckDB loader")?;
            Ok(Arc::new(DuckDBLoader::new(db_path, table_name)))
        }
        _ => Err(anyhow::anyhow!("Unsupported loader type: {}", loader_type)),
    }
}
