# Sutra AI Control Center

Modern, secure control center for monitoring and managing the Sutra AI system. Built with React 18, Material Design 3, and a secure FastAPI gateway.

## Architecture

The control center follows a secure gateway pattern:
- **React Frontend**: Modern SPA with real-time updates
- **FastAPI Gateway**: Secure abstraction layer over internal gRPC services
- **WebSocket Updates**: Live system metrics and health status
- **Docker Container**: Production-ready multi-stage build

## 🌟 Features

### 🎛️ System Monitoring
- **Real-time Metrics**: Live system performance data
- **Health Status**: Service availability and connection status  
- **Knowledge Graph Stats**: Concepts, associations, activity scores
- **Performance Charts**: Historical data visualization

### 🔒 Security
- **Secure Gateway**: No direct access to internal services
- **Abstracted APIs**: Only essential metrics exposed
- **Production Hardening**: Non-root containers, restricted CORS
- **No Internal Details**: Service names and internals hidden

### 🎨 Modern Interface
- **Material Design 3**: Modern, accessible design system
- **React 18**: Latest React with Suspense and concurrent features
- **TypeScript**: Full type safety and developer experience
- **Responsive Design**: Works on desktop, tablet, and mobile
- **Progressive Web App**: Installable with offline capabilities

### Navigation & Features
- **Dashboard** - System overview with metrics cards and performance charts
- **Components** - Component status monitoring and control
- **Analytics** - Performance analytics and historical data visualization
- **Knowledge Graph** - Interactive knowledge exploration and visualization
- **Reasoning Engine** - AI reasoning paths and query interface
- **Bulk Ingestion** - High-performance data ingestion management interface ✅
- **Grid Management** - Distributed storage cluster management ✅
- **Settings** - System configuration and preferences

### Dashboard Features
- **System Metrics Cards** - CPU, Memory, Storage, Concepts, Associations
- **Performance Charts** - Real-time CPU and Memory usage visualization
- **Component Status** - Live monitoring of all system components
- **Recent Activity** - Timeline of system events and actions
- **Connection Status** - Visual indicator of WebSocket connectivity

### Design Highlights
- **Dark Theme Optimized** - Easy on the eyes for extended use
- **M3 Color System** - Consistent, accessible color palette
- **Micro-interactions** - Hover effects, transitions, and animations
- **Information Hierarchy** - Clear visual organization of data
- **Mobile-First** - Responsive design that works on all devices

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Start the entire stack
docker compose up -d

# Access the control center
open http://localhost:9000
```

### Development Mode

```bash
cd packages/sutra-control

# Install dependencies
npm install

# Start development server (frontend)
npm run dev

# In separate terminal, start backend
python backend/main.py
```

### Backend Integration

The secure FastAPI gateway abstracts all internal gRPC communication:
- **WebSocket**: `ws://localhost:9000/ws` - Real-time system updates
- **REST API**: `http://localhost:9000/api/*` - Abstract system metrics
- **Health Check**: `http://localhost:9000/health` - System status

## 🏗️ Architecture

### Technology Stack
- **React 18** - Modern React with concurrent features
- **TypeScript** - Type-safe development
- **Material-UI v5** - Material Design 3 components
- **Zustand** - Lightweight state management
- **Recharts** - Responsive chart library
- **Framer Motion** - Animation library
- **Vite** - Fast build tool and dev server

### Project Structure
```
src/
├── components/           # Reusable UI components
│   ├── Dashboard/       # Main dashboard components
│   ├── Layout/          # App layout and navigation
│   ├── ConnectionStatus/# WebSocket status indicator
│   └── [Other views]/   # Future feature components
├── store/               # Zustand state management
├── types/               # TypeScript type definitions
├── theme/               # Material-UI theme configuration
├── App.tsx              # Main application component
└── main.tsx             # Application entry point
```

### State Management
Using Zustand for clean, simple state management:
- **Connection State** - WebSocket status and error handling
- **System Data** - Real-time metrics and component status
- **Chart Data** - Performance history (last 50 points)
- **UI State** - Sidebar, current view, user preferences

## 🎨 Design System

### Material Design 3 Implementation
- **Color System** - Primary: Indigo, Secondary: Cyan, Surface variants
- **Typography** - Clear hierarchy with proper contrast ratios
- **Elevation** - Subtle shadows and depth
- **Motion** - Smooth transitions and micro-interactions

### Components
- **MetricCard** - Displays key system metrics with progress bars
- **PerformanceChart** - Real-time data visualization
- **SystemOverview** - Component status monitoring
- **Navigation** - Clean sidebar with quick stats

