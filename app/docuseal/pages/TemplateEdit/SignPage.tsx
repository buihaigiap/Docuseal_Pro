import React, { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Submitter, TemplateField } from '../../types';
import SignaturePad from './SignaturePad';
import DocumentViewer from '../../components/PdfViewer';
import upstashService from '../../ConfigApi/upstashService';
import { Box, Button, CircularProgress, Typography, Paper, Grid, TextField, Modal, Alert, useMediaQuery } from '@mui/material';
import { useTheme } from '@mui/material/styles';

const SignPage = () => {
  const { token: signingToken } = useParams<{ token: string }>();
  const [submitterInfo, setSubmitterInfo] = useState<Submitter | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [signatures, setSignatures] = useState<Record<number, string>>({});
  const [fieldValues, setFieldValues] = useState<Record<number, string>>({});
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [currentFieldId, setCurrentFieldId] = useState<number | null>(null);
  const [currentFieldType, setCurrentFieldType] = useState<'signature' | 'radio' | 'multiple' | 'file' | null>(null);
  const [currentField, setCurrentField] = useState<TemplateField | null>(null);
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));

  const fetchSubmitterInfo = useCallback(async () => {
    try {
      const data = await upstashService.getSubmitterInfo(signingToken);
      if (data.success) {
        setSubmitterInfo(data.data);
        // Pre-fill field values if already filled
        const existingValues: Record<number, string> = {};
        const existingSigs: Record<number, string> = {};
        data.data.bulk_signatures?.forEach((sig: {field_id: number; field_info: TemplateField; signature_value: string}) => {
          if (sig.field_info.field_type === 'signature') {
            existingSigs[sig.field_id] = sig.signature_value;
          } else {
            existingValues[sig.field_id] = sig.signature_value;
          }
        });
        setSignatures(existingSigs);
        setFieldValues(existingValues);

      } else {
        setError(data.message || 'Failed to fetch signing information.');
      }
    } catch (err) {
      setError('An unexpected error occurred.');
    } finally {
      setLoading(false);
    }
  }, [signingToken]);

  useEffect(() => {
    fetchSubmitterInfo();
  }, [fetchSubmitterInfo]);

  const openFieldModal = (field: TemplateField) => {
    setCurrentFieldId(field.id);
    setCurrentFieldType(field.field_type as 'signature' | 'radio' | 'multiple' | 'file');
    setCurrentField(field);
    setIsModalOpen(true);
  };

  const handleSaveSignature = (dataUrl: string) => {
    if (currentFieldId) {
      setSignatures({ ...signatures, [currentFieldId]: dataUrl });
    }
    setIsModalOpen(false);
    setCurrentFieldId(null);
    setCurrentFieldType(null);
    setCurrentField(null);
  };

  const handleSaveRadioSelection = (selectedValue: string) => {
    if (currentFieldId) {
      setFieldValues({ ...fieldValues, [currentFieldId]: selectedValue });
    }
    setIsModalOpen(false);
    setCurrentFieldId(null);
    setCurrentFieldType(null);
    setCurrentField(null);
  };

  const handleSaveFile = (fileUrl: string) => {
    if (currentFieldId) {
      setFieldValues({ ...fieldValues, [currentFieldId]: fileUrl });
    }
    setIsModalOpen(false);
    setCurrentFieldId(null);
    setCurrentFieldType(null);
    setCurrentField(null);
  };

  const handleSubmitSignatures = async () => {
    const allFieldValues = { ...signatures, ...fieldValues };
    const signaturePayload = Object.entries(allFieldValues).map(([field_id, signature_value]) => ({
        field_id: parseInt(field_id),
        signature_value
    }));

    try {
      const data = await upstashService.bulkSign(signingToken, { signatures: signaturePayload, ip_address: '127.0.0.1', user_agent: navigator.userAgent });
      if(data.success) {
        alert("Document signed successfully!");
        fetchSubmitterInfo();
      } else {
        alert(`Error: ${data.message}`);
      }
    } catch(err) {
      alert('An unexpected error occurred during submission.');
    }
  };
  
  if (loading) return <div className="text-center">Loading document...</div>;
  if (error) return <Alert severity="error" sx={{ m: 2 }}>{error}</Alert>;
  if (!submitterInfo) return <Alert severity="info" sx={{ m: 2 }}>No submitter information found.</Alert>;

  return (
    <Box sx={{ display: 'flex', flexDirection: isMobile ? 'column' : 'row', height: 'calc(100vh - 64px)' }}>
      <Box sx={{ flex: 1, overflowY: 'auto', p: isMobile ? 1 : 2 }}>
        <DocumentViewer
          fileUrl={submitterInfo.template.documents?.[0]?.url || ''}
          fields={submitterInfo.template.fields || []}
          signatures={signatures}
          fieldValues={fieldValues}
          onFieldClick={openFieldModal}
          isSignMode={true}
        />
      </Box>
      <Paper sx={{ 
        width: isMobile ? '100%' : '300px', 
        p: 2, 
        display: 'flex', 
        flexDirection: 'column', 
        gap: 2,
        borderLeft: isMobile ? 'none' : `1px solid ${theme.palette.divider}`,
        borderTop: isMobile ? `1px solid ${theme.palette.divider}` : 'none',
      }}>
        <Typography variant="h6">Signing Panel</Typography>
        <Typography variant="body2">
          Signed by: <strong>{submitterInfo.name}</strong> ({submitterInfo.email})
        </Typography>
        <Button 
          variant="contained" 
          color="primary" 
          onClick={handleSubmitSignatures}
          disabled={Object.keys(signatures).length === 0 && Object.keys(fieldValues).length === 0}
        >
          Submit Document
        </Button>
      </Paper>

      <Modal open={isModalOpen} onClose={() => setIsModalOpen(false)}>
        <Box sx={{
          bgcolor: 'background.paper',
          borderRadius: 2,
          boxShadow: 24,
          p: 4,
          maxWidth: 400,
          width: '90%',
          mx: 'auto',
          mt: '10%',
        }}>
          {currentFieldType === 'signature'  ? (
            <>
              <Typography variant="h6" component="h2" gutterBottom>
                Provide Your Signature
              </Typography>
              <SignaturePad 
                onSave={handleSaveSignature}
                onClear={() => {
                    if(currentFieldId) {
                        const newSigs = {...signatures};
                        delete newSigs[currentFieldId];
                        setSignatures(newSigs);
                    }
                }}
              />
            </>
          ) : currentFieldType === 'radio' ? (
            <>
              <Typography variant="h6" component="h2" gutterBottom>
                Select {currentField.name}
              </Typography>
              <div className="space-y-3">
                {currentField.options?.map((option, index) => (
                  <label key={index} className="flex items-center space-x-3 cursor-pointer">
                    <input
                      type="radio"
                      name={`radio-${currentField.id}`}
                      value={option}
                      checked={fieldValues[currentField.id] === option}
                      onChange={() => handleSaveRadioSelection(option)}
                      className="w-4 h-4 text-indigo-600 bg-gray-700 border-gray-600 focus:ring-indigo-500"
                    />
                    <span className="text-gray-300">{option}</span>
                  </label>
                ))}
              </div>
            </>
          ) : currentFieldType === 'multiple' ? (
            <>
              <Typography variant="h6" component="h2" gutterBottom>
                Select {currentField.name}
              </Typography>
              <div className="space-y-3">
                {currentField.options?.map((option, index) => {
                  const currentSelections = fieldValues[currentField.id] ? fieldValues[currentField.id].split(',') : [];
                  const isChecked = currentSelections.includes(option);
                  return (
                    <label key={index} className="flex items-center space-x-3 cursor-pointer">
                      <input
                        type="checkbox"
                        value={option}
                        checked={isChecked}
                        onChange={(e) => {
                          const newSelections = e.target.checked 
                            ? [...currentSelections, option]
                            : currentSelections.filter(item => item !== option);
                          handleSaveRadioSelection(newSelections.join(','));
                        }}
                        className="w-4 h-4 text-indigo-600 bg-gray-700 border-gray-600 rounded focus:ring-indigo-500"
                      />
                      <span className="text-gray-300">{option}</span>
                    </label>
                  );
                })}
              </div>
            </>
          ) : currentFieldType === 'file' ? (
            <>
              <Typography variant="h6" component="h2" gutterBottom>
                Upload {currentField.name}
              </Typography>
              <input
                type="file"
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (file) {
                    try {
                      const uploadFormData = new FormData();
                      uploadFormData.append('file', file);
                      uploadFormData.append('file_type', 'attachment');

                      const uploadData = await upstashService.uploadFile(uploadFormData);
                      if (uploadData.success) {
                        handleSaveFile(uploadData.data.url);
                      } else {
                        alert('Failed to upload file.');
                      }
                    } catch (err) {
                      alert('Upload error.');
                    }
                  }
                }}
                className="w-full p-2 bg-gray-700 border border-gray-600 rounded-md text-gray-300"
              />
            </>
          ) : null}
          <Button onClick={() => {
            setIsModalOpen(false);
            setCurrentFieldId(null);
            setCurrentFieldType(null);
            setCurrentField(null);
          }} variant="outlined" color="secondary" sx={{ mt: 2 }}>
            Cancel
          </Button>
        </Box>
      </Modal>
    </Box>
  );
};

export default SignPage;
