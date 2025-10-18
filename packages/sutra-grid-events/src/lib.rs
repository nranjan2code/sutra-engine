/*!
# Sutra Grid Events

Event-driven observability for Sutra Grid using Sutra Storage as the backend.

Instead of traditional logs/metrics/telemetry, Grid components emit structured events
that are stored in Sutra's knowledge graph for semantic and temporal querying.

## Architecture

```text
Grid Components → EventEmitter → Sutra Storage → Sutra Control (Chat UI)
```

## Usage

```rust
use sutra_grid_events::{EventEmitter, GridEvent};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to reserved storage instance
    let emitter = EventEmitter::new("http://localhost:50051".to_string()).await?;
    
    // Emit events
    emitter.emit(GridEvent::AgentRegistered {
        agent_id: "agent-001".to_string(),
        hostname: "host1".to_string(),
        platform: "linux".to_string(),
        agent_endpoint: "host1:8001".to_string(),
        max_storage_nodes: 5,
        timestamp: Utc::now(),
    });
    
    Ok(())
}
```

## Event Types

- **Agent Lifecycle**: Registered, Heartbeat, Degraded, Offline, Recovered, Unregistered
- **Node Lifecycle**: SpawnRequested, SpawnSucceeded, SpawnFailed, StopRequested, StopSucceeded, StopFailed
- **Node Health**: NodeCrashed, NodeRestarted
- **Cluster Health**: ClusterHealthy, ClusterDegraded, ClusterCritical

## Querying Events

Events can be queried through Sutra Control's chat interface:

- "Show me all spawn failures today"
- "Which agents went offline in the last hour?"
- "What's the crash history of node-abc123?"
- "List all restart events for agent-001"

*/

pub mod events;
pub mod emitter;

pub use events::GridEvent;
pub use emitter::EventEmitter;

/// Initialize event emission for a Grid component
pub async fn init_events(storage_endpoint: String) -> anyhow::Result<EventEmitter> {
    EventEmitter::new(storage_endpoint).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_exports() {
        // Ensure all public types are accessible
        let _event_type: Option<GridEvent> = None;
    }
}
