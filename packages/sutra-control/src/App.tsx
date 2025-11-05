import React, { lazy, Suspense } from 'react';
import { BrowserRouter as Router } from 'react-router-dom';
import { ThemeProvider, CssBaseline, CircularProgress, Box as MuiBox } from '@mui/material';
import theme from './theme';

// 🔥 PRODUCTION: Lazy load Layout for better initial bundle size
const Layout = lazy(() => import('./components/Layout').then(m => ({ default: m.Layout })));

// Loading fallback component
const LoadingFallback = () => (
  <MuiBox
    sx={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      height: '100vh',
      bgcolor: 'background.default',
    }}
  >
    <CircularProgress />
  </MuiBox>
);

function App() {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <MuiBox sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
          <Suspense fallback={<LoadingFallback />}>
            <Layout />
          </Suspense>
        </MuiBox>
      </Router>
    </ThemeProvider>
  );
}

export default App;