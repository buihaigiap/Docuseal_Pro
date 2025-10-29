import React from 'react';
import { Box, CircularProgress, Typography } from '@mui/material';
import { motion } from 'framer-motion';

const DashboardLoading = () => {
  return (
    <Box sx={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      py: { xs: 8, md: 12 },
      px: { xs: 2, sm: 4 }
    }}>
      <motion.div
        animate={{ rotate: 360 }}
        transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
        style={{
          position: 'relative'
        }}
      >
        <CircularProgress
          size={60}
          thickness={3}
          sx={{
            color: '#4F46E5',
            filter: 'drop-shadow(0 0 20px rgba(79, 70, 229, 0.5))',
            fontSize: { xs: '3rem', sm: '5rem' }
          }}
        />
        <Box sx={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: { xs: 15, sm: 20 },
          height: { xs: 15, sm: 20 },
          borderRadius: '50%',
          background: 'linear-gradient(135deg, #4F46E5, #7C3AED)',
          boxShadow: '0 0 20px rgba(79, 70, 229, 0.8)'
        }} />
      </motion.div>
      <Typography
        variant="h5"
        sx={{
          mt: 3,
          color: '#4F46E5',
          fontWeight: 600,
          fontSize: { xs: '1.2rem', sm: '1.5rem' },
          textShadow: '0 0 20px rgba(79, 70, 229, 0.3)',
          textAlign: 'center'
        }}
      >
        Loading data...
      </Typography>
    </Box>
  );
};

export default DashboardLoading;