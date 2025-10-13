-- Add partner column to template_fields table
-- This column will store information about which partner/signer this field belongs to

ALTER TABLE template_fields 
ADD COLUMN IF NOT EXISTS partner VARCHAR(255);

-- Add index for better performance when querying by partner
CREATE INDEX IF NOT EXISTS idx_template_fields_partner ON template_fields(partner);

-- Add index for compound queries (template_id + partner)
CREATE INDEX IF NOT EXISTS idx_template_fields_template_partner ON template_fields(template_id, partner);

-- Add comment for documentation
COMMENT ON COLUMN template_fields.partner IS 'Tên của bên ký (partner/signer) mà field này thuộc về. Cho phép nhiều bên ký vào cùng một hợp đồng';