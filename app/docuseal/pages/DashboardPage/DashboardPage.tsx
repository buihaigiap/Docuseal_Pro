import { useState, useEffect } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { Template } from '../../types';
import upstashService from '../../ConfigApi/upstashService';
import { Box, CircularProgress } from '@mui/material';
import { motion } from 'framer-motion';
import toast from 'react-hot-toast';
import DashboardHeader from './DashboardHeader';
import DashboardError from './DashboardError';
import TemplatesGrid from './TemplatesGrid';
import EmptyState from './EmptyState';
import FoldersList from '../../components/FoldersList';
import NewTemplateModal from '../../components/NewTemplateModal';

const DashboardPage = () => {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [folders, setFolders] = useState<any[]>([]);
  const [showNewTemplateModal, setShowNewTemplateModal] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const { token } = useAuth();

  // Check if we just returned from Google OAuth
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('google_drive_connected') === '1') {
      // Remove the query parameter
      window.history.replaceState({}, '', window.location.pathname);
      toast.success('Google Drive connected successfully!');
    }
  }, []);

  const fetchTemplates = async () => {
    if (!token) {
        setError("Authentication token not found.");
        setLoading(false);
        return;
    }
    try {
      setLoading(true);
      setError('');
      const data = await upstashService.getTemplates();
      if (data.success) {
        setTemplates(data.data);
      } else {
        setError(data.message || 'Failed to fetch templates.');
      }

      // Fetch folders
      const foldersData = await upstashService.getFolders();
      if (foldersData.success) {
        setFolders(foldersData.data || []);
      }
    } catch (err) {
      setError('An unexpected error occurred while fetching templates.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTemplates();
  }, [token]);

  const filteredFolders = folders.filter(folder =>
    folder.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredTemplates = templates.filter(template =>
    template.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <Box sx={{
      marginTop: { xs: 4, md: 6 },
    }}>
      <Box >
        {/* Header Section */}
        <DashboardHeader 
          onCreateNew={() => setShowNewTemplateModal(true)} 
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
        />

        {/* Content Container */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          transition={{ delay: 0.3, duration: 0.6, ease: "easeOut" }}
        >
          <FoldersList folders={filteredFolders} />

        {loading ? (
            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '400px' }}>
              <CircularProgress size={60} />
            </Box>
          ) : error ? (
            <DashboardError error={error} />
          ) : (
            <>
              {filteredTemplates.length > 0 && (
                <TemplatesGrid templates={filteredTemplates} onRefresh={fetchTemplates} />
              )}
              <EmptyState />
            </>
          )}
        </motion.div>
      </Box>
      <NewTemplateModal
        open={showNewTemplateModal}
        onClose={() => setShowNewTemplateModal(false)}
        folderId={null}
        onSuccess={fetchTemplates}
      />
    </Box>
  );
};

export default DashboardPage;
