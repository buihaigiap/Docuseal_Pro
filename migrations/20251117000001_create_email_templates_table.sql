-- Create email templates table
CREATE TABLE email_templates (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    template_type TEXT NOT NULL, -- 'invitation', 'reminder', 'completion'
    subject TEXT NOT NULL,
    body TEXT NOT NULL, -- Combined body field (can be text or HTML)
    body_format TEXT NOT NULL DEFAULT 'text', -- 'text' or 'html'
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index on user_id for faster queries
CREATE INDEX idx_email_templates_user_id ON email_templates(user_id);
-- Create index on template_type for faster queries
CREATE INDEX idx_email_templates_type ON email_templates(template_type);