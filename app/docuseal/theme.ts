import { createTheme } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    primary: {
      main: '#4F46E5',
    },
    secondary: {
      main: '#EC4899',
    },
    background: {
      default: '#0D071F',
      paper: '#FFFFFF',
    },
    text: {
      primary: '#FFFFFF',
      secondary: '#6B7280',
    },
  },
  typography: {
    fontFamily: 'Inter, sans-serif',
    h1: {
      fontSize: '3rem',
      fontWeight: 700,
    }, 
    h2: {
      fontSize: '2.25rem',
      fontWeight: 700,
    },
    h3: {
        fontSize: '1.875rem',
        fontWeight: 700,
    },
    body1: {
        fontSize: '1rem',
        fontWeight: 400,
    },
  },
  shape: {
    borderRadius: 8,
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: 'none',
          boxShadow: 'none',
          '&:hover': {
            boxShadow: 'none',
          },
        },
      },
    },
    MuiTextField: {
        styleOverrides: {
            root: {
                '& label.Mui-focused': {
                    color: '#4F46E5',
                },
                '& .MuiOutlinedInput-root': {
                    '&.Mui-focused fieldset': {
                        borderColor: '#4F46E5',
                    },
                },
            },
        },
    },
    MuiDialog: {
      defaultProps: {
        PaperProps: {
          sx: {
            backgroundColor: "#1e293b", // slate-800
            borderRadius: 3,
            color: "white",
            boxShadow: "0 0 20px rgba(0,0,0,0.6)",
          },
        },
      },
    },
  },
});

export default theme;
