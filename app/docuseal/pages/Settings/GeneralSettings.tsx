import { useState, useEffect } from 'react';
import {
  Typography,
  Box,
  TextField,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem
} from '@mui/material';
import upstashService from '../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import { useTranslation } from 'react-i18next';

const TIMEZONES = [
  "International Date Line West",
  "Midway Island",
  "American Samoa",
  "Hawaii",
  "Alaska",
  "Pacific Time (US & Canada)",
  "Tijuana",
  "Mountain Time (US & Canada)",
  "Arizona",
  "Chihuahua",
  "Mazatlan",
  "Central Time (US & Canada)",
  "Saskatchewan",
  "Guadalajara",
  "Mexico City",
  "Monterrey",
  "Central America",
  "Eastern Time (US & Canada)",
  "Indiana (East)",
  "Bogota",
  "Lima",
  "Quito",
  "Atlantic Time (Canada)",
  "Caracas",
  "La Paz",
  "Santiago",
  "Asuncion",
  "Newfoundland",
  "Brasilia",
  "Buenos Aires",
  "Montevideo",
  "Georgetown",
  "Puerto Rico",
  "Greenland",
  "Mid-Atlantic",
  "Azores",
  "Cape Verde Is.",
  "Dublin",
  "Edinburgh",
  "Lisbon",
  "London",
  "Casablanca",
  "Monrovia",
  "UTC",
  "Belgrade",
  "Bratislava",
  "Budapest",
  "Ljubljana",
  "Prague",
  "Sarajevo",
  "Skopje",
  "Warsaw",
  "Zagreb",
  "Brussels",
  "Copenhagen",
  "Madrid",
  "Paris",
  "Amsterdam",
  "Berlin",
  "Bern",
  "Zurich",
  "Rome",
  "Stockholm",
  "Vienna",
  "West Central Africa",
  "Bucharest",
  "Cairo",
  "Helsinki",
  "Kyiv",
  "Riga",
  "Sofia",
  "Tallinn",
  "Vilnius",
  "Athens",
  "Istanbul",
  "Minsk",
  "Jerusalem",
  "Harare",
  "Pretoria",
  "Kaliningrad",
  "Moscow",
  "St. Petersburg",
  "Volgograd",
  "Samara",
  "Kuwait",
  "Riyadh",
  "Nairobi",
  "Baghdad",
  "Tehran",
  "Abu Dhabi",
  "Muscat",
  "Baku",
  "Tbilisi",
  "Yerevan",
  "Kabul",
  "Ekaterinburg",
  "Islamabad",
  "Karachi",
  "Tashkent",
  "Chennai",
  "Kolkata",
  "Mumbai",
  "New Delhi",
  "Kathmandu",
  "Dhaka",
  "Sri Jayawardenepura",
  "Almaty",
  "Astana",
  "Novosibirsk",
  "Rangoon",
  "Bangkok",
  "Hanoi",
  "Jakarta",
  "Krasnoyarsk",
  "Beijing",
  "Chongqing",
  "Hong Kong",
  "Urumqi",
  "Kuala Lumpur",
  "Singapore",
  "Taipei",
  "Perth",
  "Irkutsk",
  "Ulaanbaatar",
  "Seoul",
  "Osaka",
  "Sapporo",
  "Tokyo",
  "Yakutsk",
  "Darwin",
  "Adelaide",
  "Canberra",
  "Melbourne",
  "Sydney",
  "Brisbane",
  "Hobart",
  "Vladivostok",
  "Guam",
  "Port Moresby",
  "Magadan",
  "Srednekolymsk",
  "Solomon Is.",
  "New Caledonia",
  "Fiji",
  "Kamchatka",
  "Marshall Is.",
  "Auckland",
  "Wellington",
  "Nuku'alofa",
  "Tokelau Is.",
  "Chatham Is.",
  "Samoa"
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

const GeneralSettings = () => {
  const { t, i18n } = useTranslation();
  const [companyName, setCompanyName] = useState('');
  const [timezone, setTimezone] = useState('');
  const [locale, setLocale] = useState('');
  const [loading, setLoading] = useState(false);
  const [fetchLoading, setFetchLoading] = useState(true);

  useEffect(() => {
    const fetchSettings = async () => {
      try {
        const response = await upstashService.getBasicSettings();
        const settings = response.data;
        setCompanyName(settings.company_name || '');
        setTimezone(settings.timezone || '');
        setLocale(settings.locale || '');
      } catch (err) {
        toast.error('Failed to fetch settings');
      } finally {
        setFetchLoading(false);
      }
    };
    fetchSettings();
  }, []);

  const handleUpdateSettings = async (e) => {
    e.preventDefault();
    setLoading(true);
    try {
      await upstashService.updateBasicSettings({
        company_name: companyName || null,
        timezone: timezone || null,
        locale: locale || null,
      });
      
      // Change i18next language if locale was updated
      if (locale) {
        const languageCode = locale.split('-')[0]; // Extract language code (e.g., 'en' from 'en-US')
        await i18n.changeLanguage(languageCode);
      }
      
      toast.success(t('settings.general.updated'));
    } catch (err) {
      toast.error(t('settings.general.updateFailed'));
    } finally {
      setLoading(false);
    }
  };

  if (fetchLoading) {
    return <Typography>{t('common.loading')}</Typography>;
  }

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>
        {t('settings.general.title')}
      </Typography>
      <Box component="form" onSubmit={handleUpdateSettings}>
        <div className="bg-white/5 border border-white/10 rounded-lg p-4 mb-4">
          <Typography variant="h6" sx={{ mb: 2 }}>
            {t('settings.general.basicInfo')}
          </Typography>

          {/* Company Name */}
          <TextField
            fullWidth
            label={t('settings.general.companyName')}
            value={companyName}
            onChange={(e) => setCompanyName(e.target.value)}
            sx={{ mb: 2 }}
          />

          {/* Time Zone & Language/Locale cùng 1 hàng */}
          <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
            <FormControl fullWidth sx={{ flex: 1 }}>
              <InputLabel>{t('settings.general.timeZone')}</InputLabel>
              <Select
                value={timezone}
                label={t('settings.general.timeZone')}
                onChange={(e) => setTimezone(e.target.value)}
              >
                {TIMEZONES.map((tz) => (
                  <MenuItem key={tz} value={tz}>
                    {tz}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <FormControl fullWidth sx={{ flex: 1 }}>
              <InputLabel>{t('settings.general.language')}</InputLabel>
              <Select
                value={locale}
                label={t('settings.general.language')}
                onChange={(e) => setLocale(e.target.value)}
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

        <Button
          type="submit"
          variant="contained"
          disabled={loading}
          sx={{ bgcolor: 'primary.main', '&:hover': { bgcolor: 'primary.dark' } }}
        >
          {loading ? t('settings.general.updating') : t('settings.general.update')}
        </Button>
      </Box>
    </Box>
  );
};

export default GeneralSettings;
