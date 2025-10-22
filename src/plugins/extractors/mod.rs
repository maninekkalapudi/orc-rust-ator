//! Houses various data extractor implementations.
//! 
//! This module contains concrete implementations of the `Extractor` trait for different
//! data sources, such as APIs, CSV files, and Parquet files.

pub mod api_extractor;
pub mod csv_extractor;
pub mod parquet_extractor;