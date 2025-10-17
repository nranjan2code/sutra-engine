# WebSocket Protocol

## Overview

The Control Center uses WebSocket for **real-time bidirectional communication** between server and browser clients. The server broadcasts status updates every 2 seconds to all connected clients.

## Connection

### Endpoint

```
ws://localhost:9000/ws
```

(Use `wss://` for secure connections in production)

### Handshake

```http
GET /ws HTTP/1.1
Host: localhost:9000
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13

HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

### Client-Side Connection (JavaScript)

```javascript
const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

const ws = new WebSocket(wsUrl);

ws.onopen = () => {
    console.log('Connected to Control Center');
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    // Process update
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Disconnected from Control Center');
    // Implement reconnection logic
};
```

## Message Format

### Server → Client: Status Update

Sent every **2 seconds** with complete system state.

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

### Client → Server

**v0.1.0**: No client-to-server messages (read-only WebSocket).  
Control commands use REST API endpoints.

**Future v0.2.0+**: Client may send subscription preferences:

```json
{
  "type": "subscribe",
  "components": ["storage", "api"],
  "metrics": ["cpu", "memory"]
}
```

## Connection Lifecycle

```
┌──────────┐                                ┌──────────┐
│  Client  │                                │  Server  │
└────┬─────┘                                └────┬─────┘
     │                                           │
     ├─────── WS Handshake ────────────────────►│
     │                                           │
     │◄────── 101 Switching Protocols ──────────┤
     │                                           │
     │                   Connected               │
     │                                           │
     │◄── Status Update (JSON) ──────────────────┤
     │                                           │
     │     (2 seconds)                           │
     │                                           │
     │◄── Status Update (JSON) ──────────────────┤
     │                                           │
     │     (2 seconds)                           │
     │                                           │
     │◄── Status Update (JSON) ──────────────────┤
     │                                           │
     ...                                        ...
     │                                           │
     ├─────── Close Frame ──────────────────────►│
     │                                           │
     │                 Disconnected              │
     │                                           │
```

## Error Handling

### Connection Errors

**Network Failure**: 
- Client automatically attempts reconnection
- Exponential backoff starting at 3 seconds
- Status indicator shows disconnected state

**Server Restart**:
- WebSocket closes with code 1006
- Client reconnects automatically
- No data loss (server rebuilds state on startup)

### Message Errors

If server sends invalid JSON:
```javascript
ws.onmessage = (event) => {
    try {
        const data = JSON.parse(event.data);
        updateDashboard(data);
    } catch (error) {
        console.error('Invalid message from server:', error);
    }
};
```

## Close Codes

| Code | Meaning | Action |
|------|---------|--------|
| 1000 | Normal closure | No reconnection |
| 1001 | Going away (server restart) | Reconnect after 3s |
| 1006 | Abnormal closure | Reconnect after 3s |
| 1008 | Policy violation | Alert user, no reconnect |
| 1011 | Internal server error | Reconnect with backoff |

## Reconnection Strategy

```javascript
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;
const BASE_DELAY = 3000; // 3 seconds

function connectWebSocket() {
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
        reconnectAttempts = 0;
        updateConnectionStatus(true);
    };
    
    ws.onclose = () => {
        updateConnectionStatus(false);
        
        if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
            const delay = BASE_DELAY * Math.pow(2, reconnectAttempts);
            reconnectAttempts++;
            
            console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts})`);
            setTimeout(connectWebSocket, delay);
        } else {
            console.error('Max reconnection attempts reached');
            alert('Lost connection to Control Center. Please refresh the page.');
        }
    };
}
```

## Performance Characteristics

### Bandwidth

**Per Update** (~2KB compressed):
- Components: ~500 bytes
- Metrics: ~300 bytes
- Overhead: ~200 bytes

**Per Minute**: ~60 KB (30 updates × 2KB)  
**Per Hour**: ~3.6 MB

### Latency

- **LAN**: <10ms per update
- **Local Network**: 10-50ms
- **Internet**: 50-200ms

