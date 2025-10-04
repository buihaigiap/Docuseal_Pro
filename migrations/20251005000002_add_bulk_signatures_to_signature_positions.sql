-- Add bulk_signatures column to signature_positions table-- Add bulk_signatures column to signature_positions table

-- This allows storing multiple signatures in one record without creating a separate table-- This allows storing multiple signatures in one record without creating a separate table



ALTER TABLE signature_positionsALTER TABLE signature_positions

    ADD COLUMN IF NOT EXISTS bulk_signatures JSONB NULL;    ADD COLUMN IF NOT EXISTS bulk_signatures JSONB NULL;



-- Add comment for documentation-- Add comment for documentation

COMMENT ON COLUMN signature_positions.bulk_signatures IS 'JSON array chứa nhiều signatures: [{field_id, field_name, signature_value}, ...]';COMMENT ON COLUMN signature_positions.bulk_signatures IS 'JSON array chứa nhiều signatures: [{field_id, field_name, signature_value}, ...]';



-- Create index for better query performance on bulk signatures-- Create index for better query performance on bulk signatures

CREATE INDEX IF NOT EXISTS idx_signature_positions_bulk_signatures ON signature_positions USING GIN (bulk_signatures)CREATE INDEX IF NOT EXISTS idx_signature_positions_bulk_signatures ON signature_positions USING GIN (bulk_signatures)

WHERE bulk_signatures IS NOT NULL;WHERE bulk_signatures IS NOT NULL;=
