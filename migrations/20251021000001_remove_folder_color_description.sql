-- Remove color and description columns from template_folders table
ALTER TABLE template_folders DROP COLUMN IF EXISTS color;
ALTER TABLE template_folders DROP COLUMN IF EXISTS description;