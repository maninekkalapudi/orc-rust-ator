-- In src/state/schema.sql

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Table to store job definitions
CREATE TABLE job_definitions (
    job_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_name VARCHAR(255) NOT NULL,
    description TEXT,
    schedule VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table to store task definitions for each job
CREATE TABLE task_definitions (
    task_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id UUID NOT NULL REFERENCES job_definitions(job_id) ON DELETE CASCADE,
    task_order INT NOT NULL,
    extractor_config JSONB NOT NULL,
    loader_config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (job_id, task_order)
);

-- Table to store the history of job runs
CREATE TABLE job_runs (
    run_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id UUID NOT NULL REFERENCES job_definitions(job_id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL, -- e.g., 'queued', 'running', 'success', 'failed'
    triggered_by VARCHAR(50) NOT NULL, -- e.g., 'scheduled', 'manual'
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes to improve query performance
CREATE INDEX idx_job_definitions_is_active ON job_definitions(is_active);
CREATE INDEX idx_job_runs_status ON job_runs(status);
CREATE INDEX idx_job_runs_job_id ON job_runs(job_id);
CREATE INDEX idx_task_definitions_job_id ON task_definitions(job_id);
