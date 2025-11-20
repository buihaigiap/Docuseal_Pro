-- Add completion and redirect settings to global_settings table
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'global_settings' AND column_name = 'completion_title') THEN
        ALTER TABLE global_settings ADD COLUMN completion_title TEXT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'global_settings' AND column_name = 'completion_body') THEN
        ALTER TABLE global_settings ADD COLUMN completion_body TEXT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'global_settings' AND column_name = 'redirect_title') THEN
        ALTER TABLE global_settings ADD COLUMN redirect_title TEXT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'global_settings' AND column_name = 'redirect_url') THEN
        ALTER TABLE global_settings ADD COLUMN redirect_url TEXT;
    END IF;
END $$;