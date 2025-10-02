-- Create signature_positions table
CREATE TABLE IF NOT EXISTS signature_positions (
    id BIGSERIAL PRIMARY KEY,
    submitter_id BIGINT NOT NULL REFERENCES submitters(id) ON DELETE CASCADE,
    field_name VARCHAR(255) NOT NULL,
    page INTEGER NOT NULL,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    width DOUBLE PRECISION NOT NULL,
    height DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create signature_data table
CREATE TABLE IF NOT EXISTS signature_data (
    id BIGSERIAL PRIMARY KEY,
    submitter_id BIGINT NOT NULL REFERENCES submitters(id) ON DELETE CASCADE,
    signature_image TEXT NOT NULL,
    signed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_signature_positions_submitter_id ON signature_positions(submitter_id);
CREATE INDEX IF NOT EXISTS idx_signature_positions_field_name ON signature_positions(field_name);
CREATE INDEX IF NOT EXISTS idx_signature_data_submitter_id ON signature_data(submitter_id);
CREATE INDEX IF NOT EXISTS idx_signature_data_signed_at ON signature_data(signed_at);