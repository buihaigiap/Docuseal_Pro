import { useState, useEffect } from 'react';
import {
  Typography,
  Box,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Switch
} from '@mui/material';
import upstashService from '../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import { useTranslation } from 'react-i18next';

const TIMEZONES = [ 
  "International Date Line West","Midway Island","American Samoa","Hawaii","Alaska","Pacific Time (US & Canada)","Tijuana","Mountain Time (US & Canada)","Arizona","Chihuahua","Mazatlan","Central Time (US & Canada)","Saskatchewan","Guadalajara","Mexico City","Monterrey","Central America","Eastern Time (US & Canada)","Indiana (East)","Bogota","Lima","Quito","Atlantic Time (Canada)","Caracas","La Paz","Santiago","Asuncion","Newfoundland","Brasilia","Buenos Aires","Montevideo","Georgetown","Puerto Rico","Greenland","Mid-Atlantic","Azores","Cape Verde Is.","Dublin","Edinburgh","Lisbon",
  "London","Casablanca","Monrovia","UTC","Belgrade","Bratislava","Budapest","Ljubljana","Prague","Sarajevo","Skopje","Warsaw","Zagreb","Brussels","Copenhagen","Madrid","Paris","Amsterdam","Berlin","Bern","Zurich","Rome","Stockholm","Vienna","West Central Africa","Bucharest","Cairo","Helsinki","Kyiv","Riga","Sofia","Tallinn","Vilnius","Athens","Istanbul","Minsk","Jerusalem","Harare","Pretoria","Kaliningrad","Moscow","St. Petersburg",
  "Volgograd","Samara","Kuwait","Riyadh","Nairobi","Baghdad","Tehran","Abu Dhabi","Muscat","Baku","Tbilisi","Yerevan","Kabul","Ekaterinburg","Islamabad","Karachi","Tashkent","Chennai","Kolkata","Mumbai","New Delhi","Kathmandu","Dhaka","Sri Jayawardenepura","Almaty","Astana","Novosibirsk","Rangoon","Bangkok","Hanoi","Jakarta","Krasnoyarsk","Beijing","Chongqing","Hong Kong","Urumqi","Kuala Lumpur","Singapore","Taipei","Perth","Irkutsk","Ulaanbaatar","Seoul","Osaka","Sapporo","Tokyo","Yakutsk","Darwin","Adelaide","Canberra","Melbourne","Sydney","Brisbane","Hobart","Vladivostok","Guam","Port Moresby","Magadan","Srednekolymsk","Solomon Is.","New Caledonia","Fiji","Kamchatka","Marshall Is.","Auckland","Wellington","Nuku'alofa","Tokelau Is.","Chatham Is.","Samoa"
];

const LOCALES = [
  { value: 'en-US', label: 'English (United States)' },
  { value: 'en-GB', label: 'English (United Kingdom)' },
  { value: 'fr-FR', label: 'Français' },
  { value: 'es-ES', label: 'Español' },
  { value: 'pt-PT', label: 'Português' },
  { value: 'de-DE', label: 'Deutsch' },
  { value: 'it-IT', label: 'Italiano' },
  { value: 'nl-NL', label: 'Nederlands' }
];

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

