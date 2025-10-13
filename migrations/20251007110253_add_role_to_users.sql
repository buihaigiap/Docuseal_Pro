-- Create enum type for user roles
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('admin', 'team_member', 'recipient');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Add role column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS role user_role NOT NULL DEFAULT 'team_member';