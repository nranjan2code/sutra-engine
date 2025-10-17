import { Box } from '@mui/material'
import MessageThread from '../components/MessageThread'
import ChatInput from '../components/ChatInput'

export default function HomePage() {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: 'calc(100vh - 56px)', // Subtract header height
        maxWidth: 1200,
        mx: 'auto',
        pb: '140px', // Space for fixed input
      }}
    >
      <MessageThread />
      <ChatInput />
    </Box>
  )
}
