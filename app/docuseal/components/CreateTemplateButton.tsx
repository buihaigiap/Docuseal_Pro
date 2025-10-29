import React from 'react';
import { Button, CircularProgress } from '@mui/material';

interface CreateTemplateButtonProps {
  onClick?: () => void;
  text?: string;
  loading?: boolean;
  background ?: string;
  icon?: any;
}

const CreateTemplateButton: React.FC<CreateTemplateButtonProps> = ({
  onClick,
  text = '',
  loading = false,
  background = 'linear-gradient(135deg, #4F46E5 0%, #7C3AED 100%)',
  icon = null,
}) => {
  return (
    <Button
      startIcon={icon}
      type='submit'
      onClick={onClick}
      variant="contained"
      disabled={loading}
      sx={{
        background,
        color: 'white',
        fontWeight: '700',
        boxShadow: '0 8px 32px rgba(79, 70, 229, 0.4)',
        position: 'relative',
        overflow: 'hidden',
        // minWidth: { xs: 'auto', sm: '200px' },
        '&::before': {
          content: '""',
          position: 'absolute',
          top: 0,
          left: '-100%',
          width: '100%',
          height: '100%',
          background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent)',
          transition: 'left 0.5s',
        },
        '&:hover': {
          background: 'linear-gradient(135deg, #4338CA 0%, #6D28D9 100%)',
          boxShadow: '0 12px 40px rgba(79, 70, 229, 0.5)',
          '&::before': {
            left: '100%'
          }
        },
        '&:disabled': {
          background: 'linear-gradient(135deg, #9CA3AF 0%, #6B7280 100%)',
          color: '#E5E7EB',
          boxShadow: 'none',
        }
      }}
    >
      {loading ? <CircularProgress size={20} sx={{ color: 'white' }} /> : text}
    </Button>
  );
};

export default CreateTemplateButton;