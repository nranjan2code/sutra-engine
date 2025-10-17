# Quick Setup Guide

## Installation

```bash
cd packages/sutra-client
npm install
```

## Running Locally

1. **Start the Sutra API** (required):
   ```bash
   cd packages/sutra-api
   python -m sutra_api.main
   ```

2. **Start the web client** (in another terminal):
   ```bash
   cd packages/sutra-client
   npm run dev
   ```

3. Open http://localhost:3000

## What's Different from the Dashboard Version?

### Before (Dashboard)
- Split layout with cards for query/learning
- Separate panel for reasoning visualization
- Busy header with metrics chips
- Form-based interaction

### After (Chat-First)
- **Single conversation thread** (like ChatGPT)
- **Inline reasoning graphs** that expand per message
- **Minimal header** with just health indicator
- **Fixed bottom input** with mode toggle
- **Keyboard shortcuts** (Enter to send, Shift+Enter for newline)

## Architecture

```
HomePage
  ├─ MessageThread (scrollable)
  │    └─ ChatMessage[] (user/AI/system)
  │         └─ InlineReasoningGraph (collapsible)
  └─ ChatInput (fixed bottom)
       ├─ Mode toggle (query/learn)
       └─ Auto-resize text field
```

## State Management

```typescript
// Messages are stored chronologically
interface Message {
  id: string
  type: 'user-query' | 'ai-response' | 'user-learn' | 'system'
  content: string
  reasoning?: ReasoningResult
  metadata?: { factsLearned?: number; error?: string }
}

// Auto-scroll on new messages
// Graphs expand/collapse per message
```

## Development Tips

1. **Testing reasoning**: Use the query mode (default)
2. **Teaching facts**: Toggle to learn mode (📚 icon)
3. **Expanding graphs**: Click the ▼ arrow on AI responses
4. **Keyboard shortcuts**: 
   - Enter: Send
   - Shift+Enter: New line
   - Hover health dot: See concept count

## Customization

### Change Theme Colors
Edit `src/theme.ts`:
```typescript
primary: {
  main: '#6750A4', // Change this
}
```

### Adjust Chat Width
Edit `src/pages/HomePage.tsx`:
```typescript
maxWidth: 1200, // Change this (px)
```

### Modify Input Height
Edit `src/components/ChatInput.tsx`:
```typescript
pb: '140px', // Adjust padding bottom
```

## Building for Production

```bash
npm run build
npm run preview  # Test production build
```

Output will be in `dist/` directory.

## Troubleshooting

### API Connection Issues
- Check that sutra-api is running on http://localhost:8000
- Vite proxy is configured in `vite.config.ts`

### Graphs Not Rendering
- Ensure ReactFlow styles are imported: `import 'reactflow/dist/style.css'`
- Check browser console for errors

### TypeScript Errors
```bash
npm run build  # Will show type errors
```

## Next Steps

- Add dark mode support
- Implement chat history persistence
- Add export/share conversation feature
- Create mobile-optimized view
