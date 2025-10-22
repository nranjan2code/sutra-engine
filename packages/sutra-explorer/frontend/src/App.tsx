import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ThemeProvider, createTheme, CssBaseline, Box } from '@mui/material';
import { useState } from 'react';

// Components
import Sidebar from './components/Sidebar';
import Dashboard from './pages/Dashboard';
import ConceptBrowser from './pages/ConceptBrowser';
import GraphExplorer from './pages/GraphExplorer';
import SearchPage from './pages/SearchPage';
import PathFinder from './pages/PathFinder';

const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#6366f1',
    },
    secondary: {
      main: '#f59e0b',
    },
    background: {
      default: '#0f172a',
      paper: '#1e293b',
    },
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
  },
});

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Box sx={{ display: 'flex', height: '100vh' }}>
          <Sidebar open={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />
          <Box
            component="main"
            sx={{
              flexGrow: 1,
              p: 3,
              overflow: 'auto',
              ml: sidebarOpen ? 0 : '-240px',
              transition: 'margin 0.3s',
            }}
          >
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/concepts" element={<ConceptBrowser />} />
              <Route path="/graph" element={<GraphExplorer />} />
              <Route path="/search" element={<SearchPage />} />
              <Route path="/pathfinder" element={<PathFinder />} />
            </Routes>
          </Box>
        </Box>
      </Router>
    </ThemeProvider>
  );
}

export default App;
