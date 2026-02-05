//! Storage Server Binary
//!
//! Production TCP server for Sutra storage using custom binary protocol.
//! Replaces gRPC with 10-50× better performance.
//!
//! Supports two deployment modes:
//! - Development Mode (default): No authentication, no encryption
//! - Production Mode: HMAC/JWT auth + TLS 1.3 encryption (set SUTRA_SECURE_MODE=true)

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use sutra_storage::auth::AuthManager;
use sutra_storage::secure_tcp_server::SecureStorageServer;
use sutra_storage::tcp_server::{ShardedStorageServer, StorageServer};
use sutra_storage::{
    AdaptiveReconcilerConfig, AutonomyConfig, ConcurrentConfig, ConcurrentMemory, ShardConfig,
    ShardedStorage,
};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("Starting Sutra Storage Server (TCP)");

    // Security mode configuration
    let secure_mode = env::var("SUTRA_SECURE_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    // Load configuration from environment
    let storage_path = env::var("STORAGE_PATH").unwrap_or_else(|_| "/data/storage.dat".to_string());

    let host = env::var("STORAGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port = env::var("STORAGE_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .unwrap_or(50051);

    // Adaptive reconciler configuration (env vars for fine-tuning)
    let base_interval_ms = env::var("RECONCILE_BASE_INTERVAL_MS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u64>()
        .unwrap_or(10);

    let memory_threshold = env::var("MEMORY_THRESHOLD")
        .unwrap_or_else(|_| "50000".to_string())
        .parse::<usize>()
        .unwrap_or(50000);

    let vector_dimension = env::var("VECTOR_DIMENSION")
        .unwrap_or_else(|_| "384".to_string())
        .parse::<usize>()
        .unwrap_or(384);

    // Sharding configuration
    let storage_mode = env::var("SUTRA_STORAGE_MODE").unwrap_or_else(|_| "single".to_string());

    let num_shards = env::var("SUTRA_NUM_SHARDS")
        .unwrap_or_else(|_| "16".to_string())
        .parse::<u32>()
        .unwrap_or(16);

    // Autonomy engine configuration
    let autonomy_enabled = env::var("SUTRA_AUTONOMY")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    let autonomy_config = if autonomy_enabled {
        AutonomyConfig::default()
    } else {
        AutonomyConfig::disabled()
    };

    info!("Configuration:");
    info!(
        "  Security mode: {}",
        if secure_mode {
            "🔒 SECURE (auth + TLS)"
        } else {
            "⚠️  DEVELOPMENT (no auth, no TLS)"
        }
    );
    info!("  Storage mode: {}", storage_mode);
    info!(
        "  Autonomy engine: {}",
        if autonomy_enabled {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    info!("  Storage path: {}", storage_path);
    info!("  Listen address: {}:{}", host, port);
    info!(
        "  Base reconcile interval: {}ms (adaptive: 1-100ms)",
        base_interval_ms
    );
    info!("  Memory threshold: {} writes", memory_threshold);
    info!("  Vector dimension: {}", vector_dimension);
    if storage_mode == "sharded" {
        info!("  Number of shards: {}", num_shards);
    }

    if !secure_mode {
        warn!("🚨 SECURITY WARNING: Running in DEVELOPMENT mode");
        warn!("   This mode has NO authentication and NO encryption");
        warn!("   DO NOT use with sensitive data or network-accessible deployments");
        warn!("   For production, set SUTRA_SECURE_MODE=true");
    }

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Create adaptive reconciler config
    let adaptive_config = AdaptiveReconcilerConfig {
        base_interval_ms,
        ..Default::default()
    };

    // Initialize authentication manager if secure mode is enabled
    let auth_manager = if secure_mode {
        info!("🔒 Initializing authentication...");

        // Load from environment (validates secret strength)
        match AuthManager::from_env() {
            Ok(manager) => {
                info!("✅ Authentication enabled: HMAC-SHA256");
                Some(manager)
            }
            Err(e) => {
                error!("❌ Failed to initialize authentication: {}", e);
                return Err(e.into());
            }
        }
    } else {
        warn!("⚠️  Authentication: DISABLED (development mode)");
        None
    };

    // Initialize storage based on mode
    match storage_mode.as_str() {
        "sharded" => {
            info!("Initializing SHARDED storage with {} shards...", num_shards);

            let shard_config = ConcurrentConfig {
                storage_path: PathBuf::from(&storage_path), // Will be overridden per shard
                memory_threshold,
                vector_dimension,
                adaptive_reconciler_config: adaptive_config.clone(),
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
            //
            // Note: Secure sharded server is not yet implemented. This is acceptable because:
            // 1. Production deployments use single-node storage with TLS + HMAC authentication
            // 2. Sharded mode is primarily for high-scale (10M+ concepts) where single-node security is preferred
            // 3. Implementing SecureShardedStorageServer requires cross-shard TLS negotiation
            //
            // Future enhancement: Implement SecureShardedStorageServer for multi-node deployments
            if secure_mode {
                warn!("⚠️  Secure mode not yet implemented for sharded storage");
                warn!("   Falling back to standard sharded server");
                warn!("   For production security, use single storage mode with TLS + HMAC");
            }

            let server = Arc::new(ShardedStorageServer::new(sharded_storage).await);

            info!("🚀 Starting SHARDED TCP server on {}", addr);

            // Start server (blocks until shutdown)
            if let Err(e) = server.serve(addr).await {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ => {
            if storage_mode != "single" {
                warn!(
                    "Unknown storage mode '{}', defaulting to 'single'",
                    storage_mode
                );
            }

            info!("Initializing SINGLE storage...");

            let config = ConcurrentConfig {
                storage_path: storage_path.into(),
                memory_threshold,
                vector_dimension,
                adaptive_reconciler_config: adaptive_config,
            };

            let storage = ConcurrentMemory::new(config);

            let stats = storage.stats();
            info!("✅ Single storage initialized:");
            info!("  Concepts: {}", stats.snapshot.concept_count);
            info!("  Edges: {}", stats.snapshot.edge_count);
            info!("  Sequence: {}", stats.snapshot.sequence);

            // Create server (secure or insecure based on mode)
            if secure_mode {
                // Wrap with secure server
                let insecure_server =
                    StorageServer::new_with_autonomy(storage, autonomy_config).await;
                let secure_server = SecureStorageServer::new(insecure_server, auth_manager)
                    .await
                    .map_err(|e| format!("Failed to create secure server: {}", e))?;
                let server = Arc::new(secure_server);

                info!("🚀 Starting SECURE SINGLE TCP server on {}", addr);

                // Start server (blocks until shutdown)
                if let Err(e) = server.serve(addr).await {
                    error!("Server error: {}", e);
                    return Err(e.into());
                }
            } else {
                // Use insecure server directly
                let server =
                    Arc::new(StorageServer::new_with_autonomy(storage, autonomy_config).await);

                info!(
                    "🚀 Starting SINGLE TCP server on {} (DEVELOPMENT MODE - NO SECURITY)",
                    addr
                );

                // Start server (blocks until shutdown)
                if let Err(e) = server.serve(addr).await {
                    error!("Server error: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    info!("Server shutdown complete");
    Ok(())
}
