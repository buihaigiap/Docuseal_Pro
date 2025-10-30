import React, { useRef, useEffect, useState } from 'react';
import SignatureCanvas from 'react-signature-canvas';
import { TextField, Button, Box, Typography, Fade, Stack, Card, CardMedia } from '@mui/material';
import { PenLine, Type, Eraser, Upload } from 'lucide-react';
import upstashService from '../../ConfigApi/upstashService';
import toast from 'react-hot-toast';

interface SignaturePadProps {
  onSave: (dataUrl: string) => void;
  onClear?: () => void;
  initialData?: string;
  onFileSelected?: (file: File | null) => void; // New prop for file handling
  onUploadComplete?: () => void; // New prop to notify when upload is complete
}

const SignaturePad: React.FC<SignaturePadProps> = ({ onSave, onClear, initialData, onFileSelected, onUploadComplete }) => {
  const sigPadRef = useRef<SignatureCanvas>(null);
  const [isEmpty, setIsEmpty] = useState(true);
  const [mode, setMode] = useState<'draw' | 'type' | 'upload'>('draw');
  const [typedText, setTypedText] = useState('');
  const [uploadedImage, setUploadedImage] = useState<string>('');
  const [uploading, setUploading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  useEffect(() => {
    if (initialData) {
      if (initialData.startsWith('data:image/')) {
        setMode('draw');
        sigPadRef.current?.fromDataURL(initialData);
        setIsEmpty(false);
      } else if (initialData.startsWith('blob:')) {
        // Handle local blob URLs from previous sessions
        setMode('upload');
        setUploadedImage(initialData);
        setIsEmpty(false);
      } else if (initialData.startsWith('http') || initialData.startsWith('/')) {
        setMode('upload');
        setUploadedImage(initialData);
        setIsEmpty(false);
      } else {
        try {
          const pointGroups = JSON.parse(initialData);
          setMode('draw');
          sigPadRef.current?.fromData(pointGroups);
          setIsEmpty(false);
        } catch {
          setMode('type');
          setTypedText(initialData);
        }
      }
    }
  }, [initialData]);

  const cleanupBlobUrl = () => {
    if (uploadedImage && uploadedImage.startsWith('blob:')) {
      URL.revokeObjectURL(uploadedImage);
    }
  };

  // Expose cleanup function to parent
  useEffect(() => {
    if (onUploadComplete) {
      // This effect runs when onUploadComplete changes, but we don't need to do anything here
      // The parent will call cleanupBlobUrl when upload is complete
    }
  }, [onUploadComplete]);

  const handleClear = () => {
    if (mode === 'draw') {
      sigPadRef.current?.clear();
      setIsEmpty(true);
    } else if (mode === 'type') {
      setTypedText('');
    } else if (mode === 'upload') {
      setUploadedImage('');
      setIsEmpty(true);
    }
    onClear?.();
  };

  const handleSave = () => {
    if (mode === 'draw' && sigPadRef.current) {
      const vectorData = JSON.stringify(sigPadRef.current.toData());
      onSave(vectorData);
    } else if (mode === 'type') {
      onSave(typedText);
    } else if (mode === 'upload') {
      onSave(uploadedImage);
    }
  };

  const handleBegin = () => setIsEmpty(false);

    const handleModeChange = (newMode: 'draw' | 'type' | 'upload') => {
    setMode(newMode);
    if (newMode === 'draw') {
      setTypedText('');
      setUploadedImage('');
    } else if (newMode === 'type') {
      sigPadRef.current?.clear();
      setUploadedImage('');
    } else if (newMode === 'upload') {
      sigPadRef.current?.clear();
      setTypedText('');
    }
    setIsEmpty(true);
  };

  const handleImageUpload = async (file: File) => {
    // Store file locally for preview
    setSelectedFile(file);
    const blobUrl = URL.createObjectURL(file);
    setUploadedImage(blobUrl); // Create local preview URL
    setIsEmpty(false);
    setMode('upload'); // Set mode to upload
    
    // Notify parent component about file selection (this will update texts)
    onFileSelected?.(file);
  };

  return (
    <Box
      sx={{
        width: 460,
        mx: 'auto',
        bgcolor: 'background.paper',
        boxShadow: '0 6px 20px rgba(0,0,0,0.1)',
        borderRadius: 3,
        p: 3,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
      }}
    >
      <Stack direction="row" spacing={1.5} sx={{ mb: 2 }}>
        <Button
          startIcon={<PenLine size={18} />}
          variant={mode === 'draw' ? 'contained' : 'outlined'}
          color="primary"
          onClick={() => handleModeChange('draw')}
          sx={{ textTransform: 'none', borderRadius: 2, px: 2 }}
        >
          Draw
        </Button>
        <Button
          startIcon={<Type size={18} />}
          variant={mode === 'type' ? 'contained' : 'outlined'}
          color="primary"
          onClick={() => handleModeChange('type')}
          sx={{ textTransform: 'none', borderRadius: 2, px: 2 }}
        >
          Type
        </Button>
        <Button
          startIcon={<Upload size={18} />}
          variant={mode === 'upload' ? 'contained' : 'outlined'}
          color="primary"
          onClick={() => handleModeChange('upload')}
          sx={{ textTransform: 'none', borderRadius: 2, px: 2 }}
        >
          Upload
        </Button>
        <Button
          startIcon={<Eraser size={18} />}
          variant="outlined"
          color="error"
          onClick={handleClear}
          sx={{ textTransform: 'none', borderRadius: 2, px: 2 }}
        >
          Clear
        </Button>
      </Stack>

      <Fade in={mode === 'draw'} unmountOnExit>
        <Box
          sx={{
            border: '2px dashed #ccc',
            borderRadius: 2,
            bgcolor: 'white',
            position: 'relative',
            width: 420,
            height: 200,
            overflow: 'hidden',
            mb: 2,
          }}
        >
          <SignatureCanvas
            ref={sigPadRef}
            canvasProps={{
              width: 420,
              height: 200,
              className: 'signature-canvas cursor-crosshair',
            }}
            penColor="#000"
            onBegin={handleBegin}
            onEnd={handleSave}
          />
          {isEmpty && (
            <Typography
              sx={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                color: '#aaa',
                fontStyle: 'italic',
              }}
            >
              Sign here...
            </Typography>
          )}
        </Box>
      </Fade>

      <Fade in={mode === 'type'} unmountOnExit>
        <Box sx={{ width: 420, mb: 2 }}>
          <TextField
            value={typedText}
            onChange={(e) => setTypedText(e.target.value)}
            onBlur={handleSave}
            placeholder="Type your signature..."
            fullWidth
            variant="outlined"
            sx={{
              mb: 1,
              '& input': {  fontSize: '1.6rem', color: '#000' },
            }}
          />
        </Box>
      </Fade>

      <Fade in={mode === 'upload'} unmountOnExit>
        <Box sx={{ width: 420, mb: 2, display: 'flex', flexDirection: 'column', gap: 2 }}>
          <input
            type="file"
            accept="image/*"
            onChange={async (e) => {
              const file = e.target.files?.[0];
              if (file) {
                const maxSize = 5 * 1024 * 1024; // 5MB for signature images
                if (file.size > maxSize) {
                  toast.error(`File too large. Maximum allowed size is ${maxSize / (1024 * 1024)}MB.`);
                  return;
                }
                await handleImageUpload(file);
              }
            }}
            style={{ display: 'none' }}
            id="signature-image-upload"
            disabled={uploading}
          />
          {!uploadedImage && (
            <label htmlFor="signature-image-upload">
                <Button
                  variant="outlined"
                  component="span"
                  fullWidth
                  disabled={uploading}
                  sx={{ textTransform: 'none', borderRadius: 2, py: 2 }}
                >
                  {uploading ? 'Uploading...' : 'Select Signature Image'}
                </Button>
              </label>
          )}
          

          {uploadedImage && (
            <Box sx={{ mt: 1, display: 'flex', justifyContent: 'center' }}>
              <Card sx={{ maxWidth: 300 }}>
                <CardMedia
                  component="img"
                  height="140"
                  image={uploadedImage}
                  alt="Signature preview"
                  sx={{ objectFit: 'contain' }}
                />
              </Card>
            </Box>
          )}
        </Box>
      </Fade>
    </Box>
  );
};

export default SignaturePad;