### Connection Limits

**v0.1.0**: No enforced limit (typically handles 100+ concurrent clients)  
**Future**: Configurable per-server limit with graceful degradation

## Security Considerations

### v0.1.0 (Development)

⚠️ **No security** - suitable for local development only:
- No authentication
- No encryption (ws://)
- No origin validation
- No rate limiting

### Future (Production)

- **TLS encryption** (wss://)
- **JWT authentication** in handshake
- **Origin validation** (CORS-like for WebSocket)
- **Per-client rate limiting**
- **Message signing** for integrity

## Best Practices

### Client Implementation

1. **Always handle reconnection**: Network is unreliable
2. **Show connection status**: User should know if data is stale
3. **Buffer updates**: Don't block UI thread on every message
4. **Graceful degradation**: Function (partially) without WebSocket

### Example: Reconnection with Visual Feedback

```javascript
class ControlCenterClient {
    constructor(url) {
        this.url = url;
        this.ws = null;
        this.reconnectTimer = null;
        this.statusIndicator = document.getElementById('ws-status');
    }
    
    connect() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = () => this.onOpen();
        this.ws.onmessage = (e) => this.onMessage(e);
        this.ws.onclose = () => this.onClose();
        this.ws.onerror = (e) => this.onError(e);
    }
    
    onOpen() {
        console.log('Connected');
        this.statusIndicator.className = 'status-dot connected';
        clearTimeout(this.reconnectTimer);
    }
    
    onMessage(event) {
        const data = JSON.parse(event.data);
        this.updateDashboard(data);
    }
    
    onClose() {
        console.log('Disconnected');
        this.statusIndicator.className = 'status-dot disconnected';
        
        // Reconnect after 3 seconds
        this.reconnectTimer = setTimeout(() => {
            console.log('Reconnecting...');
            this.connect();
        }, 3000);
    }
    
    onError(error) {
        console.error('WebSocket error:', error);
    }
    
    updateDashboard(data) {
        // Update UI with new data
    }
}

// Usage
const client = new ControlCenterClient('ws://localhost:9000/ws');
client.connect();
```

## Testing

### Manual Testing

```bash
# Using websocat
websocat ws://localhost:9000/ws

# Using wscat
wscat -c ws://localhost:9000/ws

# Using JavaScript (browser console)
ws = new WebSocket('ws://localhost:9000/ws');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

### Load Testing

```python
import asyncio
import websockets

async def stress_test():
    tasks = []
    for i in range(100):
        tasks.append(connect_client(i))
    await asyncio.gather(*tasks)

async def connect_client(client_id):
    uri = "ws://localhost:9000/ws"
    async with websockets.connect(uri) as ws:
        print(f"Client {client_id} connected")
        while True:
            message = await ws.recv()
            # Process message
            await asyncio.sleep(0.1)

asyncio.run(stress_test())
```

## Troubleshooting

### WebSocket Connection Fails

**Symptoms**: 
- Browser console shows WebSocket error
- Connection status shows disconnected

**Solutions**:
1. Check Control Center is running (`http://localhost:9000`)
2. Verify no firewall blocking port 9000
3. Check browser console for specific error codes
4. Try different browser (Safari sometimes has WebSocket issues)

### Messages Not Updating

**Symptoms**:
- Connected but dashboard not updating
- Old/stale data displayed

**Solutions**:
1. Check browser console for JavaScript errors
2. Verify JSON parsing is working
3. Check if updateDashboard() is being called
4. Inspect network tab for WebSocket frames

### High Latency

**Symptoms**:
- Delay between server state and UI
- Laggy charts

**Solutions**:
1. Check network latency (ping server)
2. Reduce update frequency if on slow network
3. Optimize updateDashboard() for performance
4. Check browser isn't throttling inactive tabs

## See Also

- [REST API Reference](rest-endpoints.md)
- [Architecture Overview](../architecture/overview.md)
- [Frontend Implementation](../ui-ux/components.md)
