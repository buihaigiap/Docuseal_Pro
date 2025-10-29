import React from 'react';
import { Alert } from '@mui/material';
import { motion } from 'framer-motion';

interface DashboardErrorProps {
  error: string;
}

const DashboardError: React.FC<DashboardErrorProps> = ({ error }) => {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay: 0.2 }}
    >
      <Alert
        severity="error"
        sx={{
          borderRadius: 3,
          background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.1) 0%, rgba(220, 38, 38, 0.1) 100%)',
          color: '#EF4444',
          border: '1px solid rgba(239, 68, 68, 0.3)',
          boxShadow: '0 8px 32px rgba(239, 68, 68, 0.2)',
          '& .MuiAlert-icon': {
            color: '#EF4444',
            filter: 'drop-shadow(0 0 10px rgba(239, 68, 68, 0.5))'
          }
        }}
      >
        {error}
      </Alert>
    </motion.div>
  );
};

export default DashboardError;