-- Create user_reminder_settings table to store per-user default reminder configurations
CREATE TABLE IF NOT EXISTS user_reminder_settings (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    first_reminder_hours INTEGER DEFAULT 1,  -- TEST: Set to 1 minute for testing reminders
    second_reminder_hours INTEGER DEFAULT 2,  -- TEST: Set to 2 minutes for testing reminders
    third_reminder_hours INTEGER DEFAULT 3,  -- TEST: Set to 3 minutes for testing reminders
    receive_notification_on_completion BOOLEAN,  -- NULL by default, user must set to enable notifications
    completion_notification_email TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- Create index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_reminder_settings_user_id ON user_reminder_settings(user_id);

-- Add comments for documentation
COMMENT ON TABLE user_reminder_settings IS 'Default reminder configuration for each user. When all 3 minutes are set (non-NULL), reminders are automatically enabled for new submissions.';
COMMENT ON COLUMN user_reminder_settings.user_id IS 'Foreign key to users table';
COMMENT ON COLUMN user_reminder_settings.first_reminder_hours IS 'Minutes after creation to send first reminder (DEFAULT 1 minute for testing, NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.second_reminder_hours IS 'Minutes after creation to send second reminder (DEFAULT 2 minutes for testing, NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.third_reminder_hours IS 'Minutes after creation to send third reminder (DEFAULT 3 minutes for testing, NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.receive_notification_on_completion IS 'Whether to send notification to user when all signees have completed signing (NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.completion_notification_email IS 'Email address to send notifications when a submission is completed';