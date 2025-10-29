import React from 'react';
import { Snackbar, Alert } from '@mui/material';

interface ToastProps {
  open: boolean;
  message: string;
  severity: 'success' | 'error' | 'info' | 'warning';
  onClose: () => void;
  autoHideDuration?: number;
}

const Toast: React.FC<ToastProps> = ({
  open,
  message,
  severity,
  onClose,
}) => {
  return (
    <div style={{ zIndex: 99999, position: 'relative' }}>
      <Snackbar
        open={open}
        autoHideDuration={1000}
        onClose={onClose}
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
        style={{ top: '80px' }}
      >
        <Alert
          onClose={onClose}
          severity={severity}
          sx={{ width: '100%' }}
        >
          {message}
        </Alert>
      </Snackbar>
    </div>
  );
};

export default Toast;