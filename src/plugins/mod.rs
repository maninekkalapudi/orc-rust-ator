//! Defines the pluggable architecture for data extraction and loading.
//! 
//! This module provides traits (`Extractor`, `Loader`) that define the interface for
//! data source extraction and data destination loading. It also declares sub-modules
//! for specific extractor and loader implementations.

pub mod extractors;
pub mod loaders;

use anyhow::Result;
use async_trait::async_trait;
use polars::prelude::DataFrame;
use std::sync::Arc;

#[async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(&self) -> Result<DataFrame>;
}

#[async_trait]
pub trait Loader: Send + Sync {
    async fn load(&self, df: DataFrame) -> Result<()>;
}

pub enum PluginType {
    Extractor(Arc<dyn Extractor + Send + Sync>),
    Loader(Arc<dyn Loader + Send + Sync>),
}