import { useState, useCallback } from 'react';

export interface ToastState {
  open: boolean;
  message: string;
  severity: 'success' | 'error' | 'info' | 'warning';
}

export const useToast = () => {
  const [toast, setToast] = useState<ToastState>({
    open: false,
    message: '',
    severity: 'info'
  });

  const showToast = useCallback((message: string, severity: 'success' | 'error' | 'info' | 'warning' = 'info') => {
    setToast({
      open: true,
      message,
      severity
    });
  }, []);

  const hideToast = useCallback(() => {
    setToast(prev => ({
      ...prev,
      open: false
    }));
  }, []);

  return {
    toast,
    showToast,
    hideToast
  };
};