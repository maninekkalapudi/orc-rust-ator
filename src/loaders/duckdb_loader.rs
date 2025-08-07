// src/loaders/duckdb_loader.rs

use crate::pipelines::Loader;
use anyhow::{Context, Result};
use async_trait::async_trait;
use duckdb::{params, Connection};
use polars::prelude::*;
use std::fs::File;
use tempfile::NamedTempFile;
use tracing::{info, warn};

pub struct DuckDBLoader {
    pub db_path: String,
    pub table_name: String,
}

#[async_trait]
impl Loader for DuckDBLoader {
    async fn load(&self, mut df: DataFrame) -> Result<()> {
        info!(
            path = %self.db_path,
            table = %self.table_name,
            rows = df.height(),
            "Starting DuckDB load operation (IPC File Bridge with function detection)."
        );

        // --- Step 1: Create a temporary file to act as the bridge ---
        let temp_file =
            NamedTempFile::new().context("Failed to create a temporary file for the IPC bridge")?;
        let temp_file_path = temp_file
            .path()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Temporary file path is not valid UTF-8"))?;

        // Create a proper file handle for the writer.
        let file = File::create(temp_file_path).with_context(|| {
            format!("Failed to create file handle for temp path: {temp_file_path}")
        })?;

        // --- Step 2: Write the Polars DataFrame to the temp file using IPC format ---
        info!(temp_path = %temp_file_path, "Writing DataFrame to temporary IPC file.");
        IpcStreamWriter::new(file)
            .finish(&mut df)
            .context("Failed to write DataFrame to IPC stream file")?;

        // --- Step 3: Tell DuckDB to bulk-load from the IPC file ---
        let conn = Connection::open(&self.db_path)
            .with_context(|| format!("Failed to open DuckDB database at '{}'", self.db_path))?;

        info!("Installing and loading DuckDB's 'arrow' extension");

        // Try to install and load the arrow extension
        if let Err(e) = conn.execute("INSTALL arrow FROM community;", []) {
            warn!("Failed to install arrow extension: {}", e);
        }

        if let Err(e) = conn.execute("LOAD arrow;", []) {
            warn!("Failed to load arrow extension: {}", e);
        }

        // Check what arrow functions are available
        let check_functions_query = "SELECT function_name FROM duckdb_functions() WHERE function_name LIKE '%arrow%' OR function_name LIKE '%ipc%';";
        if let Ok(mut stmt) = conn.prepare(check_functions_query) {
            if let Ok(rows) = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?)) {
                info!("Available Arrow/IPC functions:");
                for row in rows {
                    if let Ok(function_name) = row {
                        info!("  - {}", function_name);
                    }
                }
            }
        }

        // Try different possible function names for reading IPC files
        let possible_functions = vec!["read_ipc", "arrow_scan", "read_arrow", "arrow_ipc_scan"];

        let mut success = false;
        for func_name in possible_functions {
            let query = format!(
                "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM {}(?);",
                self.table_name, func_name
            );

            info!("Trying function: {}", func_name);
            match conn.execute(&query, params![temp_file_path]) {
                Ok(_) => {
                    info!("Successfully loaded data using function: {}", func_name);
                    success = true;
                    break;
                }
                Err(e) => {
                    warn!("Function {} failed: {}", func_name, e);
                    continue;
                }
            }
        }

        if !success {
            return Err(anyhow::anyhow!(
                "None of the Arrow IPC functions worked. Consider using Parquet format instead."
            ));
        }

        info!(
            table = %self.table_name,
            count = df.height(),
            "Successfully loaded data into DuckDB table."
        );

        Ok(())
    }
}

