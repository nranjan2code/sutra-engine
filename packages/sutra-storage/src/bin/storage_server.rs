//! Storage Server Binary
//!
//! Production TCP server for Sutra storage using custom binary protocol.
//! Replaces gRPC with 10-50× better performance.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use sutra_storage::{ConcurrentConfig, ConcurrentMemory, ShardedStorage, ShardConfig};
use sutra_storage::tcp_server::{StorageServer, ShardedStorageServer};
use tracing::{info, error, warn};
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
    
    // Sharding configuration
    let storage_mode = env::var("SUTRA_STORAGE_MODE")
        .unwrap_or_else(|_| "single".to_string());
    
    let num_shards = env::var("SUTRA_NUM_SHARDS")
        .unwrap_or_else(|_| "16".to_string())
        .parse::<u32>()
        .unwrap_or(16);

    info!("Configuration:");
    info!("  Storage mode: {}", storage_mode);
    info!("  Storage path: {}", storage_path);
    info!("  Listen address: {}:{}", host, port);
    info!("  Reconcile interval: {}ms", reconcile_interval_ms);
    info!("  Memory threshold: {} writes", memory_threshold);
    info!("  Vector dimension: {}", vector_dimension);
    if storage_mode == "sharded" {
        info!("  Number of shards: {}", num_shards);
    }

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Initialize storage based on mode
    match storage_mode.as_str() {
        "sharded" => {
            info!("Initializing SHARDED storage with {} shards...", num_shards);
            
            let shard_config = ConcurrentConfig {
                storage_path: PathBuf::from(&storage_path), // Will be overridden per shard
                reconcile_interval_ms,
                memory_threshold,
                vector_dimension,
            };
            
            let config = ShardConfig {
                num_shards,
                base_path: PathBuf::from(&storage_path),
                shard_config,
            };
            
            let sharded_storage = ShardedStorage::new(config)
                .map_err(|e| format!("Failed to initialize sharded storage: {}", e))?;
            
            let stats = sharded_storage.stats();
            info!("✅ Sharded storage initialized:");
            info!("  Total concepts: {}", stats.total_concepts);
            info!("  Total edges: {}", stats.total_edges);
            info!("  Total vectors: {}", stats.total_vectors);
            info!("  Total writes: {}", stats.total_writes);
            info!("  Shards: {}", stats.num_shards);
            
            // Create sharded server
            let server = Arc::new(ShardedStorageServer::new(sharded_storage));
            
            info!("🚀 Starting SHARDED TCP server on {}", addr);
            
            // Start server (blocks until shutdown)
            if let Err(e) = server.serve(addr).await {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        "single" | _ => {
            if storage_mode != "single" {
                warn!("Unknown storage mode '{}', defaulting to 'single'", storage_mode);
            }
            
            info!("Initializing SINGLE storage...");
            
            let config = ConcurrentConfig {
                storage_path: storage_path.into(),
                reconcile_interval_ms,
                memory_threshold,
                vector_dimension,
            };
            
            let storage = ConcurrentMemory::new(config);
            
            let stats = storage.stats();
            info!("✅ Single storage initialized:");
            info!("  Concepts: {}", stats.snapshot.concept_count);
            info!("  Edges: {}", stats.snapshot.edge_count);
            info!("  Sequence: {}", stats.snapshot.sequence);
            
            // Create server
            let server = Arc::new(StorageServer::new(storage));
            
            info!("🚀 Starting SINGLE TCP server on {}", addr);
            
            // Start server (blocks until shutdown)
            if let Err(e) = server.serve(addr).await {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
    }

    info!("Server shutdown complete");
    Ok(())
}
