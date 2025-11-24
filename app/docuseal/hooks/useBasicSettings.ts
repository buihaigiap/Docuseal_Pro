import { useState, useEffect, useCallback } from 'react';
import upstashService from '../ConfigApi/upstashService';

export const useBasicSettings = () => {
  const [globalSettings, setGlobalSettings] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');

  const fetchSettings = useCallback(async () => {
    try {
      setLoading(true);
      setError('');
      const response = await upstashService.getUserSettings();
      if (response.success) {
        setGlobalSettings(response.data);
      } else {
        setError(response.message || 'Failed to fetch settings');
      }
    } catch (err) {
      setError('An unexpected error occurred');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  return { globalSettings, loading, error, refetch: fetchSettings };
};