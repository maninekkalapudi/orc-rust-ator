// In src/extractors/csv_extractor.rs

use crate::pipelines::Extractor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use polars::prelude::*;
use tracing::info;

pub struct CsvExtractor {
    pub path: String,
}

#[async_trait]
impl Extractor for CsvExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        info!(path = %self.path, "Extracting data from CSV source.");

        // Clone the path so it can be moved into the blocking thread.
        let path_clone = self.path.clone();

        // --- THE FIX IS HERE ---
        // We move the entire blocking Polars operation into a dedicated thread.
        let df = tokio::task::spawn_blocking(move || {
            LazyCsvReader::new(path_clone)
                .with_has_header(true)
                .with_infer_schema_length(Some(100))
                .finish()?
                .collect()
        })
            .await? // The first `await` waits for the thread to finish.
            .with_context(|| format!("Failed to read or parse CSV file at '{}'", self.path))?; // The second `?` handles the inner Polars Result.

        info!(path = %self.path, rows = df.height(), "CSV data extracted successfully.");
        Ok(df)
    }
}