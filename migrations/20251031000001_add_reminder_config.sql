-- Add reminder configuration columns to submitters table
ALTER TABLE submitters 
ADD COLUMN IF NOT EXISTS reminder_config JSONB,
ADD COLUMN IF NOT EXISTS last_reminder_sent_at TIMESTAMP WITH TIME ZONE,
ADD COLUMN IF NOT EXISTS reminder_count INTEGER DEFAULT 0;

-- Create index for efficient reminder queue processing
CREATE INDEX IF NOT EXISTS idx_submitters_reminder_queue 
ON submitters(status, last_reminder_sent_at, created_at) 
WHERE reminder_config IS NOT NULL AND status = 'pending';

-- Add comments for documentation
COMMENT ON COLUMN submitters.reminder_config IS 'JSON configuration for automatic reminders (first_reminder_hours, second_reminder_hours, third_reminder_hours)';
COMMENT ON COLUMN submitters.last_reminder_sent_at IS 'Timestamp of the last reminder sent to this submitter';
COMMENT ON COLUMN submitters.reminder_count IS 'Number of reminders sent (0-3)';
