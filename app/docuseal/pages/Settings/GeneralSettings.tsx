import {
  Typography,
  Box
} from '@mui/material';
import { useTranslation } from 'react-i18next';
import BasicInformation from './BasicInformation';
import PreferencesSection from './PreferencesSection';

const GeneralSettings = () => {
  const { t } = useTranslation();

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>
        {t('settings.general.title')}
      </Typography>

      <BasicInformation
        initialCompanyName=""
        initialTimezone=""
        initialLocale=""
      />

      <PreferencesSection
        initialPreferences={{
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
        }}
      />
    </Box>
  );
};

export default GeneralSettings;
