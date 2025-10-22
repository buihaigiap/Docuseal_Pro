CREATE TABLE IF NOT EXISTS user_invitations (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    role user_role NOT NULL,
    activation_code VARCHAR(8) NOT NULL UNIQUE,
    invited_by_user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for quick lookups
CREATE INDEX IF NOT EXISTS idx_user_invitations_email ON user_invitations(email);
CREATE INDEX IF NOT EXISTS idx_user_invitations_code ON user_invitations(activation_code);
