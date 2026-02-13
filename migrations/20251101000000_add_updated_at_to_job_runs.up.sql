-- Add updated_at to job_runs
ALTER TABLE job_runs ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
