-- Combined migration for template_fields table setup and enhancement-- Combined migration for template_fields table setup and enhancement

-- Includes: table creation and column additions-- Includes: table creation and column additions



-- Create template_fields table to store field definitions separately from templates-- From 20251003152945_create_template_fields_table.sql

CREATE TABLE IF NOT EXISTS template_fields (-- Create template_fields table to store field definitions separately from templates

    id BIGSERIAL PRIMARY KEY,CREATE TABLE IF NOT EXISTS template_fields (

    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,    id BIGSERIAL PRIMARY KEY,

    name VARCHAR(255) NOT NULL,    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,

    field_type VARCHAR(50) NOT NULL,    name VARCHAR(255) NOT NULL,

    required BOOLEAN DEFAULT FALSE,    field_type VARCHAR(50) NOT NULL,

    display_order INTEGER DEFAULT 0,    required BOOLEAN DEFAULT FALSE,

    position JSONB, -- Field position data    display_order INTEGER DEFAULT 0,

    options JSONB, -- Options for select/radio fields    position JSONB, -- Field position data

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    options JSONB, -- Options for select/radio fields

    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(template_id, name) -- Ensure unique field names per template    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

);    UNIQUE(template_id, name) -- Ensure unique field names per template

);

-- Create indexes for better performance

CREATE INDEX IF NOT EXISTS idx_template_fields_template_id ON template_fields(template_id);-- Create indexes for better performance

CREATE INDEX IF NOT EXISTS idx_template_fields_name ON template_fields(name);CREATE INDEX IF NOT EXISTS idx_template_fields_template_id ON template_fields(template_id);

CREATE INDEX IF NOT EXISTS idx_template_fields_created_at ON template_fields(created_at);CREATE INDEX IF NOT EXISTS idx_template_fields_name ON template_fields(name);

CREATE INDEX IF NOT EXISTS idx_template_fields_created_at ON template_fields(created_at);

-- Add metadata column

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS metadata JSONB;-- From 20251004000001_refactor_fields_to_separate_table.sql

-- Migration: Add metadata and position columns to template_fields table

-- Add position columns

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_x FLOAT8;-- Add metadata column

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_y FLOAT8;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS metadata JSONB;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_width FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_height FLOAT8;-- Add position columns

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_page INTEGER;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_x FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_x FLOAT8;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_y FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_y FLOAT8;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_width FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_width FLOAT8;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_height FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_height FLOAT8;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS position_page INTEGER;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_page INTEGER;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_x FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS allow_custom_position BOOLEAN DEFAULT true;ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_y FLOAT8;

ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_width FLOAT8;

-- Create additional indexes for performanceALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_height FLOAT8;

CREATE INDEX IF NOT EXISTS idx_template_fields_field_type ON template_fields(field_type);ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS suggested_page INTEGER;

CREATE INDEX IF NOT EXISTS idx_template_fields_display_order ON template_fields(template_id, display_order);ALTER TABLE template_fields ADD COLUMN IF NOT EXISTS allow_custom_position BOOLEAN DEFAULT true;



-- Create function trigger to auto-update updated_at-- Create additional indexes for performance

CREATE OR REPLACE FUNCTION update_template_fields_updated_at()CREATE INDEX IF NOT EXISTS idx_template_fields_field_type ON template_fields(field_type);

RETURNS TRIGGER AS $$CREATE INDEX IF NOT EXISTS idx_template_fields_display_order ON template_fields(template_id, display_order);

BEGIN

    NEW.updated_at = CURRENT_TIMESTAMP;-- Create function trigger to auto-update updated_at

    RETURN NEW;CREATE OR REPLACE FUNCTION update_template_fields_updated_at()

END;RETURNS TRIGGER AS $$

$$ LANGUAGE plpgsql;BEGIN

    NEW.updated_at = CURRENT_TIMESTAMP;

CREATE TRIGGER trigger_template_fields_updated_at    RETURN NEW;

    BEFORE UPDATE ON template_fieldsEND;

    FOR EACH ROW$$ LANGUAGE plpgsql;

    EXECUTE FUNCTION update_template_fields_updated_at();

CREATE TRIGGER trigger_template_fields_updated_at

-- Add comments for documentation    BEFORE UPDATE ON template_fields

COMMENT ON TABLE template_fields IS 'Bảng lưu trữ fields của templates, tách riêng để dễ quản lý và tái sử dụng';    FOR EACH ROW

COMMENT ON COLUMN template_fields.template_id IS 'Foreign key tới templates table';    EXECUTE FUNCTION update_template_fields_updated_at();

COMMENT ON COLUMN template_fields.field_type IS 'Loại field: text, signature, date, checkbox, select, radio, etc.';

COMMENT ON COLUMN template_fields.display_order IS 'Thứ tự hiển thị của field trong template';-- Add comments for documentation

COMMENT ON COLUMN template_fields.options IS 'Options cho select/radio fields dưới dạng JSON array';COMMENT ON TABLE template_fields IS 'Bảng lưu trữ fields của templates, tách riêng để dễ quản lý và tái sử dụng';

COMMENT ON COLUMN template_fields.metadata IS 'Metadata bổ sung, có thể mở rộng sau';COMMENT ON COLUMN template_fields.template_id IS 'Foreign key tới templates table';

COMMENT ON COLUMN template_fields.field_type IS 'Loại field: text, signature, date, checkbox, select, radio, etc.';
COMMENT ON COLUMN template_fields.display_order IS 'Thứ tự hiển thị của field trong template';
COMMENT ON COLUMN template_fields.options IS 'Options cho select/radio fields dưới dạng JSON array';
COMMENT ON COLUMN template_fields.metadata IS 'Metadata bổ sung, có thể mở rộng sau';
