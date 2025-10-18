//! Storage Server Binary
//!
//! Production TCP server for Sutra storage using custom binary protocol.
//! Replaces gRPC with 10-50× better performance.

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use sutra_storage::{ConcurrentConfig, ConcurrentMemory};
use sutra_storage::tcp_server::StorageServer;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("Starting Sutra Storage Server (TCP)");

    // Load configuration from environment
    let storage_path = env::var("STORAGE_PATH")
        .unwrap_or_else(|_| "/data/storage.dat".to_string());
    
    let host = env::var("STORAGE_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    
    let port = env::var("STORAGE_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .unwrap_or(50051);

    let reconcile_interval_ms = env::var("RECONCILE_INTERVAL_MS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u64>()
        .unwrap_or(10);

    let memory_threshold = env::var("MEMORY_THRESHOLD")
        .unwrap_or_else(|_| "50000".to_string())
        .parse::<usize>()
        .unwrap_or(50000);

    let vector_dimension = env::var("VECTOR_DIMENSION")
        .unwrap_or_else(|_| "768".to_string())
        .parse::<usize>()
        .unwrap_or(768);

    info!("Configuration:");
    info!("  Storage path: {}", storage_path);
    info!("  Listen address: {}:{}", host, port);
    info!("  Reconcile interval: {}ms", reconcile_interval_ms);
    info!("  Memory threshold: {} writes", memory_threshold);
    info!("  Vector dimension: {}", vector_dimension);

    // Initialize storage
    let config = ConcurrentConfig {
        storage_path: storage_path.into(),
        reconcile_interval_ms,
        memory_threshold,
        vector_dimension,
    };

    info!("Initializing storage...");
    let storage = ConcurrentMemory::new(config);
    
    let stats = storage.stats();
    info!("Storage initialized:");
    info!("  Concepts: {}", stats.snapshot.concept_count);
    info!("  Edges: {}", stats.snapshot.edge_count);
    info!("  Sequence: {}", stats.snapshot.sequence);

    // Create server
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let server = Arc::new(StorageServer::new(storage));

    info!("Starting TCP server on {}", addr);
    
    // Start server (blocks until shutdown)
    if let Err(e) = server.serve(addr).await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    info!("Server shutdown complete");
    Ok(())
}
