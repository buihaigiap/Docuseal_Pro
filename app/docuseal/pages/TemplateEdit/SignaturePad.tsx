import React, { useRef, useEffect, useState } from 'react';
import SignatureCanvas from 'react-signature-canvas';
import { TextField, Button, Box, Typography, Fade, Stack } from '@mui/material';
import { PenLine, Type, Eraser } from 'lucide-react';

interface SignaturePadProps {
  onSave: (dataUrl: string) => void;
  onClear?: () => void;
  initialData?: string;
}

const SignaturePad: React.FC<SignaturePadProps> = ({ onSave, onClear, initialData }) => {
  const sigPadRef = useRef<SignatureCanvas>(null);
  const [isEmpty, setIsEmpty] = useState(true);
  const [mode, setMode] = useState<'draw' | 'type'>('draw');
  const [typedText, setTypedText] = useState('');

  useEffect(() => {
    if (initialData) {
      if (initialData.startsWith('data:image/')) {
        setMode('draw');
        sigPadRef.current?.fromDataURL(initialData);
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

  const handleClear = () => {
    if (mode === 'draw') {
      sigPadRef.current?.clear();
      setIsEmpty(true);
    } else {
      setTypedText('');
    }
    onClear?.();
  };

  const handleSave = () => {
    if (mode === 'draw' && sigPadRef.current) {
      const vectorData = JSON.stringify(sigPadRef.current.toData());
      onSave(vectorData);
    } else if (mode === 'type') {
      onSave(typedText);
    }
  };

  const handleBegin = () => setIsEmpty(false);

  const handleModeChange = (newMode: 'draw' | 'type') => {
    setMode(newMode);
    if (newMode === 'draw') setTypedText('');
    else sigPadRef.current?.clear();
    setIsEmpty(true);
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
    </Box>
  );
};

export default SignaturePad;
