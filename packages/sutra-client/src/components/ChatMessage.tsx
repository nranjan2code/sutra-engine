import { Box, Typography, Paper, Chip, alpha } from '@mui/material'
import {
  Person as PersonIcon,
  Psychology as PsychologyIcon,
  School as SchoolIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
} from '@mui/icons-material'
import { Message } from '../services/store'
import { useAppStore } from '../services/store'
import InlineReasoningGraph from './InlineReasoningGraph'

interface Props {
  message: Message
}

export default function ChatMessage({ message }: Props) {
  const { expandedGraphs, toggleGraph } = useAppStore()
  const isExpanded = expandedGraphs.has(message.id)

  // User query message
  if (message.type === 'user-query') {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 3 }}>
        <Box sx={{ maxWidth: '70%', display: 'flex', alignItems: 'flex-start', gap: 1.5 }}>
          <Paper
            elevation={0}
            sx={{
              px: 3,
              py: 1.5,
              bgcolor: 'primary.main',
              color: 'white',
              borderRadius: 3,
              borderBottomRightRadius: 0.5,
            }}
          >
            <Typography variant="body1">{message.content}</Typography>
          </Paper>
          <Box
            sx={{
              width: 32,
              height: 32,
              borderRadius: '50%',
              bgcolor: 'primary.light',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            <PersonIcon sx={{ fontSize: 18, color: 'white' }} />
          </Box>
        </Box>
      </Box>
    )
  }

  // AI response message
  if (message.type === 'ai-response') {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'flex-start', mb: 3 }}>
        <Box sx={{ maxWidth: '85%', display: 'flex', alignItems: 'flex-start', gap: 1.5 }}>
          <Box
            sx={{
              width: 32,
              height: 32,
              borderRadius: '50%',
              bgcolor: (theme) => alpha(theme.palette.primary.main, 0.1),
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            <PsychologyIcon sx={{ fontSize: 18, color: 'primary.main' }} />
          </Box>
          <Box sx={{ flex: 1 }}>
            <Paper
              elevation={0}
              sx={{
                px: 3,
                py: 1.5,
                bgcolor: 'grey.50',
                borderRadius: 3,
                borderBottomLeftRadius: 0.5,
              }}
            >
              <Typography variant="body1" sx={{ mb: message.reasoning ? 1 : 0 }}>
                {message.content}
              </Typography>
            </Paper>
            {message.reasoning && (
              <InlineReasoningGraph
                reasoning={message.reasoning}
                expanded={isExpanded}
                onToggle={() => toggleGraph(message.id)}
              />
            )}
          </Box>
        </Box>
      </Box>
    )
  }

  // Learning confirmation message
  if (message.type === 'user-learn') {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 3 }}>
        <Box sx={{ maxWidth: '70%', display: 'flex', alignItems: 'flex-start', gap: 1.5 }}>
          <Paper
            elevation={0}
            sx={{
              px: 3,
              py: 1.5,
              bgcolor: 'secondary.main',
              color: 'white',
              borderRadius: 3,
              borderBottomRightRadius: 0.5,
            }}
          >
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
              <SchoolIcon sx={{ fontSize: 16 }} />
              <Typography variant="body2" sx={{ fontWeight: 500 }}>
                Teaching
              </Typography>
            </Box>
            <Typography variant="body1">{message.content}</Typography>
          </Paper>
          <Box
            sx={{
              width: 32,
              height: 32,
              borderRadius: '50%',
              bgcolor: 'secondary.light',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            <PersonIcon sx={{ fontSize: 18, color: 'white' }} />
          </Box>
        </Box>
      </Box>
    )
  }

  // System message (confirmations, errors)
  if (message.type === 'system') {
    const isError = !!message.metadata?.error
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', mb: 2 }}>
        <Chip
          icon={isError ? <ErrorIcon /> : <CheckCircleIcon />}
          label={message.content}
          color={isError ? 'error' : 'success'}
          variant="outlined"
          size="small"
        />
      </Box>
    )
  }

  return null
}
