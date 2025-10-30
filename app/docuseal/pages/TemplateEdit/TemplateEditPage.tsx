import  { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import upstashService from '../../ConfigApi/upstashService';
import SignaturePad from './SignaturePad';
import CreateTemplateButton from '../../components/CreateTemplateButton';
import PdfFullView from './PdfFullView';
import { Dialog, DialogContent, DialogActions, Button, IconButton,
   Typography, LinearProgress, TextField, Checkbox, Radio, 
   RadioGroup, FormControlLabel, Select, MenuItem, 
   FormControl, InputLabel, Box, Card, 
   CardMedia, Link } from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import toast from 'react-hot-toast';
import { useNavigate } from 'react-router-dom';
interface TemplateField {
  id: number;
  template_id: number;
  name: string;
  field_type: string;
  required: boolean;
  display_order: number;
  position: {
    x: number;
    y: number;
    width: number;
    height: number;
    page: number;
    suggested?: string;
    allow_custom?: boolean;
  };
  options?: any;
  partner?: string;
  created_at: string;
  updated_at: string;
}

interface TemplateInfo {
  id: number; 
  name: string;
  slug: string;
  user_id: number;
  document: {
    filename: string;
    content_type: string;
    size: number;
    url: string;
  };
}


const TemplateEditPage = () => {
  const { token } = useParams<{ token: string }>();
  const [templateInfo, setTemplateInfo] = useState<TemplateInfo | null>(null);
  const [fields, setFields] = useState<TemplateField[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [texts, setTexts] = useState<Record<number, string>>({});
  const [currentFieldIndex, setCurrentFieldIndex] = useState(0);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [page, setPage] = useState(1);
  const [pendingUploads, setPendingUploads] = useState<Record<number, File>>({});
  const navigate = useNavigate();
  const [fileUploading, setFileUploading] = useState(false);
  const uploadFile = async (file: File): Promise<string | null> => {
    try {
      setFileUploading(true);
      const formData = new FormData();
      formData.append('file', file);

      console.log('Uploading file:', file.name, 'Size:', file.size);
      const response = await upstashService.uploadPublicFile(formData);
      console.log('Upload response:', response);

      // Extract data from axios response
      const data = response.data;
      if (data && data.success && data.data && data.data.url) {
        console.log('Upload successful, URL:', data.data.url);
        return data.data.url;
      } else {
        console.error('File upload failed - invalid response:', data);
        return null;
      }
    } catch (error) {
      console.error('File upload error:', error);
      // Log more details about the error
      if (error.response) {
        console.error('Response status:', error.response.status);
        console.error('Response data:', error.response.data);
      }
      return null;
    } finally {
      setFileUploading(false);
    }
  };

  const fetchTemplateFields = useCallback(async () => {
    try {
      const data = await upstashService.getSubmissionFields(token);
      if (data.success) {
        setTemplateInfo(data.data.template_info);
        
        // Convert position from pixels to decimal (0-1) if needed
        const processedFields = data.data.template_fields.map((field: TemplateField) => {
          if (field.position && typeof field.position.x === 'number') {
            // Use default page dimensions since we don't have actual page dimensions here
            const pageWidth = 600; // Default A4 width in pixels
            const pageHeight = 800; // Default A4 height in pixels
            
            // Check if position is in pixels (values > 1) or already in decimal (0-1)
            if (field.position.x > 1 || field.position.y > 1 || field.position.width > 1 || field.position.height > 1) {
              // Position is in pixels, convert to decimal (0-1)
              return {
                ...field,
                position: {
                  ...field.position,
                  x: field.position.x / pageWidth,
                  y: field.position.y / pageHeight,
                  width: field.position.width / pageWidth,
                  height: field.position.height / pageHeight
                }
              };
            }
            // Already in decimal format
            return field;
          }
          return field;
        });
        
        setFields(processedFields);
        console.log('Processed fields for signing:', processedFields);
      } else {
        setError(data.message || 'Failed to fetch template fields.');
      }
    } catch (err) {
      console.error('Fetch error:', err);
      setError(`Failed to load template. Please check your connection and try again. Details: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    fetchTemplateFields();
  }, [fetchTemplateFields]);

  const onFieldClick = (field: TemplateField) => {
    const globalIndex = fields.findIndex(f => f.id === field.id);
    setCurrentFieldIndex(globalIndex);
    setPage(field.position.page);
    setIsModalOpen(true);
  };

  const handleTextChange = (fieldId: number, value: string, isMultiple: boolean = false, checked?: boolean) => {
    if (isMultiple && checked !== undefined) {
      // Handle multiple selection
      const currentSelections = texts[fieldId] ? texts[fieldId].split(',') : [];
      let newSelections;
      if (checked) {
        newSelections = [...currentSelections, value];
      } else {
        newSelections = currentSelections.filter(item => item !== value);
      }
      setTexts(prev => ({ ...prev, [fieldId]: newSelections.join(',') }));
    } else {
      setTexts(prev => ({ ...prev, [fieldId]: value }));
    }
  };

  const handleNext = () => {
    if (currentFieldIndex < fields.length - 1) {
      const nextIndex = currentFieldIndex + 1;
      setCurrentFieldIndex(nextIndex);
      const nextField = fields[nextIndex];
      if (nextField.position.page !== page) {
        setPage(nextField.position.page);
      }
    }
  };

  const handlePrevious = () => {
    if (currentFieldIndex > 0) {
      const prevIndex = currentFieldIndex - 1;
      setCurrentFieldIndex(prevIndex);
      const prevField = fields[prevIndex];
      if (prevField.position.page !== page) {
        setPage(prevField.position.page);
      }
    }
  };

  const handleComplete = async () => {
    // Upload any pending files first
    const finalTexts = { ...texts };
    for (const [fieldId, file] of Object.entries(pendingUploads)) {
      try {
        const fileUrl = await uploadFile(file);
        if (fileUrl) {
          finalTexts[parseInt(fieldId)] = fileUrl;
          // Cleanup blob URL after successful upload
          const blobUrl = texts[parseInt(fieldId)];
          if (blobUrl && blobUrl.startsWith('blob:')) {
            URL.revokeObjectURL(blobUrl);
          }
        } else {
          console.error(`Upload failed for field ${fieldId}`);
          toast.error(`Failed to upload file for field ${fieldId}`);
          return;
        }
      } catch (error) {
        console.error(`Upload error for field ${fieldId}:`, error);
        toast.error(`Upload error for field ${fieldId}`);
        return;
      }
    }

    // Validate required fields
    const missingFields = fields.filter(field => {
      if (!field.required) return false;
      const value = finalTexts[field.id];
      if (!value) return true;
      // For signature fields, check if it's not empty
      if (field.field_type === 'signature') {
        return !value || value.trim() === '';
      }
      // For radio fields, check if an option is selected
      if (field.field_type === 'radio') {
        return !value.trim();
      }
      // For multiple fields, check if at least one option is selected
      if (field.field_type === 'multiple') {
        return !value || value.split(',').filter(item => item.trim()).length === 0;
      }
      // For checkbox fields, 'false' is a valid value
      if (field.field_type === 'checkbox') {
        return false;
      }
      // For other fields, check if trimmed value exists
      return !value.trim();
    });
    if (missingFields.length > 0) {
      toast.error(`Please fill in the required fields: ${missingFields.map(f => f.name).join(', ')}`);
      return;
    }


    try {
      const signatures = fields.map(field => ({
        field_id: field.id,
        signature_value: finalTexts[field.id] || ''
      }));

      const data = await upstashService.bulkSign(token, {
        signatures,
        user_agent: navigator.userAgent
      });
      console.log(data)
      if (data.success) {
        toast.success(data?.message);
        // Navigate to template view page
        navigate(`/templates/${templateInfo?.id}`);
        // Clear pending uploads after successful submission
        setPendingUploads({});
        // Redirect or show success message
      } else {
        toast.error(`Error: ${data.message}`);
      }
    } catch (err) {
      console.error('Submit error:', err);
      toast.error('Unable to submit signature. Please try again.');
    }
  };

  const currentField = fields[currentFieldIndex];
  const isLastField = currentFieldIndex === fields.length - 1;
  if (loading) return <div className="flex items-center justify-center min-h-screen">Loading...</div>;
  if (error) return <div className="text-red-500 text-center p-4">{error}</div>;
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">{templateInfo?.name}</h1>
      {/* PDF Full View */}
      <PdfFullView
        templateInfo={templateInfo}
        fields={fields}
        page={page}
        onPageChange={setPage}
        onFieldClick={onFieldClick}
        texts={texts}
        token={token}
      />

      {/* Form Modal */}
      <Dialog
        open={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogContent sx={{ position: 'relative' }}>
          <IconButton
            onClick={() => setIsModalOpen(false)}
            sx={{
              position: 'absolute',
              top: 8,
              right: 8,
              color: 'grey.500',
            }}
          >
            <CloseIcon />
          </IconButton>
          <div className="mb-4">
            <Typography variant="body2" sx={{ mb: 1 }}>
              Field {currentFieldIndex + 1} / {fields.length}
            </Typography>
            <LinearProgress
              variant="determinate"
              value={((currentFieldIndex + 1) / fields.length) * 100}
              sx={{
                height: 8,
                borderRadius: 4,
                backgroundColor: 'grey.300',
                '& .MuiLinearProgress-bar': {
                  backgroundColor: 'primary.main',
                  borderRadius: 4,
                },
              }}
            />
          </div>

          {currentField && (
            <div className="mb-6">
              {currentField.field_type === 'date' ? (
                <TextField
                  type="date"
                  value={texts[currentField.id] || ''}
                  onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                  fullWidth
                  required={currentField.required}
                  autoFocus
                  InputLabelProps={{ shrink: true }}
                />
              ) : currentField.field_type === 'checkbox' ? (
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={texts[currentField.id] === 'true'}
                      onChange={(e) => handleTextChange(currentField.id, e.target.checked ? 'true' : 'false')}
                      required={currentField.required}
                      autoFocus
                    />
                  }
                  label={currentField.name}
                />
              ) : currentField.field_type === 'signature' || currentField.field_type === 'initials' ? (
                <SignaturePad
                  onSave={(dataUrl) => handleTextChange(currentField.id, dataUrl)}
                  onClear={() => handleTextChange(currentField.id, '')}
                  initialData={texts[currentField.id]}
                  onFileSelected={(file) => {
                    if (file) {
                      // Create blob URL for immediate preview
                      const blobUrl = URL.createObjectURL(file);
                      // Update texts with blob URL for preview
                      handleTextChange(currentField.id, blobUrl);
                      // Store file for later upload
                      setPendingUploads(prev => ({ ...prev, [currentField.id]: file }));
                    } else {
                      setPendingUploads(prev => {
                        const newUploads = { ...prev };
                        delete newUploads[currentField.id];
                        return newUploads;
                      });
                    }
                  }}
                />
              ) : currentField.field_type === 'image' ? (
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                  <input
                    type="file"
                    accept="image/*"
                    onChange={async (e) => {
                      const file = e.target.files?.[0];
                      if (file) {
                        const maxSize = 10 * 1024 * 1024; // 10MB
                        if (file.size > maxSize) {
                          toast.error(`File too large. Maximum allowed size is ${maxSize / (1024 * 1024)}MB. Current file: ${(file.size / (1024 * 1024)).toFixed(2)}MB.`);
                          return;
                        }
                        const imageUrl = await uploadFile(file);
                        if (imageUrl) {
                          handleTextChange(currentField.id, imageUrl);
                        } else {
                          toast.error('Unable to upload image. Please try again.');
                        }
                      }
                    }}
                    style={{ display: 'none' }}
                    id={`image-upload-${currentField.id}`}
                    disabled={fileUploading}
                    required={currentField.required}
                  />
                  <label htmlFor={`image-upload-${currentField.id}`}>
                    <Button variant="outlined" component="span" fullWidth disabled={fileUploading}>
                      Select image
                    </Button>
                  </label>
                  <Typography variant="caption" color="text.secondary">
                    Kích thước tối đa: 10MB
                  </Typography>
                  {fileUploading && (
                    <Typography variant="body2" color="primary">
                      Uploading image...
                    </Typography>
                  )}
                  {texts[currentField.id] && (
                    <Box sx={{ mt: 1 }}>
                      <Card sx={{ maxWidth: 200 }}>
                        <CardMedia
                          component="img"
                          height="140"
                          image={texts[currentField.id]}
                          alt="Uploaded preview"
                        />
                      </Card>
                      <Button
                        size="small"
                        color="error"
                        onClick={() => handleTextChange(currentField.id, '')}
                        sx={{ mt: 1 }}
                      >
                        Delete image
                      </Button>
                    </Box>
                  )}
                </Box>
              ) : currentField.field_type === 'file' ? (
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                  <input
                    type="file"
                    onChange={async (e) => {
                      const file = e.target.files?.[0];
                      if (file) {
                        const maxSize = 10 * 1024 * 1024; // 10MB
                        if (file.size > maxSize) {
                          toast.error(`File too large. Maximum allowed size is ${maxSize / (1024 * 1024)}MB. Current file: ${(file.size / (1024 * 1024)).toFixed(2)}MB.`);
                          return;
                        }
                        const fileUrl = await uploadFile(file);
                        if (fileUrl) {
                          handleTextChange(currentField.id, fileUrl);
                        } else {
                          toast.error('Unable to upload file. Please try again.');
                        }
                      }
                    }}
                    style={{ display: 'none' }}
                    id={`file-upload-${currentField.id}`}
                    disabled={fileUploading}
                    required={currentField.required}
                  />
                  <label htmlFor={`file-upload-${currentField.id}`}>
                    <Button variant="outlined" component="span" fullWidth disabled={fileUploading}>
                      Select file
                    </Button>
                  </label>
                  <Typography variant="caption" color="text.secondary">
                    Kích thước tối đa: 10MB
                  </Typography>
                  {fileUploading && (
                    <Typography variant="body2" color="primary">
                      Uploading file...
                    </Typography>
                  )}
                  {texts[currentField.id] && (
                    <Box sx={{ mt: 1 }}>
                      <Link href={texts[currentField.id]} download underline="hover">
                        {decodeURIComponent(texts[currentField.id].split('/').pop() || 'File')}
                      </Link>
                      <Button
                        size="small"
                        color="error"
                        onClick={() => handleTextChange(currentField.id, '')}
                        sx={{ ml: 1 }}
                      >
                        Delete file
                      </Button>
                    </Box>
                  )}
                </Box>
              ) : currentField.field_type === 'number' ? (
                <TextField
                  type="number"
                  value={texts[currentField.id] || ''}
                  onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                  fullWidth
                  placeholder={`Enter ${currentField.name}`}
                  required={currentField.required}
                  autoFocus
                />
              ) : currentField.field_type === 'radio' ? (
                <RadioGroup
                  value={texts[currentField.id] || ''}
                  onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                >
                  {currentField.options?.map((option: string, index: number) => (
                    <FormControlLabel
                      key={index}
                      value={option}
                      control={<Radio required={currentField.required} />}
                      label={option}
                    />
                  ))}
                </RadioGroup>
              ) : currentField.field_type === 'select' ? (
                <FormControl fullWidth required={currentField.required}>
                  <InputLabel>Select an option</InputLabel>
                  <Select
                    value={texts[currentField.id] || ''}
                    label="Select an option"
                    onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                    MenuProps={{
                      PaperProps: {
                        sx: {
                          color:'black'
                        }
                      }
                    }}
                  >
                    {currentField.options?.map((option: string, index: number) => (
                      <MenuItem key={index} value={option}>
                        {option}
                      </MenuItem>
                    ))}
                  </Select>
                </FormControl>
              ) : currentField.field_type === 'cells' ? (
                <TextField
                  value={texts[currentField.id] || ''}
                  onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                  fullWidth
                  placeholder={`Enter up to ${currentField.options?.columns || 1} characters`}
                  required={currentField.required}
                  autoFocus
                  inputProps={{
                    maxLength: currentField.options?.columns || 1,
                  }}
                />
              ) : (
                <TextField
                  value={texts[currentField.id] || ''}
                  onChange={(e) => handleTextChange(currentField.id, e.target.value)}
                  fullWidth
                  placeholder={`Enter ${currentField.name}`}
                  required={currentField.required}
                  autoFocus
                />
              )}
            </div>
          )}
        </DialogContent>
        <DialogActions>
            <Button
              onClick={handlePrevious}
              variant="outlined"
              color="inherit"
              sx={{
                borderColor: "#475569",
                color: "#cbd5e1",
                textTransform: "none",
                fontWeight: 500,
                "&:hover": { backgroundColor: "#334155" },
              }}
            >
                Previous
            </Button>
          {!isLastField ? (
            <CreateTemplateButton
              onClick={handleNext}
              text="Next"

            />
          ) : (
            <CreateTemplateButton onClick={() => handleComplete()} text="Complete" />
          )}
        </DialogActions>
      </Dialog>
    </div>
  );
};

export default TemplateEditPage;
