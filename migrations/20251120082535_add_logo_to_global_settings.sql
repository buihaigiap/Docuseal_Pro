-- Add logo_url column to global_settings table
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS logo_url TEXT;