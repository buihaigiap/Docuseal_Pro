-- Optimized migration: Add per-user settings to global_settings table
-- Combines all preference-related changes into one migration
-- Migration: 20251114000001_optimized_per_user_settings.sql

-- Drop the check constraint that ensures only one row
ALTER TABLE global_settings DROP CONSTRAINT IF EXISTS global_settings_id_check;

-- Add user_id column, nullable for global settings
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE CASCADE;

-- Add preference columns with NOT NULL constraints
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS force_2fa_with_authenticator_app BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS add_signature_id_to_the_documents BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS require_signing_reason BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS allow_typed_text_signatures BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS allow_to_resubmit_completed_forms BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS allow_to_decline_documents BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS remember_and_pre_fill_signatures BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS require_authentication_for_file_download_links BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS combine_completed_documents_and_audit_log BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN IF NOT EXISTS expirable_file_download_links BOOLEAN NOT NULL DEFAULT FALSE;

-- Create unique constraint on user_id (NULL allowed for global)
ALTER TABLE global_settings ADD CONSTRAINT unique_user_or_global UNIQUE (user_id);

-- The existing row (id=1) remains with user_id=NULL for global settings

-- Insert user-specific settings for each user, copying boolean values from global
INSERT INTO global_settings (
    user_id,
    force_2fa_with_authenticator_app,
    add_signature_id_to_the_documents,
    require_signing_reason,
    allow_typed_text_signatures,
    allow_to_resubmit_completed_forms,
    allow_to_decline_documents,
    remember_and_pre_fill_signatures,
    require_authentication_for_file_download_links,
    combine_completed_documents_and_audit_log,
    expirable_file_download_links
)
SELECT
    u.id,
    gs.force_2fa_with_authenticator_app,
    gs.add_signature_id_to_the_documents,
    gs.require_signing_reason,
    gs.allow_typed_text_signatures,
    gs.allow_to_resubmit_completed_forms,
    gs.allow_to_decline_documents,
    gs.remember_and_pre_fill_signatures,
    gs.require_authentication_for_file_download_links,
    gs.combine_completed_documents_and_audit_log,
    gs.expirable_file_download_links
FROM users u
CROSS JOIN global_settings gs
WHERE gs.user_id IS NULL;