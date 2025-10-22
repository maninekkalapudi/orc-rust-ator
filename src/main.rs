// In src/main.rs

//! Main entry point for the `orc-rust-ator` application.
//! 
//! This module is responsible for initializing the application and calling the main
//! `run_app` function from the library.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    orc_rust_ator::run_app().await
}