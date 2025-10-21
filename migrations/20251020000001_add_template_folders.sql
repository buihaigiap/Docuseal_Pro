-- Add template folders support
-- Create template_folders table
CREATE TABLE IF NOT EXISTS template_folders (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_folder_id BIGINT NULL REFERENCES template_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add folder_id column to templates table if not exists
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'templates' AND column_name = 'folder_id') THEN
        ALTER TABLE templates ADD COLUMN folder_id BIGINT NULL REFERENCES template_folders(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_template_folders_user_id ON template_folders(user_id);
CREATE INDEX IF NOT EXISTS idx_template_folders_parent_folder_id ON template_folders(parent_folder_id);
CREATE INDEX IF NOT EXISTS idx_templates_folder_id ON templates(folder_id);

-- Add unique constraint to prevent duplicate folder names within the same parent folder and user
CREATE UNIQUE INDEX IF NOT EXISTS idx_template_folders_unique_name ON template_folders(user_id, parent_folder_id, name);