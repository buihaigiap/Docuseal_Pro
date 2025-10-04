-- Create templates table-- Create templates table

CREATE TABLE IF NOT EXISTS templates (CREATE TABLE IF NOT EXISTS templates (

    id BIGSERIAL PRIMARY KEY,    id BIGSERIAL PRIMARY KEY,

    name VARCHAR(255) NOT NULL,    name VARCHAR(255) NOT NULL,

    slug VARCHAR(255) UNIQUE NOT NULL,    slug VARCHAR(255) UNIQUE NOT NULL,

    fields JSONB, -- JSON array of field definitions    fields JSONB, -- JSON array of field definitions

    submitters JSONB, -- JSON array of submitter definitions    submitters JSONB, -- JSON array of submitter definitions

    documents JSONB, -- JSON array of document metadata    documents JSONB, -- JSON array of document metadata

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP

););



-- Create indexes for better performance-- Create indexes for better performance

CREATE INDEX IF NOT EXISTS idx_templates_slug ON templates(slug);CREATE INDEX IF NOT EXISTS idx_templates_slug ON templates(slug);

CREATE INDEX IF NOT EXISTS idx_templates_created_at ON templates(created_at);CREATE INDEX IF NOT EXISTS idx_templates_created_at ON templates(created_at);
