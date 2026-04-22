-- Create audit_logs table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255),
    old_values JSONB,
    new_values JSONB,
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    session_id UUID,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for efficient querying
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX idx_audit_logs_request_id ON audit_logs(request_id);

-- Create composite index for common queries
CREATE INDEX idx_audit_logs_user_resource ON audit_logs(user_id, resource_type, timestamp DESC);
CREATE INDEX idx_audit_logs_resource_action ON audit_logs(resource_type, action, timestamp DESC);

-- Create function to automatically log user changes
CREATE OR REPLACE FUNCTION log_user_changes()
RETURNS TRIGGER AS $$
BEGIN
    -- Log INSERT operations
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (
            user_id, action, resource_type, resource_id, new_values
        ) VALUES (
            NEW.id, 'CREATE', 'USER', NEW.id::TEXT, to_jsonb(NEW)
        );
        RETURN NEW;
    END IF;

    -- Log UPDATE operations
    IF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (
            user_id, action, resource_type, resource_id, old_values, new_values
        ) VALUES (
            NEW.id, 'UPDATE', 'USER', NEW.id::TEXT, to_jsonb(OLD), to_jsonb(NEW)
        );
        RETURN NEW;
    END IF;

    -- Log DELETE operations
    IF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (
            user_id, action, resource_type, resource_id, old_values
        ) VALUES (
            OLD.id, 'DELETE', 'USER', OLD.id::TEXT, to_jsonb(OLD)
        );
        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ language 'plpgsql';

-- Create trigger for user audit logging
CREATE TRIGGER user_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION log_user_changes();