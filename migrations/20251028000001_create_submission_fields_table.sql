-- Create submission_fields table to store field snapshots for each submission
CREATE TABLE IF NOT EXISTS submission_fields (
    id BIGSERIAL PRIMARY KEY,
    submitter_id BIGINT NOT NULL REFERENCES submitters(id) ON DELETE CASCADE,
    template_field_id BIGINT NOT NULL REFERENCES template_fields(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL,
    required BOOLEAN DEFAULT FALSE,
    display_order INTEGER DEFAULT 0,
    position JSONB, -- Field position data
    options JSONB, -- Options for select/radio fields
    metadata JSONB, -- Additional metadata
    partner VARCHAR(255), -- Partner/signer this field belongs to
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_submission_fields_submitter_id ON submission_fields(submitter_id);
CREATE INDEX IF NOT EXISTS idx_submission_fields_template_field_id ON submission_fields(template_field_id);