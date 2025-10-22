//! Extracts data from a specified CSV file.
//! 
//! This module provides the `CsvExtractor` struct, which implements the `Extractor` trait
//! to read data from a local CSV file and parse it into a Polars DataFrame.

use anyhow::Result;
use async_trait::async_trait;
use polars::prelude::*;

use crate::plugins::Extractor;

pub struct CsvExtractor {
    pub path: String,
}

#[async_trait]
impl Extractor for CsvExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        let path_clone = self.path.clone();
        let df = LazyCsvReader::new(path_clone)
            .with_has_header(true)
            .finish()?;
        Ok(df.collect()?)
    }
}