# Architecture Overview

## System Architecture

Sutra Control Center follows a **client-server architecture** with real-time bidirectional communication via WebSocket.

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        Browser (Client)                      │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                   Dashboard UI                         │  │
│  │  - HTML5 + CSS3 + Vanilla JS                          │  │
│  │  - Chart.js for visualizations                        │  │
│  │  - Event-driven architecture                          │  │
│  └────────────────────────────────────────────────────────┘  │
│                          │                                   │
│                          │ WebSocket + REST                  │
└──────────────────────────┼───────────────────────────────────┘
                           │
┌──────────────────────────┼───────────────────────────────────┐
│                   Control Center Server                      │
│  ┌────────────────────────────────────────────────────────┐  │
│  │               FastAPI Application                      │  │
│  │  ┌──────────────┐    ┌──────────────┐                │  │
│  │  │ REST API     │    │  WebSocket   │                │  │
│  │  │ Endpoints    │    │  Handler     │                │  │
│  │  └──────┬───────┘    └──────┬───────┘                │  │
│  │         │                    │                         │  │
│  │         └────────┬───────────┘                         │  │
│  │                  │                                     │  │
│  │         ┌────────▼─────────┐                          │  │
│  │         │  LifecycleManager│                          │  │
│  │         │  - Process mgmt  │                          │  │
│  │         │  - Status track  │                          │  │
│  │         │  - Metrics       │                          │  │
│  │         └────────┬─────────┘                          │  │
│  │                  │                                     │  │
│  │       ┌──────────┴──────────┐                         │  │
│  │       │                     │                          │  │
│  │  ┌────▼─────┐         ┌────▼─────┐                   │  │
│  │  │subprocess│         │ psutil   │                   │  │
│  │  │ (Popen)  │         │ monitor  │                   │  │
│  │  └────┬─────┘         └────┬─────┘                   │  │
│  │       │                    │                          │  │
│  └───────┼────────────────────┼──────────────────────────┘  │
│          │                    │                             │
└──────────┼────────────────────┼─────────────────────────────┘
           │                    │
           │                    └─► System Metrics
           │
           └─► Managed Components
               ┌──────────────┐    ┌──────────────┐
               │ Storage      │    │ Sutra API    │
               │ Engine       │    │ Server       │
               └──────────────┘    └──────────────┘
```

## Component Breakdown

### 1. **Frontend Layer**

#### Dashboard UI (`templates/dashboard.html`)
- **Purpose**: Single-page application for monitoring and control
- **Technology**: Semantic HTML5 with accessibility features
- **Features**:
  - System metrics cards
  - Component status cards
  - Performance charts
  - Control buttons

#### Styling (`static/css/dashboard.css`)
- **Design System**: CSS variables for theming
- **Theme**: Dark mode with gradient accents
- **Layout**: CSS Grid + Flexbox
- **Responsive**: Mobile-first design with breakpoints
- **Colors**:
  - Primary: `#667eea` (purple-blue)
  - Success: `#48bb78` (green)
  - Warning: `#ed8936` (orange)
  - Error: `#f56565` (red)
  - Background: `#0f172a` → `#1e293b` (gradient)

#### JavaScript (`static/js/dashboard.js`)
- **Architecture**: Event-driven with functional programming
- **Key Functions**:
  - `connectWebSocket()`: Establish and maintain WebSocket connection
  - `updateDashboard(data)`: Main update orchestrator
  - `updateMetrics(metrics)`: Update metric cards
  - `updateComponents(components)`: Render component cards
  - `updateCharts(metrics)`: Update Chart.js instances
  - `controlComponent(name, action)`: Send control commands
- **Chart Management**:
  - Ring buffer for data points (MAX_DATA_POINTS = 20)
  - No-animation updates for smooth 60 FPS
  - Automatic scale adjustment

### 2. **Backend Layer**

#### FastAPI Application (`sutra_control/main.py`)

**Application Structure**:
```python
app = FastAPI(
    title="Sutra Control Center",
    version="0.1.0"
)

# Middleware
- StaticFiles for /static
- CORS (if needed)

# Routes
- GET /              → Dashboard HTML
- GET /api/status    → Current status
- POST /api/components/{name}/start
- POST /api/components/{name}/stop
- POST /api/components/{name}/restart
- WS /ws             → Real-time updates
```

#### LifecycleManager

**Responsibilities**:
1. **Component Registry**: Track all manageable components
2. **Process Management**: Start/stop/restart via subprocess
3. **Status Tracking**: Monitor component state and health
4. **Metrics Collection**: Gather system and process metrics