### Responsive Breakpoints
- **Mobile**: < 768px - Stack layout, collapsible sidebar
- **Tablet**: 768px - 1024px - Adaptive grid system
- **Desktop**: > 1024px - Full layout with persistent sidebar

## 📊 Data Flow

### WebSocket Integration
```typescript
// Real-time updates every 2 seconds
const data = {
  components: { /* component status */ },
  metrics: { /* system metrics */ }
};

// Automatically updates charts and metrics
useAppStore.getState().setSystemStatus(data);
```

### Chart Data Management
- Maintains rolling window of last 50 data points
- Automatically scales and formats for display
- Smooth animations for new data points

## 🔧 Configuration

### Environment Variables
```bash
# Development
VITE_API_URL=http://localhost:5000
VITE_WS_URL=ws://localhost:5000

# Production
VITE_API_URL=https://your-sutra-api.com
VITE_WS_URL=wss://your-sutra-api.com
```

### Proxy Configuration
Development server proxies to backend:
```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': 'http://localhost:5000',
    '/ws': { target: 'ws://localhost:5000', ws: true }
  }
}
```

## 🚧 Future Roadmap

### Phase 1: Enhanced Visualizations
- Interactive knowledge graph using D3.js/Cytoscape
- Advanced performance analytics dashboard
- Historical data persistence and trends

### Phase 2: AI-Specific Features
- **Reasoning Path Explorer** - Visual representation of AI reasoning
- **Query Interface** - Interactive query testing and debugging
- **Knowledge Management** - Add/edit concepts and associations

### Phase 3: Advanced Features
- **Multi-user Support** - Authentication and role-based access
- **Alerts & Notifications** - Configurable system alerts
- **Export/Import** - Data backup and configuration management
- **Plugin System** - Extensible architecture for custom widgets

### Phase 4: Enterprise Features
- **Multi-instance Management** - Monitor multiple Sutra AI deployments
- **Advanced Security** - SSO, audit logging, encryption
- **API Management** - Rate limiting, usage analytics
- **Custom Dashboards** - User-configurable layouts

## 🎯 Performance Optimization

### Bundle Optimization
- Code splitting by route and feature
- Tree shaking for unused dependencies
- Optimized Material-UI imports

### Runtime Performance
- React.memo for expensive components
- Virtualization for large lists
- Debounced real-time updates

### PWA Features
- Service worker for offline functionality
- App installation prompt
- Background sync for data

## 🧪 Testing Strategy

### Planned Testing Implementation
- **Unit Tests** - Jest + React Testing Library
- **Integration Tests** - Component interaction testing  
- **E2E Tests** - Playwright for full user workflows
- **Visual Regression** - Automated screenshot comparison

## 🔒 Security Considerations

### Current Security Measures
- Input sanitization for all user data
- Secure WebSocket connections (WSS in production)
- Content Security Policy headers

### Production Security Checklist
- [ ] Enable HTTPS/WSS
- [ ] Implement authentication
- [ ] Add rate limiting
- [ ] Enable audit logging
- [ ] Configure CSP headers

## 📱 Mobile Experience

### Mobile-Optimized Features
- Touch-friendly interface elements
- Swipe gestures for navigation
- Optimized chart interactions
- Collapsible sidebar for more screen space

### Performance on Mobile
- Reduced animation complexity on low-end devices
- Optimized bundle size for faster loading
- Efficient memory usage for long sessions

## 🛠️ Development

### Available Scripts
```bash
# Development
npm run dev          # Start dev server
npm run build        # Production build
npm run preview      # Preview production build
npm run lint         # TypeScript type checking

# Future additions
npm run test         # Run test suite
npm run test:e2e     # End-to-end tests
npm run storybook    # Component documentation
```

### Code Quality
- ESLint + Prettier configuration
- Husky pre-commit hooks
- Conventional commit messages
- TypeScript strict mode

## 💡 Contributing

### Development Setup
1. Fork and clone the repository
2. Install dependencies: `npm install`
3. Start the development server: `npm run dev`
4. Make changes and test locally
5. Submit a pull request with a clear description

### Code Style Guidelines
- Use TypeScript for all new code
- Follow Material Design 3 patterns
- Maintain responsive design principles
- Add proper error handling and loading states

## 📄 License

Part of the Sutra AI project. See the main project license for details.

## 🤝 Support

For questions, issues, or contributions:
1. Check the existing GitHub issues
2. Create a new issue with a clear description
3. Include steps to reproduce any bugs
4. Provide system information and screenshots when helpful

---

**Built with ❤️ for the Sutra AI project**

This modern React control center represents the future of AI system management - combining professional design, powerful functionality, and delightful user experience.