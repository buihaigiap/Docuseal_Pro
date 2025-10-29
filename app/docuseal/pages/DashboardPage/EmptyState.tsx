import React from 'react';
import { Link } from 'react-router-dom';
import { Box, Typography, Button } from '@mui/material';
import { Add as AddIcon, FolderOpen as FolderOpenIcon } from '@mui/icons-material';
import { motion } from 'framer-motion';

const EmptyState = () => {
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

        <Typography variant="body1" sx={{ color: '#64748b', mb: 6, fontSize: { xs: '0.9rem', sm: '1rem' } }}>
          Click the "Create New Template" button to upload a PDF and start the signing process.
        </Typography>
      </Box>
    </motion.div>
  );
};

export default EmptyState;