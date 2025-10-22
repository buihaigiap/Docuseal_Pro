-- Add is_active and activation_token columns to users table
-- Migration: 20250123000001_add_activation_to_users.sql

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'is_active') THEN
        ALTER TABLE users ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'activation_token') THEN
        ALTER TABLE users ADD COLUMN activation_token TEXT;
    END IF;
END $$;