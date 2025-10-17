# Sutra Control Center Documentation

**Version**: 0.1.0  
**Status**: Alpha  
**Last Updated**: 2025-10-17

## Overview

Sutra Control Center is a web-based lifecycle management and monitoring system for the Sutra AI platform. It provides real-time visualization, component orchestration, and operational control through a modern, responsive dashboard.

## Purpose

The Control Center serves as the **operational hub** for managing Sutra AI deployments, enabling:

- **Lifecycle Management**: Start, stop, and restart system components
- **Real-time Monitoring**: Live metrics and performance tracking
- **Health Visibility**: Component status and system health at a glance
- **Operational Control**: Centralized management interface

## Key Features

### ✅ Implemented (v0.1.0)

- **Real-time WebSocket Updates**: 2-second refresh rate for live monitoring
- **Component Lifecycle Control**: Manage Storage Engine and API Server
- **System Metrics Dashboard**: CPU, memory, storage, concepts, associations
- **Performance Charts**: Resource usage and storage growth visualization
- **Modern Dark Theme UI**: Responsive design with gradient accents
- **Process Monitoring**: PID tracking, uptime, per-process resource usage
- **REST API**: Programmatic control endpoints
- **Auto-reconnect**: Resilient WebSocket with automatic reconnection

### 🚧 Planned (Future Versions)

#### v0.2.0 - Enhanced Observability
- Live streaming logs viewer with filtering/search
- Historical data persistence (SQLite backend)
- Alert/notification system with thresholds
- Metrics export (Prometheus/Grafana compatible)

#### v0.3.0 - AI Operations
- Interactive query testing interface
- Knowledge graph visualization (D3.js/Cytoscape)
- Reasoning path explorer
- Batch learning interface

#### v0.4.0 - Production Readiness
- Authentication and RBAC
- Multi-node cluster management
- Backup/restore functionality
- Advanced alerting rules
- Integration with Rust storage metrics

## Documentation Structure

```
docs/packages/sutra-control/
├── README.md                    # This file
├── architecture/
│   ├── overview.md             # System architecture
│   ├── components.md           # Component breakdown
│   └── data-flow.md            # Data flow diagrams
├── api/
│   ├── rest-endpoints.md       # REST API reference
│   ├── websocket-protocol.md  # WebSocket protocol
│   └── models.md               # Data models
├── deployment/
│   ├── quickstart.md           # Quick start guide
│   ├── production.md           # Production deployment
│   └── docker.md               # Docker setup
├── development/
│   ├── setup.md                # Development setup
│   ├── contributing.md         # Contribution guide
│   └── testing.md              # Testing strategy
└── ui-ux/
    ├── design-system.md        # Design system
    ├── components.md           # UI components
    └── interactions.md         # User interactions
```

## Quick Links

- [Architecture Overview](architecture/overview.md)
- [API Reference](api/rest-endpoints.md)
- [Quick Start Guide](deployment/quickstart.md)
- [Development Setup](development/setup.md)
- [UI/UX Design](ui-ux/design-system.md)

## Technology Stack

### Backend
- **Framework**: FastAPI 0.104+
- **WebSocket**: Native FastAPI WebSocket support
- **Process Management**: Python subprocess + psutil
- **ASGI Server**: Uvicorn with async support

### Frontend
- **Core**: Vanilla JavaScript (ES6+)
- **Charts**: Chart.js 4.4.0
- **Styling**: Modern CSS with CSS variables
- **Architecture**: Event-driven with WebSocket

### Infrastructure
- **Process Orchestration**: Python subprocess management
- **Resource Monitoring**: psutil for system metrics
- **File System**: Path-based component discovery

## System Requirements

### Runtime
- Python 3.8+
- 512 MB RAM minimum (1 GB recommended)
- Modern web browser (Chrome 90+, Firefox 88+, Safari 14+)

### Network
- Port 9000 (Control Center, configurable)
- Port 8000 (Sutra API, if managing via Control Center)
- WebSocket support required

## Architecture Highlights

