// In src/extractors/parquet_extractor.rs

use crate::pipelines::Extractor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use polars::prelude::*;
use tracing::info;

pub struct ParquetExtractor {
    pub path: String,
}

#[async_trait]
impl Extractor for ParquetExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        info!(path = %self.path, "Extracting data from Parquet source.");

        // Clone the path to move it into the blocking thread.
        let path_clone = self.path.clone();

        // We move the entire synchronous Polars operation into a dedicated thread
        // to avoid blocking the async runtime.
        let df = tokio::task::spawn_blocking(move || {
            // Use scan_parquet for lazy reading, which is highly efficient.
            // It allows the engine to optimize the query before reading data.
            LazyFrame::scan_parquet(path_clone, ScanArgsParquet::default())?.collect()
            // Execute the lazy query to get a DataFrame.
        })
        .await? // Wait for the thread to finish. Propagates JoinError.
        .with_context(|| format!("Failed to read or parse Parquet file at '{}'", self.path))?; // Handle the inner Polars Result.

        info!(path = %self.path, rows = df.height(), "Parquet data extracted successfully.");
        Ok(df)
    }
}
