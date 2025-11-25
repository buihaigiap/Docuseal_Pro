-- Migration: Create Team Accounts System
-- This migration adds the accounts table and links users to accounts

-- Create accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index for accounts
CREATE INDEX IF NOT EXISTS idx_accounts_slug ON accounts(slug);

-- Add account_id to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS archived_at TIMESTAMP WITH TIME ZONE;

-- Create index for users account_id
CREATE INDEX IF NOT EXISTS idx_users_account_id ON users(account_id);
CREATE INDEX IF NOT EXISTS idx_users_archived_at ON users(archived_at);

-- Create account_linked_accounts for testing/linking accounts
CREATE TABLE IF NOT EXISTS account_linked_accounts (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    linked_account_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, linked_account_id)
);

-- Create indexes for account_linked_accounts
CREATE INDEX IF NOT EXISTS idx_account_linked_accounts_account_id ON account_linked_accounts(account_id);
CREATE INDEX IF NOT EXISTS idx_account_linked_accounts_linked_account_id ON account_linked_accounts(linked_account_id);

-- Update user_invitations to include account_id
ALTER TABLE user_invitations ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;
ALTER TABLE user_invitations ADD COLUMN IF NOT EXISTS token VARCHAR(255) UNIQUE;

-- Create index for user_invitations account_id and token
CREATE INDEX IF NOT EXISTS idx_user_invitations_account_id ON user_invitations(account_id);
CREATE INDEX IF NOT EXISTS idx_user_invitations_token ON user_invitations(token);

-- Drop the unique constraint on email for user_invitations (allow multiple invitations to same email for different accounts)
ALTER TABLE user_invitations DROP CONSTRAINT IF EXISTS user_invitations_email_key;

-- Add composite unique constraint
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_invitations_email_account ON user_invitations(email, account_id) WHERE is_used = FALSE;

-- Update templates to reference account instead of user
ALTER TABLE templates ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_templates_account_id ON templates(account_id);

-- Update template_folders to reference account instead of user
ALTER TABLE template_folders ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_template_folders_account_id ON template_folders(account_id);

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_accounts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for accounts table
DROP TRIGGER IF EXISTS trigger_update_accounts_updated_at ON accounts;
CREATE TRIGGER trigger_update_accounts_updated_at
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_accounts_updated_at();
