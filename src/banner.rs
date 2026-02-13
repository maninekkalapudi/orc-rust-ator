// In src/banner.rs

//! # Banner Generation
//! 
//! This module is responsible for generating and printing the project banner
//! to the console when the application starts.
//!
//! ## How it Works
//!
//! It uses the `figlet-rs` crate to generate ASCII art from the project name,
//! and the `colored` crate to add color to the output. The font file is
//! embedded directly into the binary at compile time using `include_str!`.
//!
//! ## Technical Details & Trade-offs
//! 
//! - **`include_str!`:** The `include_str!` macro is used to embed the font
//!   file in the binary. This makes the executable self-contained and avoids
//!   the need to distribute the font file separately. The trade-off is a
//!   slightly larger binary size.
//! - **`env!` Macro:** The `env!` macro is used to fetch the package version
//!   from `Cargo.toml` at compile time. This ensures that the version number
//!   in the banner is always in sync with the project's version.

use colored::*
use figlet_rs::{FIGfont, FIGure};

// Embed the font file directly into the binary at compile time.
const SLANT_FONT: &str = include_str!("../resources/slant.flf");

/// Holds metadata about the application.
struct AppInfo {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    author: &'static str,
}

/// Prints the project banner to the console.
pub fn print_banner() {
    let app_info = AppInfo {
        name: "ORC-RUST-ATOR",
        version: env!("CARGO_PKG_VERSION"), // Fetches version from Cargo.toml
        description: "A powerful orchestration tool written in Rust",
        author: "Built by Rahul J", // Customize this!
    };

    // Parse the font data.
    let font = FIGfont::from_content(SLANT_FONT).unwrap();

    // Create the ASCII art from the project name.
    let figure: FIGure = font.convert(app_info.name).unwrap();

    // Print the formatted banner.
    println!(
        "{}",
        "=========================================================".dimmed()
    );
    println!("{}", figure.to_string().bright_green());
    println!("{}", app_info.description.italic());
    println!("\n{}", format!("Version: {}", app_info.version).yellow());
    println!("{}", app_info.author.cyan());
    println!(
        "{}",
        "=========================================================".dimmed()
    );
    println!(); // Add a blank line for spacing
}
