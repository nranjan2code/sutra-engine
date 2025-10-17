# Sutra Client

Chat-first web interface for Sutra AI with Material Design 3. Like ChatGPT/Claude, but with explainable graph-based reasoning.

## Features

- **💬 Chat-First UX**: Conversational interface just like ChatGPT/Claude
- **🎨 Material Design 3**: Clean, modern UI with Google's latest design system
- **🧠 Inline Reasoning**: Expandable graph visualizations within chat messages
- **📚 Live Learning**: Teach facts naturally through conversation
- **⌨️ Keyboard Shortcuts**: Enter to send, ⌘Enter for new line
- **🔍 100% Transparent**: Every answer shows complete reasoning paths

## Tech Stack

- **React 18** with TypeScript for type-safe development
- **Vite** for lightning-fast builds and HMR
- **Material-UI v6** for Material Design 3 components
- **ReactFlow** for interactive graph visualization
- **Zustand** for lightweight state management
- **Axios** for API communication

## Quick Start

### Prerequisites

- Node.js 18+ and npm/yarn
- Sutra API server running on `http://localhost:8000`

### Installation & Run

```bash
cd packages/sutra-client
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

> See [SETUP.md](SETUP.md) for detailed setup and migration guide.

### Build for Production

```bash
npm run build
npm run preview
```

## Project Structure

```
src/
├── components/
│   ├── Layout.tsx                # Minimal header with health indicator
│   ├── MessageThread.tsx         # Scrollable conversation thread
│   ├── ChatMessage.tsx           # Message bubbles (user/AI/system)
│   ├── ChatInput.tsx             # Fixed bottom input with mode toggle
│   └── InlineReasoningGraph.tsx  # Collapsible graph in messages
├── pages/
│   └── HomePage.tsx              # Chat interface
├── services/
│   ├── api.ts                    # API service layer
│   └── store.ts                  # Conversation state (Zustand)
├── types/
│   └── api.ts                    # API types
├── theme.ts                      # Material Design 3 theme
├── App.tsx                       # Root component
├── main.tsx                      # Entry point
└── index.css                     # Global styles
```

## Usage

### Conversation Mode

The interface works exactly like ChatGPT or Claude:

1. **Ask Questions** (default mode):
   - Type your question in the input box
   - Press **Enter** to send (or **⌘Enter** for multiline)
   - AI responds with answer + expandable reasoning graph

2. **Teach Facts** (toggle to learning mode):
   - Click the 📚 icon to switch modes
   - Enter facts in natural language
   - Get confirmation of what was learned

### Understanding Reasoning Paths

- Click the **▼ arrow** on any AI response to expand the reasoning graph
- **Blue nodes**: Concepts in the reasoning chain
- **Green nodes**: Final answer nodes
- **Animated edges**: Show associations with confidence scores
- **Multiple paths**: AI found several routes to the same answer

### Keyboard Shortcuts

- **Enter**: Send message
- **Shift+Enter**: New line
- **⌘Enter**: Alternative send

## API Integration

The client connects to the Sutra API via a proxy configured in `vite.config.ts`:

- `/api/reason` - Query the reasoning engine
- `/api/learn` - Teach new facts
- `/api/metrics` - Get system metrics
- `/api/health` - Check system health

## Environment Variables

Create a `.env.local` file:

```env
VITE_API_URL=http://localhost:8000
```

## Design Philosophy

### Chat-First Design

Inspired by ChatGPT and Claude's clutter-free approach:

- **Conversation thread**: Natural chat flow, not a dashboard
- **Minimal header**: Just logo + health dot (hover for stats)
- **Fixed input**: Always accessible at the bottom
- **Inline graphs**: Reasoning appears contextually, not in separate panels
- **Generous whitespace**: Easy on the eyes, mobile-friendly

### Material Design 3

- **Dynamic color**: Purple theme (#6750A4) for cognitive/neural aesthetic
- **Rounded bubbles**: Chat messages with tail indicators
- **Subtle elevation**: Depth without heaviness
- **Typography**: Roboto with balanced weights

### Explainability First

- **Visual reasoning**: Graph shows complete reasoning paths
- **Confidence scores**: Display certainty at every step
- **Multiple paths**: Show consensus and diversity in reasoning
- **Real-time updates**: See the AI learn and adapt

## Development Commands

```bash
# Development server
npm run dev

# Type checking
npm run build

# Linting
npm run lint

# Format code
npm run format
```

## Performance

- **Instant HMR**: Vite provides sub-second hot module replacement
- **Optimized builds**: Tree-shaking and code-splitting enabled
- **Lazy loading**: Routes and heavy components loaded on demand
- **Memoization**: React hooks optimize re-renders

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Contributing

When adding new features:

1. Follow Material Design 3 guidelines
2. Maintain TypeScript strict mode
3. Add proper error handling
4. Update this README if adding new sections

## License

Part of the Sutra AI project.
