-- Add account_id to templates table for team sharing
ALTER TABLE templates 
ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;

-- Migrate existing data: copy account_id from users
UPDATE templates t
SET account_id = u.account_id
FROM users u
WHERE t.user_id = u.id AND t.account_id IS NULL;

-- Create index for account_id queries
CREATE INDEX IF NOT EXISTS idx_templates_account_id ON templates(account_id);

-- Add account_id to template_folders table for team sharing
ALTER TABLE template_folders 
ADD COLUMN IF NOT EXISTS account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE;

-- Migrate existing folder data: copy account_id from users
UPDATE template_folders tf
SET account_id = u.account_id
FROM users u
WHERE tf.user_id = u.id AND tf.account_id IS NULL;

-- Create index for account_id queries on folders
CREATE INDEX IF NOT EXISTS idx_template_folders_account_id ON template_folders(account_id);
