-- Add soft delete support for template_fields table
-- Migration: 20251008173006_add_soft_delete_to_template_fields

-- Add deleted_at column to template_fields table
ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create index for better performance on soft delete queries
CREATE INDEX IF NOT EXISTS idx_template_fields_deleted_at ON template_fields(deleted_at);

-- Add comment for documentation
COMMENT ON COLUMN template_fields.deleted_at IS 'Timestamp when the field was soft deleted (NULL means not deleted)';