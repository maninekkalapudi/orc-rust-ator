-- Table to track warehouse data locations
CREATE TABLE job_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES job_definitions(job_id),
    warehouse_table VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    row_count BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_job FOREIGN KEY (job_id) REFERENCES job_definitions(job_id) ON DELETE CASCADE
);

CREATE INDEX idx_job_results_job_id ON job_results(job_id);
CREATE INDEX idx_job_results_created_at ON job_results(created_at);