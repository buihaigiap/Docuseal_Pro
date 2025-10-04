-- Add user_id to templates table for user-specific templates-- Add user_id to templates table for user-specific templates

DO $$DO $$

BEGINBEGIN

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'templates' AND column_name = 'user_id') THEN    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'templates' AND column_name = 'user_id') THEN

        ALTER TABLE templates ADD COLUMN user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE;        ALTER TABLE templates ADD COLUMN user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE;

    END IF;    END IF;

END $$;END $$;



-- Create index for better performance-- Create index for better performance

CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);


-- Update existing templates to belong to a default user (you may need to adjust this)
-- For now, we'll assume there's at least one user with id=1
-- UPDATE templates SET user_id = 1 WHERE user_id IS NULL;