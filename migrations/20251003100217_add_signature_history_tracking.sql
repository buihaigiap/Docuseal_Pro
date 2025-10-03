-- Add version and is_active columns to signature_positions for signature history tracking
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS version INTEGER DEFAULT 1;
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT true;

-- Create index for better performance when querying signature history
CREATE INDEX IF NOT EXISTS idx_signature_positions_submitter_field_version ON signature_positions(submitter_id, field_name, version DESC);
CREATE INDEX IF NOT EXISTS idx_signature_positions_active ON signature_positions(is_active) WHERE is_active = true;

-- Update existing records to have version = 1 and is_active = true
UPDATE signature_positions SET version = 1, is_active = true WHERE version IS NULL;