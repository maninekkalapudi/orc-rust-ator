// In src/extractors/api_extractor.rs

use crate::pipelines::Extractor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use polars::prelude::*;
use std::io::Cursor;
use std::num::NonZeroUsize;
use tracing::{debug, info};

pub struct ApiExtractor {
    pub url: String,
}

#[async_trait]
impl Extractor for ApiExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        info!(url = %self.url, "Extracting data from API source.");

        // The `reqwest` part is already async, so it stays here.
        let response_bytes = reqwest::get(&self.url).await?.bytes().await?;
        debug!(bytes_count = response_bytes.len(), "API response body read into memory.");

        // --- THE FIX IS HERE ---
        // The parsing part is blocking, so we move it to a blocking thread.
        let df = tokio::task::spawn_blocking(move || {
            let cursor = Cursor::new(response_bytes);
            JsonReader::new(cursor)
                .infer_schema_len(NonZeroUsize::new(100))
                .finish()
        })
            .await? // Wait for the thread to finish.
            .context("Failed to parse API response as JSON into a DataFrame")?; // Handle the inner Polars Result.

        info!(url = %self.url, rows = df.height(), "API data extracted successfully.");
        Ok(df)
    }
}