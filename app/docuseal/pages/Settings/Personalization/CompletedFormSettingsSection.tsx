import { useState } from 'react';
import {
    Typography, Box, Button, Card, CardContent, TextField,
    Switch
} from '@mui/material';
import { Save } from '@mui/icons-material';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';

interface CompletedFormSettingsSectionProps {
  completionTitle: string;
  setCompletionTitle: (title: string) => void;
  completionBody: string;
  setCompletionBody: (body: string) => void;
  redirectTitle: string;
  setRedirectTitle: (title: string) => void;
  redirectUrl: string;
  setRedirectUrl: (url: string) => void;
  enableConfetti: boolean;
  setEnableConfetti: (enabled: boolean) => void;
}

export default function CompletedFormSettingsSection({
  completionTitle,
  setCompletionTitle,
  completionBody,
  setCompletionBody,
  redirectTitle,
  setRedirectTitle,
  redirectUrl,
  setRedirectUrl,
  enableConfetti,
  setEnableConfetti
}: CompletedFormSettingsSectionProps) {
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      const response = await upstashService.updateUserSettings({
        completion_title: completionTitle || '',
        completion_body: completionBody || '',
        redirect_title: redirectTitle || '',
        redirect_url: redirectUrl || '',
        enable_confetti: enableConfetti,
      });
      if (response.success) {
        toast.success('Completed form settings updated successfully');
      } else {
        toast.error('Failed to update completed form settings');
      }
    } catch (error) {
      console.error('Failed to save completed form settings:', error);
      toast.error('Failed to save completed form settings');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card sx={{
        my : 3
    }}>
      <CardContent>
        <Typography variant="h6" component="h3" gutterBottom>
          Completed Form Settings
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
          Customize the message and redirect button displayed when a form is successfully completed.
        </Typography>

        {/* Completed Form Message Section */}
        <Box sx={{ mb: 3 }}>
          <Typography variant="subtitle1" gutterBottom>
            Message
          </Typography>
          <TextField
            fullWidth
            label="Title"
            value={completionTitle}
            onChange={(e) => setCompletionTitle(e.target.value)}
            placeholder="Form Completed Successfully"
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Message"
            value={completionBody}
            onChange={(e) => setCompletionBody(e.target.value)}
            placeholder="Thank you for completing the form. Your submission has been received."
            multiline
            rows={3}
          />
        </Box>

        {/* Completed Form Redirect Section */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="subtitle1" gutterBottom>
            Redirect Button
          </Typography>
          <TextField
            fullWidth
            label="Button Title"
            value={redirectTitle}
            onChange={(e) => setRedirectTitle(e.target.value)}
            placeholder="Continue to Website"
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Redirect URL"
            value={redirectUrl}
            onChange={(e) => setRedirectUrl(e.target.value)}
            placeholder="https://example.com/thank-you"
            type="url"
          />
        </Box>

        {/* Confetti Settings */}
        <Box sx={{ mb: 2, display: 'flex', alignItems: 'center', gap: 2 }}>
          <Typography variant="subtitle1" sx={{ flex: 1 }}>
            Show confetti on successful completion
          </Typography>
          <Switch
            checked={enableConfetti}
            onChange={(e) => setEnableConfetti(e.target.checked)}
            color="primary"
          />
        </Box>

        <Button
          variant="contained"
          startIcon={<Save />}
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? 'Saving...' : 'Save Changes'}
        </Button>
      </CardContent>
    </Card>
  );
}