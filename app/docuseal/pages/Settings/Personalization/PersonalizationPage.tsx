import { useState, useEffect } from 'react';
import {
    Typography, Box,
} from '@mui/material';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import { useTranslation } from 'react-i18next';
import { useAuth } from '../../../contexts/AuthContext';
import LogoSection from './LogoSection';
import EmailTemplatesSection from './EmailTemplatesSection';
import CompletedFormSettingsSection from './CompletedFormSettingsSection';

interface EmailTemplate {
  id: number;
  user_id: number;
  template_type: string;
  subject: string;
  body: string;
  body_format: string;
  is_default: boolean;
  attach_documents?: boolean;
  attach_audit_log?: boolean;
  created_at: string;
  updated_at: string;
}

export default function PersonalizationPage() {
  const { t } = useTranslation();
  const { user, refreshUser } = useAuth();
  const [logoUrl, setLogoUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [templates, setTemplates] = useState<EmailTemplate[]>([]);
  const [templatesLoading, setTemplatesLoading] = useState(false);
  const [completionTitle, setCompletionTitle] = useState<string>('');
  const [completionBody, setCompletionBody] = useState<string>('');
  const [redirectTitle, setRedirectTitle] = useState<string>('');
  const [redirectUrl, setRedirectUrl] = useState<string>('');

  useEffect(() => {
    fetchSettings();
    fetchTemplates();
    // Check subscription status separately to ensure it's up to date
    checkSubscriptionStatus();
  }, []);

  const fetchSettings = async () => {
    try {
      const settingsRes = await upstashService.getUserSettings();
      if (settingsRes.success) {
        setLogoUrl(settingsRes.data.logo_url || null);
        setCompletionTitle(settingsRes.data.completion_title || '');
        setCompletionBody(settingsRes.data.completion_body || '');
        setRedirectTitle(settingsRes.data.redirect_title || '');
        setRedirectUrl(settingsRes.data.redirect_url || '');
      }
    } catch (error) {
      console.error('Failed to fetch settings:', error);
      toast.error('Failed to load personalization settings');
    } finally {
      setLoading(false);
    }
  };

  const checkSubscriptionStatus = async () => {
    try {
      await refreshUser();
      return user?.subscription_status === 'premium';
    } catch (error) {
      console.error('Failed to check subscription status:', error);
      return false;
    }
  };

  const fetchTemplates = async () => {
    setTemplatesLoading(true);
    try {
      const res = await upstashService.getEmailTemplates();
      if (res.success) {
        setTemplates(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch email templates:', error);
    } finally {
      setTemplatesLoading(false);
    }
  };

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="200px">
        <Typography>Loading...</Typography>
      </Box>
    );
  }

  const isPaidUser = user?.subscription_status === 'premium';

  return (
    <Box
    >
      <Typography variant="h4" component="h1" gutterBottom>
        Personalization
      </Typography>

      <LogoSection
        logoUrl={logoUrl}
        setLogoUrl={setLogoUrl}
        isPaidUser={isPaidUser}
      />

      <CompletedFormSettingsSection
        completionTitle={completionTitle}
        setCompletionTitle={setCompletionTitle}
        completionBody={completionBody}
        setCompletionBody={setCompletionBody}
        redirectTitle={redirectTitle}
        setRedirectTitle={setRedirectTitle}
        redirectUrl={redirectUrl}
        setRedirectUrl={setRedirectUrl}
      />

      <EmailTemplatesSection
        templates={templates}
        templatesLoading={templatesLoading}
        fetchTemplates={fetchTemplates}
      />
    </Box>
  );
}