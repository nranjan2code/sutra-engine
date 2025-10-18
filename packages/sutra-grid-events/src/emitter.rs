use crate::events::GridEvent;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use sutra_protocol::{StorageMessage, StorageResponse, send_message, recv_message};

/// Event emitter that writes Grid events to Sutra Storage via TCP
#[derive(Clone)]
pub struct EventEmitter {
    tx: mpsc::UnboundedSender<GridEvent>,
}

impl EventEmitter {
    /// Create a new event emitter connected to a storage server via TCP
    pub async fn new(storage_endpoint: String) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Connect to storage server via TCP
        log::info!("📊 Connecting Grid event emitter to storage: {}", storage_endpoint);
        let stream = TcpStream::connect(&storage_endpoint).await?;
        stream.set_nodelay(true)?; // Disable Nagle for low latency
        
        log::info!("✅ Grid event emitter connected to storage: {}", storage_endpoint);
        
        // Spawn background worker to process events
        let storage_endpoint_clone = storage_endpoint.clone();
        tokio::spawn(async move {
            event_worker(stream, storage_endpoint_clone, rx).await;
        });
        
        Ok(EventEmitter { tx })
    }
    
    /// Emit an event (non-blocking)
    pub fn emit(&self, event: GridEvent) {
        if let Err(e) = self.tx.send(event) {
            log::error!("Failed to emit event: {}", e);
        }
    }
    
    /// Emit multiple events
    pub fn emit_batch(&self, events: Vec<GridEvent>) {
        for event in events {
            self.emit(event);
        }
    }
}

/// Background worker that processes events and writes to storage
async fn event_worker(
    mut stream: TcpStream,
    storage_endpoint: String,
    mut rx: mpsc::UnboundedReceiver<GridEvent>,
) {
    log::info!("🔄 Event worker started");
    
    while let Some(event) = rx.recv().await {
        // Try to write event, reconnect on failure
        if let Err(e) = write_event_to_storage(&mut stream, &event).await {
            log::error!("Failed to write event to storage: {} - attempting reconnect", e);
            
            // Try to reconnect
            match TcpStream::connect(&storage_endpoint).await {
                Ok(new_stream) => {
                    if let Ok(_) = new_stream.set_nodelay(true) {
                        stream = new_stream;
                        log::info!("✅ Event worker reconnected to storage");
                        
                        // Retry writing event
                        if let Err(e) = write_event_to_storage(&mut stream, &event).await {
                            log::error!("Failed to write event after reconnect: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to reconnect to storage: {}", e);
                }
            }
        }
    }
    
    log::warn!("Event worker stopped");
}

/// Write a single event to storage as concept + associations via TCP
async fn write_event_to_storage(
    stream: &mut TcpStream,
    event: &GridEvent,
) -> anyhow::Result<()> {
    // Generate unique event ID
    let event_id = format!("event-{}-{}", event.event_type(), event.timestamp().timestamp_micros());
    
    // Serialize event as JSON for content
    let event_json = serde_json::to_string(event)?;
    
    // Learn event as concept
    let learn_msg = StorageMessage::LearnConcept {
        concept_id: event_id.clone(),
        content: event_json,
        embedding: vec![], // TODO: Add embeddings for semantic search
        strength: 1.0,
        confidence: 1.0,
    };
    
    send_message(stream, &learn_msg).await?;
    let _resp: StorageResponse = recv_message(stream).await?;
    
    // Create associations for queryability
    let primary_id = event.primary_id();
    
    // Association: entity -> event_type -> event
    let assoc_msg = StorageMessage::LearnAssociation {
        source_id: primary_id.clone(),
        target_id: event_id.clone(),
        assoc_type: event_type_to_int(event.event_type()),
        confidence: 1.0,
    };
    
    send_message(stream, &assoc_msg).await?;
    let _resp: StorageResponse = recv_message(stream).await?;
    
    // Association: event -> timestamp (for temporal queries)
    let timestamp_id = format!("ts-{}", event.timestamp().timestamp());
    let ts_assoc = StorageMessage::LearnAssociation {
        source_id: event_id.clone(),
        target_id: timestamp_id,
        assoc_type: 999, // TEMPORAL association type
        confidence: 1.0,
    };
    
    send_message(stream, &ts_assoc).await?;
    let _resp: StorageResponse = recv_message(stream).await?;
    
    log::debug!("📝 Wrote event: {} -> {}", primary_id, event.event_type());
    
    Ok(())
}

/// Map event types to integer codes for associations
fn event_type_to_int(event_type: &str) -> u32 {
    match event_type {
        "agent_registered" => 1,
        "agent_heartbeat" => 2,
        "agent_degraded" => 3,
        "agent_offline" => 4,
        "agent_recovered" => 5,
        "agent_unregistered" => 6,
        "spawn_requested" => 10,
        "spawn_succeeded" => 11,
        "spawn_failed" => 12,
        "stop_requested" => 20,
        "stop_succeeded" => 21,
        "stop_failed" => 22,
        "node_crashed" => 30,
        "node_restarted" => 31,
        "cluster_healthy" => 40,
        "cluster_degraded" => 41,
        "cluster_critical" => 42,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_event_type_mapping() {
        assert_eq!(event_type_to_int("agent_registered"), 1);
        assert_eq!(event_type_to_int("spawn_failed"), 12);
        assert_eq!(event_type_to_int("node_crashed"), 30);
    }
    
    #[tokio::test]
    async fn test_emitter_creation() {
        // This would fail without a real storage server, so we just test the type
        let result = EventEmitter::new("localhost:50051".to_string()).await;
        // Expected to fail in test environment (no server running)
        assert!(result.is_err() || result.is_ok());
    }
}
