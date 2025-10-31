import { createTheme } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    primary: { main: '#4F46E5' },
    secondary: { main: '#EC4899' },
    background: {
      default: '#0D071F',
      paper: '#1E1B3A',
    },
    text: {
      primary: '#FFFFFF',
      secondary: '#9CA3AF',
    },
  },
  typography: {
    fontFamily: 'Inter, sans-serif',
    h1: { fontSize: '3rem', fontWeight: 700 },
    h2: { fontSize: '2.25rem', fontWeight: 700 },
    h3: { fontSize: '1.875rem', fontWeight: 700 },
    body1: { fontSize: '1rem', fontWeight: 400 },
  },
  shape: { borderRadius: 8 },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: 'none',
          boxShadow: 'none',
          '&:hover': { boxShadow: 'none' },
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          '& label.Mui-focused': { color: '#FFFFFF' },
        },
      },
    },
    MuiInputLabel: {
      styleOverrides: {
        root: {
          color: '#FFFFFF',
          '&.Mui-focused': { color: '#FFFFFF' },
          '&.Mui-disabled': { color: 'rgba(255,255,255,0.5)' },
        },
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        input: {
          color: '#FFFFFF',
        },
        notchedOutline: {
          borderColor: 'rgba(255,255,255,0.2)',
        },
        root: {
          '&:hover .MuiOutlinedInput-notchedOutline': {
            borderColor: 'rgba(255,255,255,0.5)',
          },
          '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
            borderColor: '#4F46E5',
          },
          '&.Mui-disabled .MuiOutlinedInput-notchedOutline': {
            borderColor: 'rgba(255,255,255,0.1)',
          },
        },
      },
    },
    MuiSelect: {
      styleOverrides: {
        root: {
          width: '100%', // 🔥 luôn full width
        },
        icon: {
          color: '#FFFFFF',
        },
        outlined: {
          color: '#FFFFFF',
        },
      },
    },
    MuiMenu: {
      styleOverrides: {
        paper: {
          backgroundColor: '#1E1B3A', // menu nền tối
          color: '#FFFFFF',
          border: '1px solid rgba(255,255,255,0.15)',
        },
      },
    },
    MuiMenuItem: {
      styleOverrides: {
        root: {
          '&:hover': {
            backgroundColor: 'rgba(79,70,229,0.2)', // hover tím nhạt
          },
          '&.Mui-selected': {
            backgroundColor: 'rgba(79,70,229,0.35)',
          },
        },
      },
    },
    MuiDialog: {
      defaultProps: {
        PaperProps: {
          sx: {
            backgroundColor: '#1E1B3A',
            borderRadius: 3,
            color: 'white',
            boxShadow: '0 0 20px rgba(0,0,0,0.6)',
          },
        },
      },
    },
  },
});

export default theme;
