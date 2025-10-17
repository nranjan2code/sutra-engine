import { useEffect, useRef } from 'react'
import { Box, Typography, Button } from '@mui/material'
import { Psychology as PsychologyIcon } from '@mui/icons-material'
import { useAppStore } from '../services/store'
import ChatMessage from './ChatMessage'

export default function MessageThread() {
  const messages = useAppStore((state) => state.messages)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // Empty state
  if (messages.length === 0) {
    return (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          px: 3,
        }}
      >
        <Box
          sx={{
            width: 64,
            height: 64,
            borderRadius: '50%',
            bgcolor: 'primary.main',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            mb: 3,
          }}
        >
          <PsychologyIcon sx={{ fontSize: 32, color: 'white' }} />
        </Box>
        <Typography variant="h5" gutterBottom sx={{ fontWeight: 600 }}>
          Welcome to Sutra AI
        </Typography>
        <Typography
          variant="body1"
          color="text.secondary"
          sx={{ textAlign: 'center', maxWidth: 500, mb: 4 }}
        >
          Explainable graph-based reasoning. Ask questions, teach facts, and see complete
          reasoning paths for every answer.
        </Typography>
        <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', justifyContent: 'center' }}>
          <Typography variant="body2" color="text.secondary">
            Try asking:
          </Typography>
          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
            {[
              'Who is Alice?',
              'What does Bob like?',
              'Tell me about Python',
            ].map((example) => (
              <Button
                key={example}
                variant="outlined"
                size="small"
                sx={{ borderRadius: 3, textTransform: 'none' }}
              >
                {example}
              </Button>
            ))}
          </Box>
        </Box>
      </Box>
    )
  }

  // Message list
  return (
    <Box
      sx={{
        flex: 1,
        overflowY: 'auto',
        px: 2,
        py: 3,
        '&::-webkit-scrollbar': {
          width: '8px',
        },
        '&::-webkit-scrollbar-track': {
          bgcolor: 'transparent',
        },
        '&::-webkit-scrollbar-thumb': {
          bgcolor: 'divider',
          borderRadius: '4px',
          '&:hover': {
            bgcolor: 'grey.400',
          },
        },
      }}
    >
      {messages.map((message) => (
        <ChatMessage key={message.id} message={message} />
      ))}
      <div ref={messagesEndRef} />
    </Box>
  )
}
