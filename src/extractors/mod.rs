pub mod api_extractor;
pub mod csv_extractor;
pub mod parquet_extractor;

// Prelude submodule for organized re-exports
pub mod prelude {
    pub use super::api_extractor::ApiExtractor;
    pub use super::csv_extractor::CsvExtractor;
    pub use super::parquet_extractor::ParquetExtractor;
    // Could also include commonly used traits, types, etc.
}
