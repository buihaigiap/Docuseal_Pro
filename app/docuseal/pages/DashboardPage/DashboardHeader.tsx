import { Box, Typography } from '@mui/material';
import { Add as AddIcon, Description as DescriptionIcon } from '@mui/icons-material';
import { motion } from 'framer-motion';
import CreateTemplateButton from '@/components/CreateTemplateButton';
import { useRoleAccess } from '../../hooks/useRoleAccess';
interface DashboardHeaderProps {
  onCreateNew?: () => void;
}

const DashboardHeader: React.FC<DashboardHeaderProps> = ({ onCreateNew }) => {
  const hasAccess = useRoleAccess(['admin', 'editor' , 'member']);

  return (
    <motion.div
      initial={{ opacity: 0, y: -30 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.8, ease: "easeOut" }}
    >
      <Box sx={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexDirection: { xs: 'column', sm: 'row' },
        textAlign: { xs: 'center', sm: 'left' },
        gap: { xs: 2, sm: 3 },
      }}>
        <Box>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 2, sm: 3 }, mb: { xs: 1, sm: 2 } }}>
            <Box sx={{
              p: { xs: 1.5, sm: 1 },
              borderRadius: 3,
              background: 'linear-gradient(135deg, #4F46E5 0%, #7C3AED 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 8px 32px rgba(79, 70, 229, 0.3)',
              position: 'relative',
            }}>
              <DescriptionIcon sx={{ color: 'white', fontSize: { xs: 24, sm: 32 } }} />
            </Box>
            <Box>
              <Typography
                variant="h3"
                component="h3"
                fontWeight="800"
              >
                Dashboard
              </Typography>
              <Typography variant="h6" sx={{ color: '#94a3b8', fontWeight: 400, fontSize: { xs: '0.9rem', sm: '1rem' } }}>
                Manage your document templates
              </Typography>
            </Box>
          </Box>
        </Box>
        {hasAccess && (
          <motion.div
            whileHover={{ scale: 1.08, y: -2 }}
            whileTap={{ scale: 0.98 }}
            transition={{ type: "spring", stiffness: 400, damping: 17 }}
          >
            <CreateTemplateButton
              onClick={onCreateNew}
              text="Create New Template"
              icon={<AddIcon />}
            />
          </motion.div>
        )}
      </Box>
    </motion.div>
  );
};

export default DashboardHeader;