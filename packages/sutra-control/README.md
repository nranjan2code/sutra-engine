# 🧠 Sutra Control Center

Beautiful web-based control center for managing the Sutra AI system lifecycle.

## Features

- **🎯 Real-time Monitoring**: Live updates via WebSocket with 2-second refresh rate
- **⚡ Component Control**: Start/stop/restart Storage Engine and API Server
- **📊 Performance Metrics**: CPU, memory, storage usage with live charts
- **🎨 Modern UI**: Dark theme with gradient colors and smooth animations
- **📈 Historical Charts**: Track resource usage and storage growth over time

## Installation

```bash
cd packages/sutra-control
pip install -e .
```

## Quick Start

```bash
# Start the control center (default port 9000)
python -m sutra_control.main

# Or with uvicorn directly
cd packages/sutra-control
uvicorn sutra_control.main:app --host 0.0.0.0 --port 9000
```

Then open your browser to **http://localhost:9000**

## Architecture

```
sutra-control/
├── sutra_control/
│   ├── __init__.py
│   └── main.py          # FastAPI backend + lifecycle manager
├── static/
│   ├── css/
│   │   └── dashboard.css  # Modern dark theme styling
│   └── js/
│       └── dashboard.js   # Real-time WebSocket client
├── templates/
│   └── dashboard.html     # Main dashboard
└── setup.py
```

## How It Works

### Backend (Python/FastAPI)

- **LifecycleManager**: Manages component processes (API server, storage engine)
- **WebSocket**: Broadcasts real-time updates every 2 seconds
- **REST API**: Endpoints for start/stop/restart operations
- **Process Monitoring**: Uses `psutil` for CPU, memory, and uptime tracking

### Frontend (HTML/CSS/JS)

- **Real-time Updates**: WebSocket connection with auto-reconnect
- **Live Charts**: Chart.js for CPU/memory and storage growth visualization
- **Responsive Design**: Mobile-friendly grid layout
- **Dark Theme**: Modern glassmorphism with gradient accents

## API Endpoints

### WebSocket
- `ws://localhost:9000/ws` - Real-time status updates

### REST API
- `GET /api/status` - Get current system status
- `POST /api/components/{component}/start` - Start component
- `POST /api/components/{component}/stop` - Stop component
- `POST /api/components/{component}/restart` - Restart component

Components: `storage`, `api`

## Dashboard Features

### System Metrics
- **Storage Size**: Total disk space used by knowledge base
- **Concepts**: Total learned concepts
- **Associations**: Total concept relationships
- **CPU Usage**: System-wide CPU utilization
- **Memory Usage**: System-wide memory utilization

### Component Cards
Each component shows:
- Current state (running/stopped/starting/stopping/error)
- Uptime (formatted as seconds/minutes/hours/days)
- CPU usage (per-process)
- Memory usage (per-process)
- Process ID (PID)
- Control buttons (Start/Stop/Restart)

### Performance Charts
- **CPU & Memory**: Dual-axis line chart tracking resource usage
- **Storage Growth**: Line chart showing knowledge base expansion
- Last 20 data points displayed (40 seconds of history)

## Configuration

Control center reads from environment variables:

```bash
# Sutra API location
export SUTRA_STORAGE_PATH="./knowledge"

# Control center port (default: 9000)
# If using a different port, update the uvicorn command
```

## Development

```bash
# Install with dev dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/

# Format code
black sutra_control/
isort sutra_control/
```

## Screenshots

### Dashboard Overview
Beautiful dark-themed interface with real-time metrics and component status cards.

### Component Control
Start, stop, and restart components with a single click. Live status updates show transitions.

### Performance Charts
Track CPU, memory, and storage growth over time with smooth Chart.js visualizations.

## Integration with Sutra AI

The control center integrates with existing Sutra packages:

- **sutra-storage**: Manages Rust storage engine lifecycle
- **sutra-api**: Controls FastAPI server process
- **sutra-core**: Monitors concept and association counts
- **sutra-hybrid**: Tracks semantic embeddings if enabled

## Troubleshooting

### WebSocket Connection Fails
- Ensure control center is running on the correct port
- Check firewall settings
- Verify no port conflicts

### Component Won't Start
- Check that `SUTRA_STORAGE_PATH` is set correctly
- Ensure all dependencies are installed
- Review logs in control center console

### Charts Not Updating
- Refresh browser page
- Check browser console for JavaScript errors
- Verify WebSocket connection is active (green dot in header)

## Future Enhancements

- [ ] Historical metrics persistence (database)
- [ ] Alert/notification system for errors
- [ ] Multi-node cluster management
- [ ] Custom dashboards and widgets
- [ ] Metrics export (Prometheus/Grafana)
- [ ] Authentication and RBAC

## License

Part of the Sutra AI project.
