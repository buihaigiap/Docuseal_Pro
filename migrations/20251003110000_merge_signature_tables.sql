-- Add signature fields to signature_positions and drop signature_data table
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS signature_image TEXT;
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS signature_value TEXT;
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS signed_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS ip_address TEXT;
ALTER TABLE signature_positions ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- Migrate data from signature_data to signature_positions
INSERT INTO signature_positions (submitter_id, field_name, page, x, y, width, height, created_at, signature_image, signature_value, signed_at, ip_address, user_agent)
SELECT sp.submitter_id, sp.field_name, sp.page, sp.x, sp.y, sp.width, sp.height, sp.created_at, sd.signature_image, sd.signature_value, sd.signed_at, sd.ip_address, sd.user_agent
FROM signature_positions sp
LEFT JOIN signature_data sd ON sp.submitter_id = sd.submitter_id
ON CONFLICT DO NOTHING;

