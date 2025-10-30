import React, { useState, useEffect, useRef } from 'react';
import { Box, Modal, Button, Typography, CircularProgress } from '@mui/material';
import { Close as CloseIcon, CloudUpload as CloudUploadIcon } from '@mui/icons-material';

interface GoogleDrivePickerProps {
  open: boolean;
  onClose: () => void;
  onFileSelect: (files: any[]) => void;
}

const GoogleDrivePicker: React.FC<GoogleDrivePickerProps> = ({
  open,
  onClose,
  onFileSelect
}) => {
  const [isLoading, setIsLoading] = useState(true);
  const [showOAuth, setShowOAuth] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (event.data.type === 'google-drive-files-picked') {
        const files = event.data.files || [];
        onFileSelect(files);
        onClose();
      } else if (event.data.type === 'google-drive-picker-loaded') {
        setIsLoading(false);
      } else if (event.data.type === 'google-drive-picker-request-oauth') {
        // Redirect to OAuth instead of showing OAuth button
        handleOAuth();
      }
    };

    if (open) {
      window.addEventListener('message', handleMessage);
      setIsLoading(true);
      setShowOAuth(false);
    }

    return () => {
      window.removeEventListener('message', handleMessage);
    };
  }, [open, onClose, onFileSelect]);

  const handleOAuth = () => {
    // Redirect to Google OAuth
    const params = new URLSearchParams({
      access_type: 'offline',
      include_granted_scopes: 'true',
      prompt: 'consent',
      scope: [
        'https://www.googleapis.com/auth/userinfo.email',
        'https://www.googleapis.com/auth/drive.file'
      ].join(' '),
      state: JSON.stringify({
        redir: window.location.pathname
      })
    });

    window.location.href = `/auth/google_oauth2?${params.toString()}`;
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      aria-labelledby="google-drive-modal"
      sx={{
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        pt: 5,
        px: 2
      }}
    >
      <Box
        sx={{
          bgcolor: 'background.paper',
          borderRadius: 2,
          boxShadow: 24,
          width: '100%',
          maxWidth: 600,
          maxHeight: '80vh',
          display: 'flex',
          flexDirection: 'column'
        }}
      >
        {/* Header */}
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            p: 2,
            borderBottom: 1,
            borderColor: 'divider'
          }}
        >
          <Typography variant="h6" component="h2">
            Google Drive
          </Typography>
          <Button onClick={onClose} size="small">
            <CloseIcon />
          </Button>
        </Box>

        {/* Content */}
        <Box sx={{ flex: 1, position: 'relative', minHeight: 400 }}>
          {showOAuth ? (
            <Box
              sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                height: '100%',
                p: 3
              }}
            >
              <CloudUploadIcon sx={{ fontSize: 48, color: 'primary.main', mb: 2 }} />
              <Typography variant="h6" gutterBottom>
                Connect Google Drive
              </Typography>
              <Typography variant="body2" color="text.secondary" textAlign="center" mb={3}>
                Connect your Google Drive to upload documents directly from your cloud storage.
              </Typography>
              <Button
                variant="contained"
                onClick={handleOAuth}
                startIcon={<CloudUploadIcon />}
              >
                Connect Google Drive
              </Button>
            </Box>
          ) : (
            <>
              {isLoading && (
                <Box
                  sx={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 0,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    bgcolor: 'rgba(255, 255, 255, 0.8)',
                    zIndex: 1
                  }}
                >
                  <CircularProgress />
                </Box>
              )}
              <iframe
                ref={iframeRef}
                src="/template_google_drive"
                style={{
                  width: '100%',
                  height: '100%',
                  minHeight: 400,
                  border: 'none',
                  borderRadius: '0 0 8px 8px'
                }}
                title="Google Drive Picker"
              />
            </>
          )}
        </Box>
      </Box>
    </Modal>
  );
};

export default GoogleDrivePicker;