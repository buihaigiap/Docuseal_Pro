import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import upstashService from '../ConfigApi/upstashService';
import { Dialog, DialogTitle, DialogContent, DialogActions, TextField, Button, CircularProgress, Box, Alert, styled, Typography } from '@mui/material';
import { CloudUpload as CloudUploadIcon } from '@mui/icons-material';
import { useTranslation } from 'react-i18next';

const Input = styled('input')({
  display: 'none',
});

interface NewTemplateModalProps {
  open: boolean;
  onClose: () => void;
  folderId: number | null;
  onSuccess: () => void;
}

const NewTemplateModal: React.FC<NewTemplateModalProps> = ({ open, onClose, folderId, onSuccess }) => {
  const [file, setFile] = useState<File | null>(null);
  const [templateName, setTemplateName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { t } = useTranslation();

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setFile(e.target.files[0]);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!file || !templateName) {
      setError(t('templates.errors.nameAndFileRequired'));
      return;
    }
    setError('');
    setLoading(true);

    try {
      const uploadFormData = new FormData();
      uploadFormData.append('file', file);
      uploadFormData.append('file_type', 'document');

      const uploadData = await upstashService.uploadFile(uploadFormData);
      if (!uploadData.success) {
        throw new Error(uploadData.message || t('templates.errors.uploadFailed'));
      }

      const fileId = uploadData.data.id;

      const templateData = await upstashService.createTemplateFromFile({
        file_id: fileId,
        name: templateName,
        description: '',
        folder_id: folderId
      });
      if (templateData.success) {
        onSuccess();
        onClose();
        navigate(`/templates/${templateData.data.id}`);
      } else {
        throw new Error(templateData.message || t('templates.errors.createFailed'));
      }
    } catch (err: any) {
      setError(err.message || t('templates.errors.unexpectedError'));
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setFile(null);
    setTemplateName('');
    setError('');
    onClose();
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>{t('templates.modal.title')}</DialogTitle>
      <DialogContent>
        <Box component="form" onSubmit={handleSubmit} noValidate sx={{ mt: 1 }}>
          <TextField
            margin="normal"
            required
            fullWidth
            id="templateName"
            label={t('templates.modal.nameLabel')}
            name="templateName"
            autoFocus
            value={templateName}
            onChange={(e) => setTemplateName(e.target.value)}
            placeholder={t('templates.modal.namePlaceholder')}
          />
          <Box
            sx={{
              mt: 2,
              p: 3,
              border: '2px dashed',
              borderColor: 'grey.400',
              borderRadius: 1,
              textAlign: 'center',
              bgcolor: file ? 'primary.lighter' : 'transparent',
              transition: 'background-color 0.3s'
            }}
          >
            <CloudUploadIcon sx={{ fontSize: 48, color: 'grey.500', mb: 1 }} />
            <Typography variant="h6" color="text.secondary">
              {file ? file.name : t('templates.modal.dragDropText')}
            </Typography>
            <label htmlFor="contained-button-file">
              <Input accept="application/pdf" id="contained-button-file" type="file" onChange={handleFileChange} />
              <Button variant="contained" component="span" sx={{ mt: 2 }}>
                {t('templates.modal.uploadButton')}
              </Button>
            </label>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {t('templates.modal.pdfOnly')}
            </Typography>
          </Box>
          {error && <Alert severity="error" sx={{ mt: 2 }}>{error}</Alert>}
        </Box>
      </DialogContent>
      <DialogActions>
        <Button
             onClick={handleClose}
             variant="outlined"
             color="inherit"
             sx={{
               borderColor: "#475569", // slate-600
               color: "#cbd5e1",
               textTransform: "none",
               fontWeight: 500,
               "&:hover": { backgroundColor: "#334155" },
            }}
            >
                {t('common.cancel')}
            </Button>
        <Box sx={{ position: 'relative' }}>
       <Button
            variant="contained"
            color="secondary"
            sx={{
                backgroundColor: "#7c3aed", 
                "&:hover": { backgroundColor: "#6d28d9" },
                textTransform: "none",
                fontWeight: 600,
                color: "white",
                "&.Mui-disabled": {
                color: "white",
                opacity: 1,
                },
            }}
            type="submit"
            onClick={handleSubmit}
            disabled={loading || !file || !templateName}
            >
             {t('templates.modal.createButton')}
        </Button>

          {loading && (
            <CircularProgress
              size={24}
              sx={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                marginTop: '-12px',
                marginLeft: '-12px',
              }}
            />
          )}
        </Box>
      </DialogActions>
    </Dialog>
  );
};

export default NewTemplateModal;