import React, { useState } from 'react';
import { Box, Typography, Button } from '@mui/material';
import { Add as AddIcon, FolderOpen as FolderOpenIcon } from '@mui/icons-material';
import { motion } from 'framer-motion';
import GoogleDrivePicker from '../../components/GoogleDrivePicker';
import axios from 'axios';
import toast from 'react-hot-toast';
import CreateTemplateButton from '../../components/CreateTemplateButton';
const EmptyState = () => {
  const [showGoogleDrivePicker, setShowGoogleDrivePicker] = useState(false);

  const handleGoogleDriveSelect = async (files: any[]) => {
    if (files.length > 0) {
      const file = files[0];
      try {
        // Create template from Google Drive file
        const response = await axios.post('/api/templates/google_drive_documents', {
          google_drive_file_ids: [file.id],
          name: file.name.replace('.pdf', '')
        }, {
          headers: {
            Authorization: `Bearer ${localStorage.getItem('token')}`
          }
        });

        if (response.data.success) {
          toast.success('Template created successfully!');
          window.location.reload(); // Refresh to show new template
        } else {
          toast.error('Failed to create template');
        }
      } catch (error) {
        console.error('Error creating template from Google Drive:', error);
        toast.error('Failed to create template from Google Drive');
      }
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay: 0.2 }}
    >
      <Box sx={{
        textAlign: 'center',
      }}>
        <Box sx={{
          width: { xs: 80, sm: 120 },
          height: { xs: 80, sm: 120 },
          borderRadius: '50%',
          background: 'linear-gradient(135deg, #4F46E5 0%, #7C3AED 100%)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          mx: 'auto',
          mb: { xs: 4, sm: 6 },
          boxShadow: '0 20px 60px rgba(79, 70, 229, 0.3)',
          position: 'relative',
          '&::before': {
            content: '""',
            position: 'absolute',
            inset: 0,
            borderRadius: '50%',
            padding: '3px',
            background: 'linear-gradient(135deg, rgba(255,255,255,0.3), rgba(255,255,255,0.1))',
            mask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
            maskComposite: 'subtract'
          }
        }}>
          <FolderOpenIcon sx={{ fontSize: { xs: 40, sm: 60 }, color: 'white' }} />
        </Box>

        <Typography
          variant="h3"
          component="h2"
          fontWeight="800"
          sx={{
            color: 'white',
            mb: 3,
            fontSize: { xs: '1.5rem', sm: '2rem', md: '2.5rem' },
            background: 'linear-gradient(135deg, #ffffff 0%, #e2e8f0 100%)',
            backgroundClip: 'text',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent'
          }}
        >
          Welcome to Letmesign
        </Typography>

        <Typography variant="h6" sx={{ color: '#94a3b8', mb: 4, maxWidth: 600, mx: 'auto', lineHeight: 1.6, fontSize: { xs: '1rem', sm: '1.25rem' } }}>
          You don't have any document templates yet. Start by creating your first template to begin the document signing process!
        </Typography>

        <Box sx={{ display: 'flex', justifyContent: 'center', gap: 2, mb: 6 }}>
          <CreateTemplateButton
            text="Google Drive"
            onClick={() => setShowGoogleDrivePicker(true)}
            icon={<FolderOpenIcon />}
          />
        </Box>
      </Box>

      <GoogleDrivePicker
        open={showGoogleDrivePicker}
        onClose={() => setShowGoogleDrivePicker(false)}
        onFileSelect={handleGoogleDriveSelect}
      />
    </motion.div>
  );
};

export default EmptyState;