### Backend Architecture
```
┌─────────────────────────────────────────┐
│         FastAPI Application             │
├─────────────────────────────────────────┤
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ REST API     │  │ WebSocket       │ │
│  │ - /api/*     │  │ - /ws           │ │
│  └──────────────┘  └─────────────────┘ │
├─────────────────────────────────────────┤
│       LifecycleManager                  │
│  - Component orchestration              │
│  - Process management                   │
│  - Status tracking                      │
├─────────────────────────────────────────┤
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ subprocess   │  │ psutil          │ │
│  │ (Popen)      │  │ (monitoring)    │ │
│  └──────────────┘  └─────────────────┘ │
└─────────────────────────────────────────┘
```

### Frontend Architecture
```
┌─────────────────────────────────────────┐
│         Dashboard (SPA)                 │
├─────────────────────────────────────────┤
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ WebSocket    │  │ Chart.js        │ │
│  │ Client       │  │ Visualizations  │ │
│  └──────────────┘  └─────────────────┘ │
├─────────────────────────────────────────┤
│  Dashboard Controller                   │
│  - updateMetrics()                      │
│  - updateComponents()                   │
│  - updateCharts()                       │
├─────────────────────────────────────────┤
│  ┌──────────────┐  ┌─────────────────┐ │
│  │ DOM          │  │ Event           │ │
│  │ Manipulation │  │ Handlers        │ │
│  └──────────────┘  └─────────────────┘ │
└─────────────────────────────────────────┘
```

## Current Limitations

### v0.1.0 Known Limitations

1. **No Persistence**: Metrics history lost on refresh
2. **Limited History**: Only 40 seconds (20 data points)
3. **No Logs**: Cannot view component logs
4. **Basic Components**: Only Storage and API management
5. **No Alerts**: No threshold-based notifications
6. **No Authentication**: Open access (development only)
7. **Single Instance**: No multi-node support
8. **Limited Integration**: Missing Rust storage-level metrics

## Performance Characteristics

### Backend Performance
- **WebSocket Updates**: 2-second interval (configurable)
- **Memory Footprint**: ~50-100 MB (FastAPI + monitoring)
- **CPU Usage**: <5% idle, <10% under load
- **Startup Time**: <2 seconds

### Frontend Performance
- **Initial Load**: <500ms (excluding CDN)
- **Chart Updates**: 60 FPS smooth rendering
- **Memory Usage**: ~50-80 MB (browser)
- **WebSocket Latency**: <10ms local, <100ms network

## Security Considerations

### Current Status (v0.1.0)
⚠️ **Development Only - Not Production Ready**

- No authentication/authorization
- No TLS/SSL encryption
- No rate limiting on control endpoints
- No audit logging
- Runs on 0.0.0.0 (all interfaces)

### Planned Security (v0.4.0+)
- JWT-based authentication
- Role-based access control (RBAC)
- TLS encryption for WebSocket and REST
- API rate limiting
- Comprehensive audit logging
- IP whitelisting support

## Use Cases

### Development
- Monitor local Sutra AI instance
- Quick component restarts during development
- Visual feedback on system performance
- Debug resource usage issues

### Testing
- Observe system behavior under load
- Track memory/CPU during test runs
- Verify component lifecycle management
- Integration testing support

### Production (Future)
- Centralized operational dashboard
- Health monitoring and alerting
- Incident response interface
- Performance analysis

## Integration Points

### With Sutra AI Components

```
┌──────────────────┐
│  Control Center  │
└────────┬─────────┘
         │
    ┌────┴────────────────────┐
    │                         │
    ▼                         ▼
┌─────────┐           ┌──────────────┐
│ Storage │           │  Sutra API   │
│ Engine  │◄──────────┤  (FastAPI)   │
└─────────┘           └──────────────┘
    │                         │
    │                         ▼
    │                 ┌───────────────┐
    │                 │  sutra-hybrid │
    │                 │  (SutraAI)    │
    │                 └───────────────┘
    │                         │
    └─────────────────────────┘
```

## Getting Started

See [Quick Start Guide](deployment/quickstart.md) for installation and setup.

## Contributing

See [Contributing Guide](development/contributing.md) for development guidelines.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed feature roadmap and timeline.

## License

Part of the Sutra AI project.
