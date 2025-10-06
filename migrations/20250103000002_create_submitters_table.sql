-- Create submitters table (simplified - combined submissions and submitters)
CREATE TABLE IF NOT EXISTS submitters (
    id BIGSERIAL PRIMARY KEY,
    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    signed_at TIMESTAMP WITH TIME ZONE,
    token VARCHAR(255) UNIQUE NOT NULL,
    bulk_signatures JSONB, -- Store multiple signatures as JSON array
    ip_address TEXT, -- IP address of signer
    user_agent TEXT, -- User agent of signer
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_submitters_template_id ON submitters(template_id);
CREATE INDEX IF NOT EXISTS idx_submitters_user_id ON submitters(user_id);
CREATE INDEX IF NOT EXISTS idx_submitters_token ON submitters(token);
CREATE INDEX IF NOT EXISTS idx_submitters_email ON submitters(email);
