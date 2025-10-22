//! Extracts data from a specified API endpoint.
//! 
//! This module provides the `ApiExtractor` struct, which implements the `Extractor` trait
//! to fetch data from a given URL and parse it into a Polars DataFrame.

use anyhow::Result;
use async_trait::async_trait;
use polars::prelude::*;
use std::io::Cursor;

use crate::plugins::Extractor;

pub struct ApiExtractor {
    pub url: String,
}

#[async_trait]
impl Extractor for ApiExtractor {
    async fn extract(&self) -> Result<DataFrame> {
        let response = reqwest::get(&self.url).await?.text().await?;
        let cursor = Cursor::new(response.as_bytes());
        let df = JsonReader::new(cursor)
            .infer_schema_len(None)
            .finish()?;
        Ok(df)
    }
}