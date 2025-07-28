// src/job_def.rs

// We need to use the `Deserialize` trait from the serde library,
// which allows us to parse data from formats like YAML into these structs.
use serde::Deserialize;

// We are reusing the ExtractorConfig and LoaderConfig enums
// that we already defined in our `config.rs` module. This avoids
// duplicating code and ensures consistency.
use crate::config::{ExtractorConfig, LoaderConfig};

/// Represents the structure of a single job definition in the `jobs.yaml` file.
/// The `#[derive(Deserialize)]` macro automatically generates the code needed
/// by `serde` to create this struct from a YAML object.
#[derive(Debug, Deserialize)]
pub struct JobDefinition {
    pub job_id: String,
    pub description: String,
    pub schedule: String,
    pub is_active: bool,
    // The `tasks` field in the YAML is a list of task objects.
    // In Rust, this corresponds to a Vec<TaskDefinition>.
    pub tasks: Vec<TaskDefinition>,
}

/// Represents a single task within a job's `tasks` list in `jobs.yaml`.
#[derive(Debug, Deserialize)]
pub struct TaskDefinition {
    pub task_id: String,
    pub task_order: i32,
    // These fields are themselves complex objects (enums in our case).
    // `serde` handles this nesting automatically because ExtractorConfig
    // and LoaderConfig also derive `Deserialize`.
    pub extractor_config: ExtractorConfig,
    pub loader_config: LoaderConfig,
}