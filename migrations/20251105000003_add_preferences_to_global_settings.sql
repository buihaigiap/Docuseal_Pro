-- Add preference columns to global_settings table
ALTER TABLE global_settings ADD COLUMN force_2fa_with_authenticator_app BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN add_signature_id_to_the_documents BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN require_signing_reason BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN allow_typed_text_signatures BOOLEAN DEFAULT TRUE;
ALTER TABLE global_settings ADD COLUMN allow_to_resubmit_completed_forms BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN allow_to_decline_documents BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN remember_and_pre_fill_signatures BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN require_authentication_for_file_download_links BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN combine_completed_documents_and_audit_log BOOLEAN DEFAULT FALSE;
ALTER TABLE global_settings ADD COLUMN expirable_file_download_links BOOLEAN DEFAULT FALSE;