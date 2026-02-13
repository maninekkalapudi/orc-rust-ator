/*
 * File: src/logger.rs
 * Description: Provides global logging setup for the application.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added file header and documentation comments.
 */

//! Provides global logging setup for the application.
//! 
//! This module initializes the `tracing` subscriber with a custom formatter that includes
//! timestamps, elapsed time, log level, and file information. It ensures that logging
//! is configured consistently across the application.

// In src/logger.rs

use chrono::Local;
use std::fmt;
use std::time::Instant;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

// Global start time for elapsed calculations. It will be set ONCE in `initialize_logger`.
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

// Custom formatter struct (Unchanged)
pub struct CustomFormatter;

// Implementation of FormatEvent trait (Refined)
impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = event.metadata().level();
        let filename = event.metadata().file().unwrap_or("unknown_file");

        // --- REFINEMENT #1: Simplified time calculation ---
        // We can safely .unwrap() here because `initialize_logger` is guaranteed
        // to have set the value already. This removes the redundant `get_or_init`.
        let elapsed = START_TIME.get().unwrap().elapsed();

        // --- REFINEMENT #2: Simplified duration formatting ---
        // The default Debug format for Duration is excellent and replaces the if/else block.
        // It produces outputs like "2.15s", "500ms", "50μs".
        let elapsed_str = format!("{:?}", elapsed);

        // The final write is now cleaner
        write!(writer, "{timestamp}|{elapsed_str}|{level}|{filename}|")?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

// Logger initialization (Refined)
pub fn initialize_logger() {
    // --- REFINEMENT #1 (Source of truth) ---
    // This is now the ONLY place where START_TIME is set.
    // .set() returns an error if it's already set, which is fine. We ignore it.
    let _ = START_TIME.set(Instant::now());

    tracing_subscriber::fmt()
        .event_format(CustomFormatter)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init().ok();
}