**State Machine**:
```
    ┌─────────┐
    │ STOPPED │
    └────┬────┘
         │ start()
         ▼
    ┌──────────┐
    │ STARTING │
    └────┬─────┘
         │ (process up)
         ▼
    ┌─────────┐  restart()  ┌───────────┐
    │ RUNNING │◄────────────┤ STOPPING  │
    └────┬────┘              └─────┬─────┘
         │ stop()                  │
         └─────────────────────────┘
         
    (any state) → ERROR (on failure)
```

**Component Definition**:
```python
class ComponentStatus(BaseModel):
    name: str
    state: ComponentState  # STOPPED, STARTING, RUNNING, STOPPING, ERROR
    pid: Optional[int]
    uptime: Optional[float]
    cpu_percent: Optional[float]
    memory_mb: Optional[float]
    error: Optional[str]
    last_updated: str
```

**Process Management**:
```python
# Start component
process = subprocess.Popen(
    command_args,
    cwd=working_directory,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE
)

# Monitor with psutil
proc = psutil.Process(pid)
cpu = proc.cpu_percent(interval=0.1)
memory = proc.memory_info().rss / 1024 / 1024
```

### 3. **Communication Layer**

#### REST API

**Endpoint Design**:
- **Idempotent operations**: GET, POST with clear semantics
- **Resource-oriented**: `/api/components/{name}/{action}`
- **JSON responses**: Consistent error handling
- **HTTP status codes**:
  - 200: Success
  - 201: Created (after start)
  - 400: Bad request
  - 404: Component not found
  - 500: Server error

#### WebSocket Protocol

**Connection Flow**:
```
Client                          Server
  │                               │
  ├────── WS Handshake ──────────►│
  │                               │
  │◄─────── Accept ───────────────┤
  │                               │
  │◄── Status Update (JSON) ──────┤ (every 2s)
  │                               │
  │◄── Status Update (JSON) ──────┤
  │                               │
  ...                            ...
  │                               │
  ├────── Close ─────────────────►│
```

**Message Format**:
```json
{
  "components": {
    "storage": {
      "name": "Storage Engine",
      "state": "running",
      "pid": 12345,
      "uptime": 3600.5,
      "cpu_percent": 2.1,
      "memory_mb": 45.2,
      "error": null,
      "last_updated": "2025-10-17T05:00:00Z"
    },
    "api": { ... }
  },
  "metrics": {
    "timestamp": "2025-10-17T05:00:00Z",
    "total_concepts": 1000,
    "total_associations": 5000,
    "storage_size_mb": 12.5,
    "cpu_percent": 5.2,
    "memory_percent": 45.1
  }
}
```

**Error Handling**:
- Automatic reconnection on disconnect
- Exponential backoff (3 seconds base)
- Client-side connection status indicator

### 4. **Process Orchestration**

#### Component Lifecycle

**Startup Sequence**:
1. Create `subprocess.Popen` with proper environment
2. Set state to STARTING
3. Wait 2 seconds for process initialization
4. Check `process.poll()` for liveness
5. If alive, set state to RUNNING and record PID
6. If dead, set state to ERROR with message

**Shutdown Sequence**:
1. Set state to STOPPING
2. Send `SIGTERM` via `process.terminate()`
3. Wait 10 seconds for graceful shutdown
4. If still alive, send `SIGKILL` via `process.kill()`
5. Set state to STOPPED

**Health Monitoring**:
```python
# Every update cycle (2 seconds)
for component in components:
    if component.pid:
        try:
            proc = psutil.Process(component.pid)
            component.cpu_percent = proc.cpu_percent()
            component.memory_mb = proc.memory_info().rss / 1024 / 1024
            component.uptime = time.time() - start_times[component]
        except psutil.NoSuchProcess:
            component.state = ERROR
            component.error = "Process died unexpectedly"
```

## Data Flow

### Monitoring Data Flow

```
System Resources          Component Processes
     │                           │
     │ psutil.cpu_percent()      │ psutil.Process(pid)
     │ psutil.virtual_memory()   │ .cpu_percent()
     │                           │ .memory_info()
     ▼                           ▼
┌──────────────────────────────────────┐
│       LifecycleManager               │
│  - update_status()                   │
│  - get_system_metrics()              │
└──────────────┬───────────────────────┘
               │
               │ Every 2 seconds
               ▼
┌──────────────────────────────────────┐
│    WebSocket Broadcast               │
│  - Serialize to JSON                 │
│  - Send to all connected clients     │
└──────────────┬───────────────────────┘
               │
               │ ws.send_json(data)
               ▼
┌──────────────────────────────────────┐
│    Browser Client                    │
│  - updateDashboard(data)             │
│  - Update DOM                        │
│  - Update charts                     │
└──────────────────────────────────────┘
```

