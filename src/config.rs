use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub pipelines: Vec<PipelineConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PipelineConfig {
    pub name: String,
    pub schedule: String,
    pub extractor: ExtractorConfig,
    pub loader: LoaderConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExtractorConfig {
    #[serde(rename = "csv")]
    Csv { path: String },
    #[serde(rename = "api")]
    Api { url: String },
    #[serde(rename = "parquet")]
    Parquet { path: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoaderConfig {
    #[serde(rename = "duckdb")]
    DuckDB { db_path: String, table_name: String },
    // Add other loaders here
}

// pub fn load_config(path: &str) -> anyhow::Result<Config> {
//     let file = std::fs::File::open(path)?;
//     let config: Config = serde_yaml::from_reader(file)?;
//     Ok(config)
// }
