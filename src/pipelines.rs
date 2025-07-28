// In src/pipelines.rs

use anyhow::Result;
use async_trait::async_trait;
use polars::prelude::DataFrame;

#[async_trait]
pub trait Extractor: Send + Sync {
    /// Extracts data from a source and returns it as a Polars DataFrame.
    async fn extract(&self) -> Result<DataFrame>;
}

#[async_trait]
pub trait Loader: Send + Sync {
    /// Loads a Polars DataFrame into a target destination.
    async fn load(&self, df: DataFrame) -> Result<()>;
}