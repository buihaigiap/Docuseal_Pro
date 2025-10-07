-- Create enum type for user roles
CREATE TYPE user_role AS ENUM ('admin', 'team_member', 'recipient');

-- Add role column to users table
ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'team_member';