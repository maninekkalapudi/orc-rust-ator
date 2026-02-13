-- Analytics schema for aggregated data
CREATE SCHEMA IF NOT EXISTS analytics;

-- Staging schema for raw data
CREATE SCHEMA IF NOT EXISTS staging;

-- Job results aggregation view
CREATE OR REPLACE VIEW analytics.job_summary AS
SELECT 
    job_id,
    COUNT(*) as execution_count,
    AVG(row_count) as avg_rows,
    MIN(created_at) as first_execution,
    MAX(created_at) as last_execution
FROM staging.job_metadata
GROUP BY job_id;