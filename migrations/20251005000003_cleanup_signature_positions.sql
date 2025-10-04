-- Drop unused columns from signature_positions table
ALTER TABLE signature_positions 
DROP COLUMN IF EXISTS field_id,
DROP COLUMN IF EXISTS field_name,
DROP COLUMN IF EXISTS signature_value;

-- Add signature_image column if it doesn't exist
ALTER TABLE signature_positions 
ADD COLUMN IF NOT EXISTS signature_image TEXT;
