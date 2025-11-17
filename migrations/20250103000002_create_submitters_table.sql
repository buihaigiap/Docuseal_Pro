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
    session_id VARCHAR(255), -- Session ID for tracking
    viewed_at TIMESTAMP WITH TIME ZONE, -- When form was first viewed
    timezone VARCHAR(100), -- User timezone
    reminder_config JSONB, -- JSON configuration for automatic reminders
    last_reminder_sent_at TIMESTAMP WITH TIME ZONE, -- Timestamp of the last reminder sent
    reminder_count INTEGER DEFAULT 0, -- Number of reminders sent (0-3)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_submitters_template_id ON submitters(template_id);
CREATE INDEX IF NOT EXISTS idx_submitters_user_id ON submitters(user_id);
CREATE INDEX IF NOT EXISTS idx_submitters_token ON submitters(token);
CREATE INDEX IF NOT EXISTS idx_submitters_email ON submitters(email);
CREATE INDEX IF NOT EXISTS idx_submitters_session_id ON submitters(session_id);
CREATE INDEX IF NOT EXISTS idx_submitters_reminder_queue ON submitters(status, last_reminder_sent_at, created_at) WHERE reminder_config IS NOT NULL AND status = 'pending';

-- Add comments for documentation
COMMENT ON COLUMN submitters.reminder_config IS 'JSON configuration for automatic reminders (first_reminder_hours, second_reminder_hours, third_reminder_hours)';
COMMENT ON COLUMN submitters.last_reminder_sent_at IS 'Timestamp of the last reminder sent to this submitter';
COMMENT ON COLUMN submitters.reminder_count IS 'Number of reminders sent (0-3)';
