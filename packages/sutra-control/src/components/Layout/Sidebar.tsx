import React from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  Box,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Typography,
  Divider,
  Chip,
  Stack,
  Toolbar,
  Badge,
} from '@mui/material';
import {
  Dashboard as DashboardIcon,
  Memory as ComponentsIcon,
  Analytics as AnalyticsIcon,
  AccountTree as KnowledgeIcon,
  Psychology as ReasoningIcon,
  Settings as SettingsIcon,
  Timeline as TimelineIcon,
  Speed as SpeedIcon,
  Storage as StorageIcon,
  Cloud as CloudIcon,
  GridView as GridIcon,
  CloudUpload as BulkIngesterIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';

import { useAppStore } from '../../store';

interface NavigationItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  path: string;
  badge?: number;
  description?: string;
}

const navigationItems: NavigationItem[] = [
  {
    id: 'dashboard',
    label: 'Dashboard',
    icon: <DashboardIcon />,
    path: '/',
    description: 'System overview and metrics',
  },
  {
    id: 'components',
    label: 'Components',
    icon: <ComponentsIcon />,
    path: '/components',
    description: 'Manage system components',
  },
  {
    id: 'analytics',
    label: 'Analytics',
    icon: <AnalyticsIcon />,
    path: '/analytics',
    description: 'Performance analytics',
  },
  {
    id: 'knowledge',
    label: 'Knowledge Graph',
    icon: <KnowledgeIcon />,
    path: '/knowledge',
    description: 'Interactive knowledge exploration',
  },
  {
    id: 'reasoning',
    label: 'Reasoning Engine',
    icon: <ReasoningIcon />,
    path: '/reasoning',
    description: 'AI reasoning paths and queries',
  },
  {
    id: 'bulk-ingester',
    label: 'Bulk Ingestion',
    icon: <BulkIngesterIcon />,
    path: '/bulk-ingester',
    description: 'High-performance data ingestion',
  },
  {
    id: 'grid',
    label: 'Grid Management',
    icon: <GridIcon />,
    path: '/grid',
    description: 'Distributed storage cluster',
  },
  {
    id: 'settings',
    label: 'Settings',
    icon: <SettingsIcon />,
    path: '/settings',
    description: 'System configuration',
  },
];

const quickStatsItems = [
  { icon: <StorageIcon />, label: 'Storage', value: '0', color: 'primary' },
  { icon: <SpeedIcon />, label: 'CPU', value: '0%', color: 'success' },
  { icon: <CloudIcon />, label: 'Memory', value: '0%', color: 'warning' },
  { icon: <TimelineIcon />, label: 'Uptime', value: '0h', color: 'info' },
];

export const Sidebar: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const { systemStatus, connection } = useAppStore();

  const handleNavigation = (path: string) => {
    navigate(path);
  };

  const getQuickStatValue = (label: string) => {
    if (!systemStatus) return '0';
    
    switch (label) {
      case 'Storage':
        return `${systemStatus.metrics.knowledge_items}`;
      case 'CPU':
        return `${systemStatus.metrics.system_load.toFixed(1)}%`;
      case 'Memory':
        return `${systemStatus.metrics.memory_usage.toFixed(1)}%`;
      case 'Uptime':
        return systemStatus.health.uptime;
      default:
        return '0';
    }
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Toolbar>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          <Box
            sx={{
              width: 40,
              height: 40,
              borderRadius: '12px',
              background: 'linear-gradient(135deg, #6366f1 0%, #06b6d4 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '1.2rem',
            }}
          >
            🧠
          </Box>
          <Box>
            <Typography variant="h6" fontWeight={700}>
              Sutra AI
            </Typography>
            <Typography variant="caption" color="text.secondary">
              Control Center
            </Typography>
          </Box>
        </Box>
      </Toolbar>

      <Divider />

      {/* Quick Stats */}
      <Box sx={{ p: 2 }}>
        <Typography variant="subtitle2" color="text.secondary" mb={1}>
          Quick Stats
        </Typography>
        <Stack spacing={1}>
          {quickStatsItems.map((item) => (
            <motion.div
              key={item.label}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
            >
              <Box
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  p: 1,
                  borderRadius: 2,
                  bgcolor: 'rgba(255,255,255,0.03)',
                  border: '1px solid rgba(255,255,255,0.08)',
                }}
              >
                <Box sx={{ color: `${item.color}.main`, mr: 1 }}>
                  {item.icon}
                </Box>
                <Box sx={{ flexGrow: 1 }}>
                  <Typography variant="caption" color="text.secondary">
                    {item.label}
                  </Typography>
                  <Typography variant="body2" fontWeight={600}>
                    {getQuickStatValue(item.label)}
                  </Typography>
                </Box>
              </Box>
            </motion.div>
          ))}
        </Stack>
      </Box>

      <Divider />

      {/* Navigation */}
      <List sx={{ flexGrow: 1, px: 1, py: 2 }}>
        {navigationItems.map((item) => (
          <motion.div
            key={item.id}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <ListItemButton
              selected={location.pathname === item.path}
              onClick={() => handleNavigation(item.path)}
              sx={{
                mb: 0.5,
                borderRadius: 2,
                '&.Mui-selected': {
                  background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(6, 182, 212, 0.1) 100%)',
                  '&:hover': {
                    background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(6, 182, 212, 0.15) 100%)',
                  },
                },
              }}
            >
              <ListItemIcon
                sx={{
                  color: location.pathname === item.path ? 'primary.main' : 'text.secondary',
                }}
              >
                {item.badge ? (
                  <Badge badgeContent={item.badge} color="error">
                    {item.icon}
                  </Badge>
                ) : (
                  item.icon
                )}
              </ListItemIcon>
              <ListItemText
                primary={item.label}
                secondary={item.description}
                primaryTypographyProps={{
                  fontWeight: location.pathname === item.path ? 600 : 400,
                  color: location.pathname === item.path ? 'primary.main' : 'text.primary',
                }}
                secondaryTypographyProps={{
                  variant: 'caption',
                  sx: { fontSize: '0.7rem' },
                }}
              />
            </ListItemButton>
          </motion.div>
        ))}
      </List>

      <Divider />

      {/* Footer */}
      <Box sx={{ p: 2, textAlign: 'center' }}>
        <Chip
          label={connection.connected ? 'Connected' : 'Disconnected'}
          color={connection.connected ? 'success' : 'error'}
          size="small"
          variant="filled"
        />
        <Typography variant="caption" display="block" color="text.secondary" mt={1}>
          v1.0.0 • {new Date().getFullYear()}
        </Typography>
      </Box>
    </Box>
  );
};