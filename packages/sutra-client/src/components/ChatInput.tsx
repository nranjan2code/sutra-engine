import { useState, useRef, KeyboardEvent } from 'react'
import {
  Box,
  TextField,
  IconButton,
  Paper,
  ToggleButtonGroup,
  ToggleButton,
  Tooltip,
  CircularProgress,
} from '@mui/material'
import {
  Send as SendIcon,
  Psychology as PsychologyIcon,
  School as SchoolIcon,
} from '@mui/icons-material'
import { sutraApi } from '../services/api'
import { useAppStore } from '../services/store'

type Mode = 'query' | 'learn'

export default function ChatInput() {
  const [input, setInput] = useState('')
  const [mode, setMode] = useState<Mode>('query')
  const { isLoading, setIsLoading, addMessage } = useAppStore()
  const inputRef = useRef<HTMLInputElement>(null)

  const handleSubmit = async () => {
    if (!input.trim() || isLoading) return

    const userContent = input.trim()
    setInput('')

    if (mode === 'query') {
      // Add user query message
      addMessage({
        type: 'user-query',
        content: userContent,
      })

      setIsLoading(true)
      try {
        const result = await sutraApi.reason(userContent)
        addMessage({
          type: 'ai-response',
          content: result.answer,
          reasoning: result,
        })
      } catch (error) {
        addMessage({
          type: 'system',
          content: `Error: ${error instanceof Error ? error.message : 'Failed to process query'}`,
          metadata: { error: 'true' },
        })
      } finally {
        setIsLoading(false)
      }
    } else {
      // Learning mode
      addMessage({
        type: 'user-learn',
        content: userContent,
      })

      setIsLoading(true)
      try {
        const result = await sutraApi.learn(userContent)
        addMessage({
          type: 'system',
          content: `✓ Learned ${result.concepts_created} concept${result.concepts_created !== 1 ? 's' : ''}, ${result.associations_created} association${result.associations_created !== 1 ? 's' : ''}`,
        })
      } catch (error) {
        addMessage({
          type: 'system',
          content: `Error: ${error instanceof Error ? error.message : 'Failed to learn'}`,
          metadata: { error: 'true' },
        })
      } finally {
        setIsLoading(false)
      }
    }
  }

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    // Enter alone to submit (like ChatGPT)
    if (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey) {
      e.preventDefault()
      handleSubmit()
    }
  }

  return (
    <Paper
      elevation={3}
      sx={{
        position: 'fixed',
        bottom: 0,
        left: 0,
        right: 0,
        borderRadius: 0,
        borderTop: 1,
        borderColor: 'divider',
        bgcolor: 'background.paper',
      }}
    >
      <Box
        sx={{
          maxWidth: 800,
          mx: 'auto',
          px: 3,
          py: 2,
        }}
      >
        <Box sx={{ display: 'flex', gap: 2, alignItems: 'flex-end' }}>
          <ToggleButtonGroup
            value={mode}
            exclusive
            onChange={(_, newMode) => {
              if (newMode) setMode(newMode)
            }}
            size="small"
            sx={{ mb: 0.5 }}
          >
            <ToggleButton value="query" sx={{ px: 2 }}>
              <Tooltip title="Ask questions">
                <PsychologyIcon fontSize="small" />
              </Tooltip>
            </ToggleButton>
            <ToggleButton value="learn" sx={{ px: 2 }}>
              <Tooltip title="Teach facts">
                <SchoolIcon fontSize="small" />
              </Tooltip>
            </ToggleButton>
          </ToggleButtonGroup>

          <TextField
            inputRef={inputRef}
            fullWidth
            multiline
            maxRows={4}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={
              mode === 'query'
                ? 'Ask a question...'
                : 'Teach new facts (e.g., "Alice is a software engineer")'
            }
            disabled={isLoading}
            variant="outlined"
            size="small"
            sx={{
              '& .MuiOutlinedInput-root': {
                borderRadius: 3,
                bgcolor: 'background.default',
              },
            }}
          />

          <IconButton
            color="primary"
            onClick={handleSubmit}
            disabled={!input.trim() || isLoading}
            size="large"
            sx={{
              bgcolor: 'primary.main',
              color: 'white',
              '&:hover': {
                bgcolor: 'primary.dark',
              },
              '&.Mui-disabled': {
                bgcolor: 'grey.300',
                color: 'grey.500',
              },
              mb: 0.5,
            }}
          >
            {isLoading ? <CircularProgress size={24} color="inherit" /> : <SendIcon />}
          </IconButton>
        </Box>

        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            mt: 1,
            gap: 2,
          }}
        >
          <Box
            component="span"
            sx={{ fontSize: '0.75rem', color: 'text.secondary', textAlign: 'center' }}
          >
            Press <strong>Enter</strong> to send • <strong>Shift+Enter</strong> for new line
          </Box>
        </Box>
      </Box>
    </Paper>
  )
}
