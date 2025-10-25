import React from 'react';
import { Routes, Route, useLocation, Navigate } from 'react-router-dom';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  useMediaQuery,
  useTheme,
  Avatar,
  Chip,
  Stack,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Brightness4 as ThemeIcon,
  Notifications as NotificationsIcon,
  Settings as SettingsIcon,
  AccountCircle as AccountIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';

import { useAppStore } from '../../store';
import { Sidebar } from './Sidebar';
import { Dashboard } from '../Dashboard';
import { Components } from '../Components';
import { Analytics } from '../Analytics';
import { KnowledgeGraph } from '../KnowledgeGraph';
import { Reasoning } from '../Reasoning';
import SemanticExplorer from '../Semantic';
import { Settings } from '../Settings';
import Grid from '../Grid';
import BulkIngester from '../BulkIngester';
import { ConnectionStatus } from '../ConnectionStatus';

const DRAWER_WIDTH = 280;

export const Layout: React.FC = () => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  
  const { sidebarOpen, setSidebarOpen, connection } = useAppStore();
  const location = useLocation();

  const handleDrawerToggle = () => {
    setSidebarOpen(!sidebarOpen);
  };

  const getPageTitle = () => {
    const routes: Record<string, string> = {
      '/': 'Dashboard',
      '/components': 'Components',
      '/analytics': 'Analytics',
      '/knowledge': 'Knowledge Graph',
      '/reasoning': 'Reasoning Engine',
      '/semantic': 'Semantic Explorer',
      '/bulk-ingester': 'Bulk Data Ingestion',
      '/grid': 'Grid Management',
      '/settings': 'Settings',
    };
    return routes[location.pathname] || 'Sutra AI Control Center';
  };

  return (
    <Box sx={{ display: 'flex', width: '100%', height: '100%' }}>
      {/* App Bar */}
      <AppBar
        position="fixed"
        sx={{
          zIndex: theme.zIndex.drawer + 1,
          width: { md: `calc(100% - ${sidebarOpen ? DRAWER_WIDTH : 0}px)` },
          ml: { md: `${sidebarOpen ? DRAWER_WIDTH : 0}px` },
          transition: theme.transitions.create(['width', 'margin'], {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.leavingScreen,
          }),
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="toggle drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { md: 'none' } }}
          >
            <MenuIcon />
          </IconButton>

          <Typography
            variant="h6"
            noWrap
            component="div"
            sx={{
              flexGrow: 1,
              background: 'linear-gradient(135deg, #6366f1 0%, #06b6d4 100%)',
              backgroundClip: 'text',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              fontWeight: 700,
            }}
          >
            {getPageTitle()}
          </Typography>

          <Stack direction="row" spacing={1} alignItems="center">
            <ConnectionStatus />
            
            <IconButton color="inherit">
              <NotificationsIcon />
            </IconButton>
            
            <IconButton color="inherit">
              <ThemeIcon />
            </IconButton>
            
            <IconButton color="inherit">
              <SettingsIcon />
            </IconButton>
            
            <Avatar
              sx={{
                width: 32,
                height: 32,
                bgcolor: 'primary.main',
                ml: 1,
              }}
            >
              <AccountIcon fontSize="small" />
            </Avatar>
          </Stack>
        </Toolbar>
      </AppBar>

      {/* Sidebar Navigation */}
      <Box
        component="nav"
        sx={{ width: { md: sidebarOpen ? DRAWER_WIDTH : 0 }, flexShrink: { md: 0 } }}
      >
        <Drawer
          variant={isMobile ? 'temporary' : 'persistent'}
          open={sidebarOpen}
          onClose={handleDrawerToggle}
          ModalProps={{
            keepMounted: true, // Better mobile performance
          }}
          sx={{
            '& .MuiDrawer-paper': {
              boxSizing: 'border-box',
              width: DRAWER_WIDTH,
              transition: theme.transitions.create('width', {
                easing: theme.transitions.easing.sharp,
                duration: theme.transitions.duration.enteringScreen,
              }),
            },
          }}
        >
          <Sidebar />
        </Drawer>
      </Box>

      {/* Main Content */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          width: { md: `calc(100% - ${sidebarOpen ? DRAWER_WIDTH : 0}px)` },
          minHeight: '100vh',
          transition: theme.transitions.create(['width', 'margin'], {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.leavingScreen,
          }),
        }}
      >
        <Toolbar /> {/* Spacer for app bar */}
        
        <Box sx={{ p: 3, height: 'calc(100vh - 64px)', overflow: 'auto' }}>
          <AnimatePresence mode="wait">
            <motion.div
              key={location.pathname}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.2 }}
              style={{ height: '100%' }}
            >
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/components" element={<Components />} />
                <Route path="/analytics" element={<Analytics />} />
                <Route path="/knowledge" element={<KnowledgeGraph />} />
                <Route path="/reasoning" element={<Reasoning />} />
                <Route path="/semantic" element={<SemanticExplorer />} />
                <Route path="/bulk-ingester" element={<BulkIngester />} />
                <Route path="/grid" element={<Grid />} />
                <Route path="/settings" element={<Settings />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </motion.div>
          </AnimatePresence>
        </Box>
      </Box>
    </Box>
  );
};