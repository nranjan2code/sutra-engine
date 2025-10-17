import { createTheme } from '@mui/material/styles'

export const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#6750A4',
      light: '#7965AF',
      dark: '#4F378B',
      contrastText: '#FFFFFF',
    },
    secondary: {
      main: '#625B71',
      light: '#7A7289',
      dark: '#4A4458',
      contrastText: '#FFFFFF',
    },
    background: {
      default: '#FEF7FF',
      paper: '#FFFFFF',
    },
    success: {
      main: '#006E1C',
      light: '#4C8400',
      dark: '#005313',
    },
    info: {
      main: '#0061A4',
      light: '#00629E',
      dark: '#004A77',
    },
  },
  typography: {
    fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
    h4: {
      fontWeight: 600,
      letterSpacing: 0,
    },
    h5: {
      fontWeight: 600,
      letterSpacing: 0,
    },
    h6: {
      fontWeight: 500,
      letterSpacing: 0.15,
    },
    body1: {
      fontSize: '1rem',
      lineHeight: 1.6,
      letterSpacing: 0.15,
    },
    body2: {
      fontSize: '0.875rem',
      lineHeight: 1.5,
      letterSpacing: 0.15,
    },
  },
  shape: {
    borderRadius: 12,
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: 'none',
          borderRadius: 20,
          paddingLeft: 24,
          paddingRight: 24,
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: 16,
          boxShadow: '0px 1px 3px rgba(0, 0, 0, 0.12)',
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          '& .MuiOutlinedInput-root': {
            borderRadius: 12,
          },
        },
      },
    },
  },
})
