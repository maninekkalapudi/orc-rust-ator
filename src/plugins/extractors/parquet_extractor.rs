//! Extracts data from a specified Parquet file.
//! 
//! This module provides the `ParquetExtractor` struct, which implements the `Extractor` trait
//! to read data from a local Parquet file and parse it into a Polars DataFrame.

use anyhow::Result;
use async_trait::async_trait;
use polars::prelude::*;

use crate::plugins::Extractor;

pub struct ParquetExtractor {
    pub path: String,
}

#[async_trait]
impl Extractor for ParquetExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        let path_clone = self.path.clone();
        let df = LazyFrame::scan_parquet(path_clone, ScanArgsParquet::default())?.collect()?;
        Ok(df)
    }
}