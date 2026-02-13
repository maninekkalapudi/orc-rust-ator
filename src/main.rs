/*
 * File: src/main.rs
 * Description: Main entry point for the `orc-rust-ator` CLI tool.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added file header and documentation comments.
 */

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use orc_rust_ator::state::db::Db;
use orc_rust_ator::utils::seeder;
use std::env;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database with jobs from a file
    Init {
        #[arg(short, long, default_value = "jobs.yaml")]
        file: String,
    },
    /// Start the application server (default)
    Start,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { file }) => {
            println!("Initializing logger...");
            orc_rust_ator::logger::initialize_logger();
            
            println!("Opening database...");
            let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
            let db = Db::new(&database_url).await?;
            println!("Running migrations...");
            db.migrate().await?;
            
            println!("Seeding jobs from {}...", file);
            seeder::seed_jobs(&db, file).await?;
            println!("Init completed successfully.");
        }
        Some(Commands::Start) | None => {
            orc_rust_ator::run_app().await?;
        }
    }

    Ok(())
}