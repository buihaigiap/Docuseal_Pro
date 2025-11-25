-- Add account_id to global_settings for multi-tenant support
-- Each account should have its own global settings

-- Add account_id column
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id);

-- Migrate existing data: copy account_id from users table
UPDATE global_settings 
SET account_id = (SELECT account_id FROM users WHERE users.id = global_settings.user_id)
WHERE account_id IS NULL AND user_id IS NOT NULL;

-- Drop the old unique constraint on user_id
ALTER TABLE global_settings DROP CONSTRAINT IF EXISTS unique_user_or_global;

-- Create new unique constraint on account_id (one settings per account)
ALTER TABLE global_settings ADD CONSTRAINT unique_account_settings UNIQUE (account_id);

-- Create index for faster lookups by account_id
CREATE INDEX IF NOT EXISTS idx_global_settings_account_id ON global_settings(account_id);

-- Note: We keep both user_id and account_id for now
-- user_id can be NULL for account-wide settings
-- account_id identifies which account the settings belong to
