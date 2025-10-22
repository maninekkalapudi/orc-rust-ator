use anyhow::{Context, Result};
use async_trait::async_trait;
use duckdb::Connection;
use polars::prelude::*;
use polars::prelude::CsvWriter;
use std::io::{Cursor, Write};
use tempfile::NamedTempFile;
use tracing::{debug, info};

use crate::plugins::Loader;

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
            "Preparing DuckDB load via CSV bridge."
        );

        let db_path_clone = self.db_path.clone();
        let table_name_clone = self.table_name.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            // 1. Convert Polars DataFrame to CSV string
            let mut buf = Cursor::new(Vec::new());
            CsvWriter::new(&mut buf)
                .include_header(true)
                .finish(&mut df)
                .context("Failed to write DataFrame to CSV string")?;
            let csv_string = String::from_utf8(buf.into_inner())
                .context("Failed to convert CSV bytes to UTF-8 string")?;

            // 2. Create a temporary file and write the CSV data to it
                        let mut temp_file = NamedTempFile::new()
                            .context("Failed to create a temporary file for CSV bridge")?;
                        temp_file.write_all(csv_string.as_bytes())
                            .context("Failed to write CSV string to temporary file")?;
                        temp_file.flush()
                            .context("Failed to flush temporary file")?;
            
                        let temp_path_obj = temp_file.into_temp_path(); // Consume NamedTempFile
                        let file_path = temp_path_obj
                            .to_str()
                            .ok_or_else(|| anyhow::anyhow!("Temporary file path is not valid UTF-8"))?.to_string();

            debug!(file_path, "Created temporary IPC file.");

            // 3. Open the DuckDB connection and load the data.
            let conn = Connection::open(&db_path_clone)
                .with_context(|| format!("Failed to open DuckDB database at '{}'", db_path_clone))?;

            // 4. Construct and execute the SQL query to read the CSV
            let escaped_file_path = file_path.replace('\\', "/");
            let query = format!(
                "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM read_csv('{}', HEADER=TRUE);",
                table_name_clone, escaped_file_path
            );
            
            info!("Loading data into DuckDB using read_csv.");
            match conn.execute(&query, []) {
                Ok(_) => info!("DuckDB read_csv query successful."),
                Err(e) => return Err(anyhow::anyhow!("Failed to execute DuckDB read_csv query: {}. Query: '{}'", e, query)),
            }

            Ok(())
        })
        .await? // Wait for the blocking task to complete. Propagates panics.
        ?;

        info!(
            table = %self.table_name,
            rows = df_height,
            "Successfully loaded data into DuckDB table."
        );

        Ok(())
    }
}
