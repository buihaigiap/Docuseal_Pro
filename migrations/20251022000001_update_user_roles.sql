-- Update user_role enum to include all DocuSeal Pro roles
DO $$ BEGIN
    -- Add new roles to the enum
    ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'editor';
    ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'member';
    ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'agent';
    ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'viewer';
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;