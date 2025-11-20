import { useState } from 'react';
import {
  Typography,
  Box,
  Switch
} from '@mui/material';
import upstashService from '../../ConfigApi/upstashService';
import toast from 'react-hot-toast';

// cấu hình các toggle field
const preferenceFields = [
  { key: 'force2fa', label: 'Force 2FA with Authenticator App' },
  { key: 'addSignatureId', label: 'Add signature ID to the documents' },
  { key: 'requireSigningReason', label: 'Require signing reason' },
  { key: 'allowTypedTextSignatures', label: 'Allow typed text signatures' },
  { key: 'allowResubmitCompletedForms', label: 'Allow to resubmit completed forms' },
  { key: 'allowDeclineDocuments', label: 'Allow to decline documents' },
  { key: 'rememberPrefillSignatures', label: 'Remember and pre-fill signatures' },
  { key: 'requireAuthForDownload', label: 'Require authentication for file download links' },
  { key: 'combineCompletedAudit', label: 'Combine completed documents and Audit Log' },
  { key: 'expirableDownloadLinks', label: 'Expirable file download links' }
];

interface PreferencesSectionProps {
  initialPreferences: {
    force2fa: boolean;
    addSignatureId: boolean;
    requireSigningReason: boolean;
    allowTypedTextSignatures: boolean;
    allowResubmitCompletedForms: boolean;
    allowDeclineDocuments: boolean;
    rememberPrefillSignatures: boolean;
    requireAuthForDownload: boolean;
    combineCompletedAudit: boolean;
    expirableDownloadLinks: boolean;
  };
}

export default function PreferencesSection({ initialPreferences }: PreferencesSectionProps) {
  const [preferences, setPreferences] = useState(initialPreferences);

  const handlePreferenceChange = async (key: string, newValue: boolean, label: string) => {
    // Update local state immediately for better UX
    setPreferences((prev) => ({
      ...prev,
      [key]: newValue
    }));

    // Auto-save the preference using user settings API
    try {
      console.log('Updating user setting:', {
        [key === 'force2fa' ? 'force_2fa_with_authenticator_app' :
         key === 'addSignatureId' ? 'add_signature_id_to_the_documents' :
         key === 'requireSigningReason' ? 'require_signing_reason' :
         key === 'allowTypedTextSignatures' ? 'allow_typed_text_signatures' :
         key === 'allowResubmitCompletedForms' ? 'allow_to_resubmit_completed_forms' :
         key === 'allowDeclineDocuments' ? 'allow_to_decline_documents' :
         key === 'rememberPrefillSignatures' ? 'remember_and_pre_fill_signatures' :
         key === 'requireAuthForDownload' ? 'require_authentication_for_file_download_links' :
         key === 'combineCompletedAudit' ? 'combine_completed_documents_and_audit_log' :
         'expirable_file_download_links']: newValue
      });

      await upstashService.updateUserSettings({
        [key === 'force2fa' ? 'force_2fa_with_authenticator_app' :
         key === 'addSignatureId' ? 'add_signature_id_to_the_documents' :
         key === 'requireSigningReason' ? 'require_signing_reason' :
         key === 'allowTypedTextSignatures' ? 'allow_typed_text_signatures' :
         key === 'allowResubmitCompletedForms' ? 'allow_to_resubmit_completed_forms' :
         key === 'allowDeclineDocuments' ? 'allow_to_decline_documents' :
         key === 'rememberPrefillSignatures' ? 'remember_and_pre_fill_signatures' :
         key === 'requireAuthForDownload' ? 'require_authentication_for_file_download_links' :
         key === 'combineCompletedAudit' ? 'combine_completed_documents_and_audit_log' :
         'expirable_file_download_links']: newValue
      });

      console.log('User setting updated successfully');
      toast.success(`${label} updated`);
    } catch (err) {
      console.error('Failed to update user setting:', err);
      toast.error(`Failed to update ${label}`);
      // Revert the change on error
      setPreferences((prev) => ({
        ...prev,
        [key]: !newValue
      }));
    }
  };

  return (
    <div className="bg-white/5 border border-white/10 rounded-lg p-4 mb-4">
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="h6">Preferences</Typography>
      </Box>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
        {preferenceFields.map(({ key, label }) => (
          <Box
            key={key}
            sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}
          >
            <Typography>{label}</Typography>
            <Switch
              checked={preferences[key as keyof typeof preferences]}
              onChange={(e) => handlePreferenceChange(key, e.target.checked, label)}
            />
          </Box>
        ))}
      </Box>
    </div>
  );
}