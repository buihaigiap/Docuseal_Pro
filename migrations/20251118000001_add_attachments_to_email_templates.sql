-- Add attachment options to email templates table
ALTER TABLE email_templates
ADD COLUMN attach_documents BOOLEAN DEFAULT FALSE,
ADD COLUMN attach_audit_log BOOLEAN DEFAULT FALSE;