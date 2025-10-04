-- Create submissions table-- Create submissions table

CREATE TABLE IF NOT EXISTS submissions (CREATE TABLE IF NOT EXISTS submissions (

    id BIGSERIAL PRIMARY KEY,    id BIGSERIAL PRIMARY KEY,

    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,    template_id BIGINT NOT NULL REFERENCES templates(id) ON DELETE CASCADE,

    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    status VARCHAR(50) NOT NULL DEFAULT 'pending',    status VARCHAR(50) NOT NULL DEFAULT 'pending',

    documents JSONB,    documents JSONB,

    submitters JSONB,    submitters JSONB,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    expires_at TIMESTAMP WITH TIME ZONE    expires_at TIMESTAMP WITH TIME ZONE

););



-- Create submitters table-- Create submitters table

CREATE TABLE IF NOT EXISTS submitters (CREATE TABLE IF NOT EXISTS submitters (

    id BIGSERIAL PRIMARY KEY,    id BIGSERIAL PRIMARY KEY,

    submission_id BIGINT NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,    submission_id BIGINT NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,    name VARCHAR(255) NOT NULL,

    email VARCHAR(255) NOT NULL,    email VARCHAR(255) NOT NULL,

    status VARCHAR(50) NOT NULL DEFAULT 'pending',    status VARCHAR(50) NOT NULL DEFAULT 'pending',

    signed_at TIMESTAMP WITH TIME ZONE,    signed_at TIMESTAMP WITH TIME ZONE,

    token VARCHAR(255) UNIQUE NOT NULL,    token VARCHAR(255) UNIQUE NOT NULL,

    fields_data JSONB,    fields_data JSONB,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP

););



-- Create indexes for better performance-- Create indexes for better performance

CREATE INDEX IF NOT EXISTS idx_submissions_user_id ON submissions(user_id);CREATE INDEX IF NOT EXISTS idx_submissions_user_id ON submissions(user_id);

CREATE INDEX IF NOT EXISTS idx_submissions_template_id ON submissions(template_id);CREATE INDEX IF NOT EXISTS idx_submissions_template_id ON submissions(template_id);

CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(status);CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(status);

CREATE INDEX IF NOT EXISTS idx_submissions_created_at ON submissions(created_at);CREATE INDEX IF NOT EXISTS idx_submissions_created_at ON submissions(created_at);

CREATE INDEX IF NOT EXISTS idx_submitters_submission_id ON submitters(submission_id);CREATE INDEX IF NOT EXISTS idx_submitters_submission_id ON submitters(submission_id);

CREATE INDEX IF NOT EXISTS idx_submitters_token ON submitters(token);CREATE INDEX IF NOT EXISTS idx_submitters_token ON submitters(token);

CREATE INDEX IF NOT EXISTS idx_submitters_email ON submitters(email);CREATE INDEX IF NOT EXISTS idx_submitters_email ON submitters(email);
