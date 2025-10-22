use anyhow::Result;
use reqwest;
use serde_json::json;
use tokio;
use tokio::time::{sleep, Duration};

mod common;

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let server_url = common::setup().await?;

    // Make a request to the /health endpoint.
    let client = reqwest::Client::new();
    let res = client.get(&format!("{}/health", server_url)).send().await?;

    // Assert that the request was successful.
    assert!(res.status().is_success());

    Ok(())
}

#[tokio::test]
async fn test_create_and_run_job_lifecycle() -> Result<()> {
    let server_url = common::setup().await?;
    let client = reqwest::Client::new();

    // 1. Create a new job
    let create_job_payload = json!({
        "job_name": "Test CSV to DuckDB",
        "description": "A test job for the full lifecycle",
        "schedule": "* * * * * *", // Every second
        "is_active": true,
        "tasks": [
            {
                "extractor_config": {
                    "type": "csv",
                    "path": "test_data.csv"
                },
                "loader_config": {
                    "type": "duckdb",
                    "db_path": ":memory:",
                    "table_name": "test_output"
                }
            }
        ]
    });

    let res = client
        .post(&format!("{}/jobs", server_url))
        .json(&create_job_payload)
        .send()
        .await?;

    assert!(res.status().is_success());
    let job_response: serde_json::Value = res.json().await?;
    let job_id = job_response["job_id"].as_str().unwrap().to_string();

    // 2. Manually trigger the job
    let res = client
        .post(&format!("{}/jobs/{}/run", server_url, job_id))
        .send()
        .await?;
    assert!(res.status().is_success());

    // 3. Poll for the job run status until it's completed
    let mut run_status = String::new();

    for _ in 0..10 { // Max 10 retries
        let res = client
            .get(&format!("{}/runs", server_url))
            .send()
            .await?;
        let runs: Vec<serde_json::Value> = res.json().await?;
        
        if let Some(run) = runs.into_iter().find(|r| r["job_id"] == job_id) {
            run_status = run["status"].as_str().unwrap().to_string();
            if run_status == "completed" || run_status == "failed" {
                break;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    // 4. Assert the final status is "completed"
    assert_eq!(run_status, "success");

    Ok(())
}