const GeneralSettings = () => {
  const { t, i18n } = useTranslation();
  const [companyName, setCompanyName] = useState('');
  const [timezone, setTimezone] = useState('');
  const [locale, setLocale] = useState('');
  const [fetchLoading, setFetchLoading] = useState(true);

  // ✅ gộp tất cả toggle vào 1 object
  const [preferences, setPreferences] = useState({
    force2fa: false,
    addSignatureId: false,
    requireSigningReason: false,
    allowTypedTextSignatures: false,
    allowResubmitCompletedForms: false,
    allowDeclineDocuments: false,
    rememberPrefillSignatures: false,
    requireAuthForDownload: false,
    combineCompletedAudit: false,
    expirableDownloadLinks: false
  });

  useEffect(() => {
    const fetchSettings = async () => {
      try {
        // Fetch user settings for both basic info and preferences
        const userResponse = await upstashService.getUserSettings();
        const userSettings = userResponse.data;
        
        // Set basic info from user settings
        setCompanyName(userSettings.company_name || '');
        setTimezone(userSettings.timezone || '');
        setLocale(userSettings.locale || '');
        
        // Set preferences from user settings
        setPreferences({
          force2fa: userSettings.force_2fa_with_authenticator_app || false,
          addSignatureId: userSettings.add_signature_id_to_the_documents || false,
          requireSigningReason: userSettings.require_signing_reason || false,
          allowTypedTextSignatures: userSettings.allow_typed_text_signatures || false,
          allowResubmitCompletedForms: userSettings.allow_to_resubmit_completed_forms || false,
          allowDeclineDocuments: userSettings.allow_to_decline_documents || false,
          rememberPrefillSignatures: userSettings.remember_and_pre_fill_signatures || false,
          requireAuthForDownload: userSettings.require_authentication_for_file_download_links || false,
          combineCompletedAudit: userSettings.combine_completed_documents_and_audit_log || false,
          expirableDownloadLinks: userSettings.expirable_file_download_links || false
        });
      } catch (err) {
        toast.error('Failed to fetch settings');
      } finally {
        setFetchLoading(false);
      }
    };
    fetchSettings();
  }, []);

  if (fetchLoading) {
    return <Typography>{t('common.loading')}</Typography>;
  }

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>
        {t('settings.general.title')}
      </Typography>

      <Box>
        {/* --- BASIC INFO --- */}
        <div className="bg-white/5 border border-white/10 rounded-lg p-4 mb-4">
          <Typography variant="h6" sx={{ mb: 2 }}>
            {t('settings.general.basicInfo')}
          </Typography>

          <TextField
            fullWidth
            label={t('settings.general.companyName')}
            value={companyName}
            onChange={async (e) => {
              const newValue = e.target.value;
              setCompanyName(newValue);
              // Auto-save company name to user settings
              try {
                await upstashService.updateUserSettings({
                  company_name: newValue
                });
                toast.success('Company name updated');
              } catch (err) {
                toast.error('Failed to update company name');
                // Revert on error
                setCompanyName(companyName);
              }
            }}
            sx={{ mb: 2 }}
          />

          <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
            <FormControl fullWidth>
              <InputLabel>{t('settings.general.timeZone')}</InputLabel>
              <Select
                value={timezone}
                label={t('settings.general.timeZone')}
                onChange={async (e) => {
                  const newValue = e.target.value;
                  setTimezone(newValue);
                  // Auto-save timezone to user settings
                  try {
                    await upstashService.updateUserSettings({
                      timezone: newValue
                    });
                    toast.success('Timezone updated');
                  } catch (err) {
                    toast.error('Failed to update timezone');
                    // Revert on error
                    setTimezone(timezone);
                  }
                }}
              >
                {TIMEZONES.map((tz) => (
                  <MenuItem key={tz} value={tz}>{tz}</MenuItem>
                ))}
              </Select>
            </FormControl>

            <FormControl fullWidth>
              <InputLabel>{t('settings.general.language')}</InputLabel>
              <Select
                value={locale}
                label={t('settings.general.language')}
                onChange={async (e) => {
                  const newValue = e.target.value;
                  setLocale(newValue);
                  // Change language immediately
                  i18n.changeLanguage(newValue);
                  // Auto-save locale to user settings
                  try {
                    await upstashService.updateUserSettings({
                      locale: newValue
                    });
                    toast.success('Language updated');
                  } catch (err) {
                    toast.error('Failed to update language');
                    // Revert on error
                    setLocale(locale);
                    i18n.changeLanguage(locale);
                  }
                }}
              >
                {LOCALES.map((loc) => (
                  <MenuItem key={loc.value} value={loc.value}>
                    {loc.label}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Box>
        </div>

        {/* --- PREFERENCES --- */}
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
                  onChange={async (e) => {
                    const newValue = e.target.checked;
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
                  }}
                />
              </Box>
            ))}
          </Box>
        </div>
      </Box>
    </Box>
  );
};

export default GeneralSettings;
