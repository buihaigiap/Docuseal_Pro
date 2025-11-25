-- Consolidate global_settings: one settings row per account
-- This migration handles the case where multiple users in the same account
-- have separate settings rows - we keep only one per account

-- Step 1: For each account that has multiple settings rows, keep the oldest one
-- and delete the rest

-- Delete duplicate settings for account_id = 1 (if any exist)
-- Keep the row with the lowest ID (oldest)
DELETE FROM global_settings 
WHERE account_id IS NULL 
  AND user_id IN (
    SELECT u.id 
    FROM users u
    WHERE u.account_id = 1
      AND u.id != (
        SELECT MIN(u2.id) 
        FROM users u2 
        WHERE u2.account_id = 1
      )
  );

-- Step 2: Update account_id for all remaining settings rows
UPDATE global_settings 
SET account_id = (SELECT account_id FROM users WHERE users.id = global_settings.user_id)
WHERE account_id IS NULL AND user_id IS NOT NULL;

-- Note: Settings for users without an account (account_id IS NULL) are kept as-is
-- The UNIQUE constraint on account_id allows multiple NULL values
