-- Create template_folders table for organizing templates
CREATE TABLE IF NOT EXISTS template_folders (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_folder_id BIGINT NULL REFERENCES template_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes for template_folders performance
CREATE INDEX IF NOT EXISTS idx_template_folders_user_id ON template_folders(user_id);
CREATE INDEX IF NOT EXISTS idx_template_folders_parent_folder_id ON template_folders(parent_folder_id);

-- Add unique constraint to prevent duplicate folder names within the same parent folder and user
CREATE UNIQUE INDEX IF NOT EXISTS idx_template_folders_unique_name ON template_folders(user_id, parent_folder_id, name);

-- Create templates table
CREATE TABLE IF NOT EXISTS templates (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    documents JSONB, -- JSON array of document metadata
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    folder_id BIGINT NULL REFERENCES template_folders(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_templates_slug ON templates(slug);
CREATE INDEX IF NOT EXISTS idx_templates_created_at ON templates(created_at);
CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);
CREATE INDEX IF NOT EXISTS idx_templates_folder_id ON templates(folder_id);

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
    metadata JSONB, -- Additional metadata
    partner VARCHAR(255), -- Partner/signer this field belongs to
    deleted_at TIMESTAMP WITH TIME ZONE, -- Soft delete timestamp
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for template_fields performance
CREATE INDEX IF NOT EXISTS idx_template_fields_template_id ON template_fields(template_id);
CREATE INDEX IF NOT EXISTS idx_template_fields_name ON template_fields(name);
CREATE INDEX IF NOT EXISTS idx_template_fields_created_at ON template_fields(created_at);
CREATE INDEX IF NOT EXISTS idx_template_fields_field_type ON template_fields(field_type);
CREATE INDEX IF NOT EXISTS idx_template_fields_display_order ON template_fields(template_id, display_order);
CREATE INDEX IF NOT EXISTS idx_template_fields_partner ON template_fields(partner);
CREATE INDEX IF NOT EXISTS idx_template_fields_template_partner ON template_fields(template_id, partner);
CREATE INDEX IF NOT EXISTS idx_template_fields_deleted_at ON template_fields(deleted_at);

-- Create function trigger to auto-update updated_at for template_fields
CREATE OR REPLACE FUNCTION update_template_fields_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_template_fields_updated_at ON template_fields;
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
COMMENT ON COLUMN template_fields.partner IS 'Tên của bên ký (partner/signer) mà field này thuộc về. Cho phép nhiều bên ký vào cùng một hợp đồng';
COMMENT ON COLUMN template_fields.deleted_at IS 'Timestamp when the field was soft deleted (NULL means not deleted)';
