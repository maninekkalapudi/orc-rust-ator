pub mod api_extractor;
pub mod csv_extractor;

// Prelude submodule for organized re-exports
pub mod prelude {
    pub use super::api_extractor::ApiExtractor;
    pub use super::csv_extractor::CsvExtractor;
    // Could also include commonly used traits, types, etc.
}