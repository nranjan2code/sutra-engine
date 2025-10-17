import { useState, useEffect } from 'react'
import { AppBar, Box, Toolbar, Typography, Tooltip } from '@mui/material'
import { Psychology as PsychologyIcon, FiberManualRecord } from '@mui/icons-material'
import { sutraApi } from '../services/api'
import { useAppStore } from '../services/store'

interface LayoutProps {
  children: React.ReactNode
}

export default function Layout({ children }: LayoutProps) {
  const [health, setHealth] = useState<{ status: string; concepts: number }>({
    status: 'unknown',
    concepts: 0,
  })
  const setMetrics = useAppStore((state) => state.setMetrics)

  const fetchMetrics = async () => {
    try {
      const [healthData, metricsData] = await Promise.all([
        sutraApi.getHealth(),
        sutraApi.getMetrics(),
      ])
      setHealth({
        status: healthData.status,
        concepts: healthData.concepts_loaded || 0,
      })
      setMetrics(metricsData)
    } catch (error) {
      console.error('Failed to fetch metrics:', error)
      setHealth((prev) => ({ ...prev, status: 'error' }))
    }
  }

  useEffect(() => {
    fetchMetrics()
    const interval = setInterval(fetchMetrics, 10000)
    return () => clearInterval(interval)
  }, [])

  const healthColor =
    health.status === 'healthy' ? 'success.main' : health.status === 'error' ? 'error.main' : 'warning.main'

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <AppBar
        position="static"
        elevation={0}
        sx={{
          bgcolor: 'background.paper',
          borderBottom: 1,
          borderColor: 'divider',
        }}
      >
        <Toolbar variant="dense" sx={{ minHeight: 56 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
            <PsychologyIcon sx={{ fontSize: 28, color: 'primary.main' }} />
            <Typography
              variant="h6"
              component="div"
              sx={{ fontWeight: 600, color: 'text.primary' }}
            >
              Sutra AI
            </Typography>
          </Box>
          <Box sx={{ flexGrow: 1 }} />
          <Tooltip
            title={`${health.concepts.toLocaleString()} concepts • ${health.status}`}
            arrow
          >
            <FiberManualRecord
              sx={{
                fontSize: 12,
                color: healthColor,
                animation: health.status === 'healthy' ? 'none' : 'pulse 2s infinite',
                '@keyframes pulse': {
                  '0%, 100%': { opacity: 1 },
                  '50%': { opacity: 0.4 },
                },
              }}
            />
          </Tooltip>
        </Toolbar>
      </AppBar>

      {children}
    </Box>
  )
}
