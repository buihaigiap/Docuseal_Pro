-- Combined migration for template_fields table setup and enhancement
-- Includes: table creation and column additions

-- Create template_fields table to store field definitions separately from templates
CREATE TABLE IF NOT EXISTS template_fields (
    id BIGSERIAL PRIMARY KEY,
    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL,
    required BOOLEAN DEFAULT FALSE,
    display_order INTEGER DEFAULT 0,
    position JSONB, -- Field position data
    options JSONB, -- Options for select/radio fields
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(template_id, name) -- Ensure unique field names per template
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_template_fields_template_id ON template_fields(template_id);
CREATE INDEX IF NOT EXISTS idx_template_fields_name ON template_fields(name);
CREATE INDEX IF NOT EXISTS idx_template_fields_created_at ON template_fields(created_at);

-- Add metadata column
ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS metadata JSONB;

-- Note: Position columns were removed in later migration 20251005000012
-- They are no longer needed as we use the position JSONB column instead

-- Create additional indexes for performance
CREATE INDEX IF NOT EXISTS idx_template_fields_field_type ON template_fields(field_type);
CREATE INDEX IF NOT EXISTS idx_template_fields_display_order ON template_fields(template_id, display_order);

-- Create function trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_template_fields_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_template_fields_updated_at
    BEFORE UPDATE ON template_fields
    FOR EACH ROW
    EXECUTE FUNCTION update_template_fields_updated_at();

-- Add comments for documentation
COMMENT ON TABLE template_fields IS 'Bảng lưu trữ fields của templates, tách riêng để dễ quản lý và tái sử dụng';
COMMENT ON COLUMN template_fields.template_id IS 'Foreign key tới templates table';
COMMENT ON COLUMN template_fields.field_type IS 'Loại field: text, signature, date, checkbox, select, radio, etc.';
COMMENT ON COLUMN template_fields.display_order IS 'Thứ tự hiển thị của field trong template';
COMMENT ON COLUMN template_fields.options IS 'Options cho select/radio fields dưới dạng JSON array';
COMMENT ON COLUMN template_fields.metadata IS 'Metadata bổ sung, có thể mở rộng sau';
