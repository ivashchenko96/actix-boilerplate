-- Create roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    permissions TEXT[] DEFAULT '{}',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create user_roles junction table
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, role_id)
);

-- Create indexes
CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_roles_is_system ON roles(is_system);
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_expires_at ON user_roles(expires_at);

-- Add updated_at trigger to roles
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default roles
INSERT INTO roles (name, display_name, description, permissions, is_system) VALUES
('admin', 'Administrator', 'System administrator with full access', 
 ARRAY['*'], TRUE),
('user', 'User', 'Regular user with basic permissions', 
 ARRAY['profile:read', 'profile:write'], TRUE),
('moderator', 'Moderator', 'Content moderator with elevated permissions',
 ARRAY['users:read', 'users:moderate', 'content:moderate'], TRUE);