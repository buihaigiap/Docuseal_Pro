-- Allow duplicate field names in templates
-- Drop the unique constraint on (template_id, name) to allow multiple fields with the same name

ALTER TABLE template_fields DROP CONSTRAINT IF EXISTS template_fields_template_id_name_key;