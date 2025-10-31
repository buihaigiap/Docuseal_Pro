-- Create user_reminder_settings table to store per-user default reminder configurations
CREATE TABLE IF NOT EXISTS user_reminder_settings (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    first_reminder_hours INTEGER,  -- NULL by default, user must set to enable reminders
    second_reminder_hours INTEGER,  -- NULL by default, user must set to enable reminders
    third_reminder_hours INTEGER,  -- NULL by default, user must set to enable reminders
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- Create index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_reminder_settings_user_id ON user_reminder_settings(user_id);

-- Add comments for documentation
COMMENT ON TABLE user_reminder_settings IS 'Default reminder configuration for each user. When all 3 hours are set (non-NULL), reminders are automatically enabled for new submissions.';
COMMENT ON COLUMN user_reminder_settings.user_id IS 'Foreign key to users table';
COMMENT ON COLUMN user_reminder_settings.first_reminder_hours IS 'Hours after creation to send first reminder (NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.second_reminder_hours IS 'Hours after creation to send second reminder (NULL = not configured)';
COMMENT ON COLUMN user_reminder_settings.third_reminder_hours IS 'Hours after creation to send third reminder (NULL = not configured)';