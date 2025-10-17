# REST API Reference

## Base URL

```
http://localhost:9000
```

## Authentication

**v0.1.0**: No authentication required (development only)  
**Future**: JWT-based authentication with Bearer tokens

## Endpoints

### Dashboard

#### GET `/`

Serves the control center dashboard HTML.

**Response**: HTML page

```http
GET / HTTP/1.1
Host: localhost:9000

HTTP/1.1 200 OK
Content-Type: text/html
```

---

### System Status

#### GET `/api/status`

Get current system status including all components and metrics.

**Response**: JSON

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
    "api": {
      "name": "Sutra API",
      "state": "stopped",
      "pid": null,
      "uptime": null,
      "cpu_percent": null,
      "memory_mb": null,
      "error": null,
      "last_updated": "2025-10-17T05:00:00Z"
    }
  },
  "metrics": {
    "timestamp": "2025-10-17T05:00:00Z",
    "total_concepts": 1000,
    "total_associations": 5000,
    "storage_size_mb": 12.5,
    "queries_per_second": 0.0,
    "avg_latency_ms": 0.0,
    "cpu_percent": 5.2,
    "memory_percent": 45.1
  }
}
```

**Status Codes**:
- `200 OK`: Success

---

### Component Management

#### POST `/api/components/{component}/start`

Start a component.

**Path Parameters**:
- `component` (string): Component name (`storage` or `api`)

**Request**: Empty body

**Response**: JSON

```json
{
  "success": true,
  "component": {
    "name": "Sutra API",
    "state": "running",
    "pid": 23456,
    "uptime": 2.0,
    "cpu_percent": 0.0,
    "memory_mb": 50.0,
    "error": null,
    "last_updated": "2025-10-17T05:00:02Z"
  }
}
```

**Error Response**:
```json
{
  "success": false,
  "error": "Component already running"
}
```

**Status Codes**:
- `200 OK`: Component started successfully
- `400 Bad Request`: Invalid component name or component already running
- `500 Internal Server Error`: Failed to start component

**Example**:
```bash
curl -X POST http://localhost:9000/api/components/api/start
```

---

#### POST `/api/components/{component}/stop`

Stop a component.

**Path Parameters**:
- `component` (string): Component name (`storage` or `api`)

**Request**: Empty body

**Response**: JSON

```json
{
  "success": true,
  "component": {
    "name": "Sutra API",
    "state": "stopped",
    "pid": null,
    "uptime": null,
    "cpu_percent": null,
    "memory_mb": null,
    "error": null,
    "last_updated": "2025-10-17T05:01:00Z"
  }
}
```

**Status Codes**:
- `200 OK`: Component stopped successfully
- `400 Bad Request`: Invalid component name or component already stopped
- `500 Internal Server Error`: Failed to stop component

**Example**:
```bash
curl -X POST http://localhost:9000/api/components/api/stop
```

---

#### POST `/api/components/{component}/restart`

Restart a component (stop then start).

**Path Parameters**:
- `component` (string): Component name (`storage` or `api`)

**Request**: Empty body

**Response**: JSON

```json
{
  "success": true,
  "component": {
    "name": "Sutra API",
    "state": "running",
    "pid": 23457,
    "uptime": 1.0,
    "cpu_percent": 0.0,
    "memory_mb": 48.0,
    "error": null,
    "last_updated": "2025-10-17T05:02:00Z"
  }
}
```

**Status Codes**:
- `200 OK`: Component restarted successfully
- `400 Bad Request`: Invalid component name
- `500 Internal Server Error`: Failed to restart component

**Example**:
```bash
curl -X POST http://localhost:9000/api/components/api/restart
```

---

## Data Models

### ComponentStatus

```typescript
interface ComponentStatus {
  name: string;              // Display name
  state: ComponentState;     // Current state
  pid: number | null;        // Process ID (null if stopped)
  uptime: number | null;     // Uptime in seconds (null if stopped)
  cpu_percent: number | null; // CPU usage % (null if stopped)
  memory_mb: number | null;  // Memory usage MB (null if stopped)
  error: string | null;      // Error message (null if no error)
  last_updated: string;      // ISO 8601 timestamp
}
```

### ComponentState

```typescript
enum ComponentState {
  STOPPED = "stopped",
  STARTING = "starting",
  RUNNING = "running",
  STOPPING = "stopping",
  ERROR = "error"
}
```

### SystemMetrics

```typescript
interface SystemMetrics {
  timestamp: string;         // ISO 8601 timestamp
  total_concepts: number;    // Total learned concepts
  total_associations: number; // Total associations
  storage_size_mb: number;   // Storage size in MB
  queries_per_second: number; // QPS (future)
  avg_latency_ms: number;    // Average latency (future)
  cpu_percent: number;       // System CPU %
  memory_percent: number;    // System memory %
}
```

## Error Handling

All endpoints use consistent error format:

```json
{
  "success": false,
  "error": "Error message describing what went wrong"
}
```

### Common Error Codes

| Code | Meaning |
|------|---------|
| 400 | Bad Request - Invalid component name or invalid operation |
| 404 | Not Found - Endpoint not found |
| 500 | Internal Server Error - Unexpected server error |

## Rate Limiting

**v0.1.0**: No rate limiting  
**Future**: 
- Control endpoints: 10 requests/minute per IP
- Status endpoints: 60 requests/minute per IP

## CORS

**v0.1.0**: CORS disabled (same-origin only)  
**Future**: Configurable CORS with whitelist

## Pagination

Not applicable - all endpoints return complete data.

## Versioning

**v0.1.0**: No API versioning  
**Future**: Version in URL path (`/api/v1/...`)

## OpenAPI Documentation

Interactive API documentation available at:
- Swagger UI: `http://localhost:9000/docs`
- ReDoc: `http://localhost:9000/redoc`
- OpenAPI JSON: `http://localhost:9000/openapi.json`

## Code Examples

### Python

```python
import requests

BASE_URL = "http://localhost:9000"

# Get status
response = requests.get(f"{BASE_URL}/api/status")
status = response.json()
print(f"Storage state: {status['components']['storage']['state']}")

# Start API
response = requests.post(f"{BASE_URL}/api/components/api/start")
result = response.json()
if result['success']:
    print(f"API started with PID {result['component']['pid']}")
else:
    print(f"Failed: {result['error']}")
```

### JavaScript

```javascript
const BASE_URL = "http://localhost:9000";

// Get status
fetch(`${BASE_URL}/api/status`)
  .then(res => res.json())
  .then(data => {
    console.log("Storage state:", data.components.storage.state);
  });

// Start API
fetch(`${BASE_URL}/api/components/api/start`, { method: "POST" })
  .then(res => res.json())
  .then(result => {
    if (result.success) {
      console.log("API started with PID", result.component.pid);
    } else {
      console.error("Failed:", result.error);
    }
  });
```

### cURL

```bash
# Get status
curl http://localhost:9000/api/status | jq

# Start API
curl -X POST http://localhost:9000/api/components/api/start | jq

# Stop API
curl -X POST http://localhost:9000/api/components/api/stop | jq

# Restart API
curl -X POST http://localhost:9000/api/components/api/restart | jq
```

## WebSocket

See [WebSocket Protocol](websocket-protocol.md) for real-time updates.

## See Also

- [WebSocket Protocol](websocket-protocol.md)
- [Data Models](models.md)
- [Architecture Overview](../architecture/overview.md)
