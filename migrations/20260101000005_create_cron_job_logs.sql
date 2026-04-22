-- Create cron_job_logs table
CREATE TABLE cron_job_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_name VARCHAR(100) NOT NULL,
    job_id VARCHAR(255),
    status VARCHAR(20) NOT NULL CHECK (status IN ('PENDING', 'RUNNING', 'SUCCESS', 'FAILED', 'CANCELLED')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    output TEXT,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 0,
    next_retry_at TIMESTAMPTZ
);

-- Create cron_job_schedules table for job configuration
CREATE TABLE cron_job_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_name VARCHAR(100) UNIQUE NOT NULL,
    cron_expression VARCHAR(100) NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    description TEXT,
    max_retries INTEGER DEFAULT 3,
    retry_delay_seconds INTEGER DEFAULT 60,
    timeout_seconds INTEGER DEFAULT 3600,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_cron_job_logs_job_name ON cron_job_logs(job_name);
CREATE INDEX idx_cron_job_logs_status ON cron_job_logs(status);
CREATE INDEX idx_cron_job_logs_started_at ON cron_job_logs(started_at);
CREATE INDEX idx_cron_job_logs_completed_at ON cron_job_logs(completed_at);
CREATE INDEX idx_cron_job_logs_job_status ON cron_job_logs(job_name, status, started_at DESC);

CREATE INDEX idx_cron_job_schedules_job_name ON cron_job_schedules(job_name);
CREATE INDEX idx_cron_job_schedules_is_enabled ON cron_job_schedules(is_enabled);
CREATE INDEX idx_cron_job_schedules_next_run_at ON cron_job_schedules(next_run_at);

-- Add updated_at trigger to cron_job_schedules
CREATE TRIGGER update_cron_job_schedules_updated_at BEFORE UPDATE ON cron_job_schedules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default cron job schedules
INSERT INTO cron_job_schedules (job_name, cron_expression, description) VALUES
('cleanup_expired_tokens', '0 2 * * *', 'Clean up expired refresh tokens and blacklisted tokens daily at 2 AM'),
('sync_search_index', '0 */6 * * *', 'Sync search index with database every 6 hours'),
('send_digest_emails', '0 9 * * 1', 'Send weekly digest emails every Monday at 9 AM');

-- Create function to update job schedule after completion
CREATE OR REPLACE FUNCTION update_job_schedule_after_run()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'SUCCESS' AND OLD.status != 'SUCCESS' THEN
        UPDATE cron_job_schedules 
        SET last_run_at = NEW.completed_at
        WHERE job_name = NEW.job_name;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to update schedule after job completion
CREATE TRIGGER update_schedule_after_job_run
    AFTER UPDATE ON cron_job_logs
    FOR EACH ROW EXECUTE FUNCTION update_job_schedule_after_run();