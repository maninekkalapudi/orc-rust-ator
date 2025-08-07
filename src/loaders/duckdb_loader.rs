// In src/loaders/duckdb_loader.rs

use crate::pipelines::Loader;
use anyhow::{Context, Result};
use async_trait::async_trait;
use duckdb::{params, Connection};
use polars::prelude::*;
use std::fs::File;
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};

pub struct DuckDBLoader {
    pub db_path: String,
    pub table_name: String,
}

impl DuckDBLoader {
    /// Convenience constructor.
    pub fn new(db_path: impl Into<String>, table_name: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl Loader for DuckDBLoader {
    async fn load(&self, mut df: DataFrame) -> Result<()> {
        let df_height = df.height();
        info!(
            path = %self.db_path,
            table = %self.table_name,
            rows = df_height,
            "Preparing DuckDB load via IPC bridge."
        );

        // --- THE FIX IS HERE ---
        // We need to move all blocking operations off the async runtime.
        // First, clone the necessary `self` data so it can be moved into the thread.
        let db_path_clone = self.db_path.clone();
        let table_name_clone = self.table_name.clone();

        // `spawn_blocking` moves the entire synchronous workload to a dedicated thread pool.
        tokio::task::spawn_blocking(move || -> Result<()> {
            // --- This entire closure runs on a blocking-safe thread ---

            // Step 1: Create a temporary file to act as the bridge.
            // `NamedTempFile` ensures the file is cleaned up automatically when it's dropped.
            let temp_file = NamedTempFile::new()
                .context("Failed to create a temporary file for the IPC bridge")?;
            let temp_path = temp_file
                .path()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Temporary file path is not valid UTF-8"))?;

            debug!(temp_path, "Created temporary IPC file.");

            // Step 2: Write the Polars DataFrame to the temp file in IPC format.
            let file_handle = File::create(temp_path).with_context(|| {
                format!("Failed to create file handle for temp path: {}", temp_path)
            })?;
            
            IpcStreamWriter::new(file_handle)
                .finish(&mut df) // `df` is now fully written to disk.
                .context("Failed to write DataFrame to temporary IPC stream file")?;

            debug!("DataFrame written to IPC file successfully.");

            // Step 3: Open the DuckDB connection and load the data.
            let conn = Connection::open(&db_path_clone)
                .with_context(|| format!("Failed to open DuckDB database at '{}'", db_path_clone))?;

            // Step 4: Ensure the Arrow extension is available.
            info!("Ensuring DuckDB 'arrow' extension is available.");
            // We use `unwrap_or_else` with a warning because an error might just mean
            // the extension is already installed or built-in, which is not a failure.
            conn.execute("INSTALL arrow;", [])
                .unwrap_or_else(|e| {
                    warn!(error = %e, "Could not install 'arrow' extension (it may already be installed)");
                    0
                });
            conn.execute("LOAD arrow;", [])
                 .unwrap_or_else(|e| {
                    warn!(error = %e, "Could not load 'arrow' extension (it may already be loaded)");
                    0
                });

            // Step 5: Detect the correct function and load the data.
            let possible_functions = ["read_ipc", "arrow_scan", "read_arrow", "arrow_ipc_scan"];
            let mut success = false;
            for func_name in possible_functions {
                let query = format!(
                    "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM {}(?);",
                    table_name_clone, func_name
                );
                
                debug!(function = func_name, "Attempting to load data with function.");
                match conn.execute(&query, params![temp_path]) {
                    Ok(_) => {
                        info!("Successfully loaded data using function '{}'.", func_name);
                        success = true;
                        break;
                    }
                    Err(e) => {
                        debug!(function = func_name, error = %e, "Function failed, trying next.");
                        continue;
                    }
                }
            }

            if success {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Failed to load data into DuckDB. None of the attempted Arrow/IPC functions ({:?}) succeeded. Check DuckDB version and 'arrow' extension.",
                    possible_functions
                ))
            }
        })
        .await? // Wait for the blocking task to complete. Propagates panics.
        ?; // Propagate the `Result<()>` from inside the closure.

        info!(
            table = %self.table_name,
            rows = df_height,
            "Successfully loaded data into DuckDB table."
        );

        Ok(())
    }
}