### Control Command Flow

```
┌──────────────────────────────────────┐
│  User clicks "Start" button          │
└──────────────┬───────────────────────┘
               │
               │ onClick handler
               ▼
┌──────────────────────────────────────┐
│  controlComponent('api', 'start')    │
│  - POST /api/components/api/start    │
└──────────────┬───────────────────────┘
               │
               │ HTTP POST
               ▼
┌──────────────────────────────────────┐
│  FastAPI Endpoint Handler            │
│  - Validate component name           │
│  - Call manager.start_component()    │
└──────────────┬───────────────────────┘
               │
               │ async call
               ▼
┌──────────────────────────────────────┐
│  LifecycleManager                    │
│  - subprocess.Popen(command)         │
│  - Update component state            │
└──────────────┬───────────────────────┘
               │
               │ Response
               ▼
┌──────────────────────────────────────┐
│  Browser receives success/error      │
│  - Alert if error                    │
│  - UI updates via WebSocket          │
└──────────────────────────────────────┘
```

## Scalability Considerations

### Current Architecture (v0.1.0)
- **Single server**: No horizontal scaling
- **In-memory state**: No persistence
- **Single node**: Cannot manage remote components
- **No load balancing**: Direct client connections

### Future Architecture (v0.4.0+)
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Client 1   │    │  Client 2   │    │  Client N   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │
       └──────────────────┼───────────────────┘
                          │
                    ┌─────▼─────┐
                    │  L nginx  │
                    │  B        │
                    └─────┬─────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
   │Control  │      │Control  │      │Control  │
   │Center 1 │      │Center 2 │      │Center N │
   └────┬────┘      └────┬────┘      └────┬────┘
        │                │                 │
        └────────────────┼─────────────────┘
                         │
                    ┌────▼────┐
                    │  Shared │
                    │  State  │
                    │ (Redis) │
                    └─────────┘
```

## Security Architecture

### v0.1.0 (Development)
```
Browser ──[HTTP/WS]──► Control Center ──[Subprocess]──► Components
          (no auth)    (no encryption)   (no isolation)
```

### Planned (v0.4.0+)
```
Browser ──[HTTPS/WSS + JWT]──► Control Center ──[Secure IPC]──► Components
          (authenticated)       (RBAC enforced)  (containerized)
                                      │
                                      └──► Audit Log (immutable)
```

## Performance Optimization

### Backend
- Async I/O throughout (FastAPI + asyncio)
- Efficient process monitoring (psutil caching)
- Minimal WebSocket broadcast (only changed data)

### Frontend
- No-animation chart updates for 60 FPS
- Ring buffer for chart data (prevent memory growth)
- DOM batch updates (single render per WebSocket message)

### Network
- WebSocket persistent connection (no polling overhead)
- JSON compression (gzip if needed)
- CDN for Chart.js (external, cached)

## Deployment Topology

### Development
```
Developer Machine
├── Control Center (port 9000)
├── Sutra API (port 8000)
└── Storage (embedded)
```

### Production (Future)
```
Server 1              Server 2              Server 3
├── Control Center    ├── Sutra API         ├── Monitoring
├── nginx             ├── Storage Engine    ├── Logging
└── TLS Termination   └── Load Balancer     └── Alerting
```

## Technology Decisions

### Why FastAPI?
- Native async/await support
- Built-in WebSocket support
- OpenAPI documentation
- High performance (Starlette/Uvicorn)
- Type hints and validation (Pydantic)

### Why Vanilla JS?
- No build step required
- Fast initial load
- Minimal dependencies
- Easy to understand and modify
- No framework lock-in

### Why Chart.js?
- Simple API
- Smooth animations
- Good mobile support
- CDN availability
- Active maintenance

### Why subprocess?
- Simple process management
- Cross-platform compatibility
- Standard library (no deps)
- Easy debugging
- Future: can migrate to systemd/Docker

## See Also

- [Component Details](components.md)
- [Data Flow Diagrams](data-flow.md)
- [API Reference](../api/rest-endpoints.md)
