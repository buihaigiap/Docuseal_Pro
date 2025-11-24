import { useState } from 'react';
import {
  Typography, Box, Button, Card, CardContent,
  Avatar
} from '@mui/material';
import { CloudUpload, Delete } from '@mui/icons-material';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';
import { useNavigate } from 'react-router-dom';
import UpdatePro from '@/components/updatePro';

interface LogoSectionProps {
  logoUrl: string | null;
  setLogoUrl: (url: string | null) => void;
  isPaidUser: boolean;
}

export default function LogoSection({ logoUrl, setLogoUrl, isPaidUser }: LogoSectionProps) {
  const [uploading, setUploading] = useState(false);
  const navigate = useNavigate();

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.type.startsWith('image/')) {
      toast.error('Please select an image file');
      return;
    }

    // Validate file size (max 5MB)
    if (file.size > 5 * 1024 * 1024) {
      toast.error('File size must be less than 5MB');
      return;
    }

    setUploading(true);
    try {
      const response = await upstashService.uploadLogo(file);
      if (response.success) {
        setLogoUrl(response.data);
        toast.success('Logo uploaded successfully');
      }
    } catch (error) {
      toast.error(error?.message || 'Failed to upload logo');
      navigate('/pricing');
    } finally {
      setUploading(false);
    }
  };

  const handleRemoveLogo = async () => {
    try {
      await upstashService.updateUserSettings({ logo_url: null });
      setLogoUrl(null);
      toast.success('Logo removed successfully');
    } catch (error) {
      console.error('Remove error:', error);
      toast.error('Failed to remove logo');
    }
  };

  if (!isPaidUser) {
    return <UpdatePro />;
  }

  return (
    <Card sx={{ mt: 3 }}>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Company Logo
        </Typography>
        <Typography variant="body2" color="textSecondary" gutterBottom>
          Upload your company logo to display it on signing forms instead of the default logo.
        </Typography>

        <Box display="flex" alignItems="center" gap={2} mt={2}>
          {logoUrl ? (
            <>
              <Avatar
                src={logoUrl}
                alt="Company Logo"
                sx={{ width: 80, height: 80 }}
                variant="rounded"
              />
              <Box>
                <Button
                  sx={{
                    color: 'white'
                  }}
                  variant="outlined"
                  startIcon={<CloudUpload />}
                  component="label"
                  disabled={uploading}
                >
                  {uploading ? 'Uploading...' : 'Change Logo'}
                  <input
                    type="file"
                    hidden
                    accept="image/*"
                    onChange={handleFileUpload}
                  />
                </Button>
                <Button
                  variant="text"
                  color="error"
                  startIcon={<Delete />}
                  onClick={handleRemoveLogo}
                  sx={{ ml: 1 }}
                >
                  Remove
                </Button>
              </Box>
            </>
          ) : (
            <Button
              variant="outlined"
              startIcon={<CloudUpload />}
              component="label"
              disabled={uploading}
              sx={{
                color: 'white'
              }}
            >
              {uploading ? 'Uploading...' : 'Upload Logo'}
              <input
                type="file"
                hidden
                accept="image/*"
                onChange={handleFileUpload}
              />
            </Button>
          )}
        </Box>
        <Typography variant="caption" color="textSecondary" sx={{ mt: 1, display: 'block' }}>
          Supported formats: PNG, JPG, JPEG. Maximum size: 5MB.
        </Typography>
      </CardContent>
    </Card>
  );
}