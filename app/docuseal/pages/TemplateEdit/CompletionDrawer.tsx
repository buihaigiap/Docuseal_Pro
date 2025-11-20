import React from 'react';
import {
  Drawer,
  Box,
  Typography,
  Button,
  IconButton,
} from '@mui/material';
import { Close as CloseIcon, Download as DownloadIcon, Email as EmailIcon, Build as TestIcon } from '@mui/icons-material';
import CreateTemplateButton from '@/components/CreateTemplateButton';
import { downloadSignedPDF } from '../../services/pdfDownloadService';
import toast from 'react-hot-toast';

interface CompletionDrawerProps {
  open: boolean;
  onClose: () => void;
  title: string;
  body: string;
  pdfUrl: string;
  signatures: any[];
  templateName: string;
  submitterInfo?: { id: number; email: string } | null;
  globalSettings?: any;
  onSendCopy: () => void;
  onTest: () => void;
}

const CompletionDrawer: React.FC<CompletionDrawerProps> = ({
  open,
  onClose,
  title,
  body,
  pdfUrl,
  signatures,
  templateName,
  submitterInfo,
  globalSettings,
  onSendCopy,
  onTest
}) => {
  const handleDownload = async () => {
    try {
      await downloadSignedPDF(
        pdfUrl,
        signatures,
        templateName,
        submitterInfo,
        globalSettings
      );
      toast.success('Document downloaded successfully');
    } catch (error) {
      console.error('Download error:', error);
      toast.error('Failed to download document');
    }
  };
  return (
    <Drawer
      anchor="bottom"
      open={open}
      onClose={onClose}
      sx={{
        '& .MuiDrawer-paper': {
          width:'100%',
          maxWidth: 800,
          position: 'absolute',
          left: '35%',
          transform: 'translate(-50%, -50%)',
          py: 3,
          px: 10,
          display: 'flex',
          flexDirection: 'column'
        }
      }}
    >
      {/* Header */}
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
        <Typography variant="h6" component="h2" sx={{ fontWeight: 600 }}>
          {title}
        </Typography>
        <IconButton onClick={onClose} size="small">
          <CloseIcon  sx={{color :'white'}}/>
        </IconButton>
      </Box>

      {/* Body */}
      <Box >
        <Typography variant="body1" sx={{ color: 'text.secondary', lineHeight: 1.6 }}>
          {body}
        </Typography>
      </Box>

      {/* Buttons */}
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <Button
          variant="contained"
          startIcon={<DownloadIcon />}
          onClick={handleDownload}
          fullWidth
        >
          Download Document
        </Button>

        <Button
          sx={{
            color : 'white'
          }}
          variant="outlined"
          startIcon={<EmailIcon />}
          onClick={onSendCopy}
          fullWidth
        >
          Send Copy Via Email
        </Button>

        <CreateTemplateButton
            text="text"
            onClick={onTest}
        />
      </Box>
    </Drawer>
  );
};

export default CompletionDrawer;