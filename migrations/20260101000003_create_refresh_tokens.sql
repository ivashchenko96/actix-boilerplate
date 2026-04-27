-- Create refresh_tokens table
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    jti UUID UNIQUE NOT NULL,
    parent_jti UUID REFERENCES refresh_tokens(jti) ON DELETE CASCADE,
    device_info JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create blacklisted_tokens table for JWT access tokens
CREATE TABLE blacklisted_tokens (
    jti UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    blacklisted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason VARCHAR(255)
);

-- Create indexes
CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_jti ON refresh_tokens(jti);
CREATE INDEX idx_refresh_tokens_parent_jti ON refresh_tokens(parent_jti);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_is_revoked ON refresh_tokens(is_revoked);
CREATE INDEX idx_refresh_tokens_created_at ON refresh_tokens(created_at);

CREATE INDEX idx_blacklisted_tokens_user_id ON blacklisted_tokens(user_id);
CREATE INDEX idx_blacklisted_tokens_expires_at ON blacklisted_tokens(expires_at);
CREATE INDEX idx_blacklisted_tokens_blacklisted_at ON blacklisted_tokens(blacklisted_at);

-- Create cleanup function for expired tokens
CREATE OR REPLACE FUNCTION cleanup_expired_tokens()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Delete expired refresh tokens
    DELETE FROM refresh_tokens WHERE expires_at < CURRENT_TIMESTAMP;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Delete expired blacklisted tokens
    DELETE FROM blacklisted_tokens WHERE expires_at < CURRENT_TIMESTAMP;
    
    RETURN deleted_count;
END;
$$ language 'plpgsql';