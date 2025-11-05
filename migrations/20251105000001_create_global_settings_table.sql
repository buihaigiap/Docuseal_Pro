-- Create global settings table for non-multi-tenant settings
CREATE TABLE global_settings (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1), -- Ensure only one row
    company_name TEXT,
    timezone TEXT,
    locale TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Insert default values
INSERT INTO global_settings (company_name, timezone, locale) VALUES ('Letmesign', 'UTC', 'en-US');