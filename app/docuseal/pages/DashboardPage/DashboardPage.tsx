import { useState, useEffect } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { Template } from '../../types';
import upstashService from '../../ConfigApi/upstashService';
import { Box } from '@mui/material';
import { motion } from 'framer-motion';
import DashboardHeader from './DashboardHeader';
import DashboardLoading from './DashboardLoading';
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
  const { token } = useAuth();
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

  return (
    <Box sx={{
      marginTop: { xs: 4, md: 6 },
    }}>
      <Box >
        {/* Header Section */}
        <DashboardHeader onCreateNew={() => setShowNewTemplateModal(true)} />

        {/* Content Container */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          transition={{ delay: 0.3, duration: 0.6, ease: "easeOut" }}
        >
          <FoldersList folders={folders} />

          {loading ? (
            <DashboardLoading />
          ) : error ? (
            <DashboardError error={error} />
          ) : templates.length > 0 ? (
            <TemplatesGrid templates={templates} onRefresh={fetchTemplates} />
          ) : (
            <EmptyState />
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
