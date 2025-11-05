-- Add NOT NULL constraints to global settings boolean columns
-- Migration: 20251105000005_add_not_null_constraints.sql

ALTER TABLE global_settings
ALTER COLUMN force_2fa_with_authenticator_app SET NOT NULL,
ALTER COLUMN add_signature_id_to_the_documents SET NOT NULL,
ALTER COLUMN require_signing_reason SET NOT NULL,
ALTER COLUMN allow_typed_text_signatures SET NOT NULL,
ALTER COLUMN allow_to_resubmit_completed_forms SET NOT NULL,
ALTER COLUMN allow_to_decline_documents SET NOT NULL,
ALTER COLUMN remember_and_pre_fill_signatures SET NOT NULL,
ALTER COLUMN require_authentication_for_file_download_links SET NOT NULL,
ALTER COLUMN combine_completed_documents_and_audit_log SET NOT NULL,
ALTER COLUMN expirable_file_download_links SET NOT NULL;