# Quick Start Guide

Get the Sutra Control Center running in under 5 minutes.

## Prerequisites

- Python 3.8 or higher
- pip (Python package manager)
- Modern web browser
- Terminal/command line access

## Installation

### Option 1: From Project Root (Recommended)

```bash
# Navigate to project root
cd /path/to/sutra-models

# Ensure virtual environment is set up
make setup  # Or: python3 -m venv venv && source venv/bin/activate

# Install control center
pip install -e packages/sutra-control/
```

### Option 2: Standalone Installation

```bash
cd packages/sutra-control

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# Install package
pip install -e .
```

## Quick Launch

### Method 1: Launch Script (Easiest)

```bash
cd packages/sutra-control
./launch.sh
```

### Method 2: Python Module

```bash
python -m sutra_control.main
```

### Method 3: Uvicorn Directly

```bash
cd packages/sutra-control
uvicorn sutra_control.main:app --host 0.0.0.0 --port 9000 --reload
```

## Access Dashboard

Open your browser to:

```
http://localhost:9000
```

You should see the Control Center dashboard with:
- System metrics cards (CPU, memory, storage)
- Component status cards (Storage, API)
- Performance charts

## First Steps

### 1. Check System Status

The dashboard shows real-time system metrics. You should see:
- Green connection indicator (top right)
- Current CPU and memory usage
- Storage size (may be 0 if no knowledge loaded)

### 2. Start Components

Click the **"Start"** button on the API component card to launch the Sutra API server.

Wait a few seconds for the status to change to **"Running"** with a green badge.

### 3. Monitor Performance

Watch the charts at the bottom update in real-time:
- **CPU & Memory**: System resource usage
- **Storage Growth**: Knowledge base size over time

### 4. Control Components

Use the buttons to manage components:
- **Start**: Launch a stopped component
- **Stop**: Gracefully shut down a running component  
- **Restart**: Stop and start in one action

## Configuration

### Environment Variables

Create a `.env` file or export variables:

```bash
# Storage location
export SUTRA_STORAGE_PATH="./knowledge"

# Control Center port (default: 9000)
# Modify in launch command if needed

# Sutra API port (managed component)
export SUTRA_API_PORT="8000"
```

### Custom Port

```bash
# Run on custom port
uvicorn sutra_control.main:app --host 0.0.0.0 --port 8080
```

## Verification

### Check Installation

```bash
python -c "import sutra_control; print(sutra_control.__version__)"
# Should print: 0.1.0
```

### Test REST API

```bash
# Check status endpoint
curl http://localhost:9000/api/status | jq

# Should return JSON with components and metrics
```

### Test WebSocket

Open browser console and run:

```javascript
ws = new WebSocket('ws://localhost:9000/ws');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

You should see status updates every 2 seconds.

## Troubleshooting

### Port Already in Use

```
Error: [Errno 48] Address already in use
```

**Solution**: Use a different port or stop the conflicting process

```bash
# Find process using port 9000
lsof -i :9000

# Kill if needed
kill -9 <PID>

# Or use different port
uvicorn sutra_control.main:app --port 9001
```

### Module Not Found

```
ModuleNotFoundError: No module named 'sutra_control'
```

**Solution**: Install the package

```bash
pip install -e packages/sutra-control/
```

### WebSocket Connection Failed

**Solution**: 
1. Ensure Control Center is running
2. Check firewall settings
3. Verify port 9000 is accessible
4. Try accessing from localhost explicitly

### Components Won't Start

**Solution**:
1. Check that sutra-api package is installed
2. Verify SUTRA_STORAGE_PATH exists
3. Look for errors in terminal output
4. Check component logs (if available)

## Next Steps

- [API Reference](../api/rest-endpoints.md) - Control via REST API
- [Architecture](../architecture/overview.md) - Understand the system
- [Development Guide](../development/setup.md) - Contribute or customize
- [Production Deployment](production.md) - Deploy for production use

## Stopping the Control Center

Press `Ctrl+C` in the terminal running the Control Center.

All managed components will continue running. Stop them via:
1. Dashboard UI (before shutting down)
2. REST API (`curl -X POST http://localhost:9000/api/components/api/stop`)
3. Manually (kill the process)

## Uninstallation

```bash
pip uninstall sutra-control
```

## Quick Reference

| Action | Command |
|--------|---------|
| Install | `pip install -e packages/sutra-control/` |
| Start | `./packages/sutra-control/launch.sh` |
| Access | `http://localhost:9000` |
| API Docs | `http://localhost:9000/docs` |
| Stop | `Ctrl+C` |
| Uninstall | `pip uninstall sutra-control` |
