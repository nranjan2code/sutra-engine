//! Production-grade TCP storage server using custom binary protocol
//! 
//! Replaces gRPC server while maintaining distributed architecture.
//! Runs as standalone service - API/Hybrid connect over network.

use crate::concurrent_memory::ConcurrentMemory;
use crate::sharded_storage::ShardedStorage;
use crate::learning_pipeline::{LearningPipeline, LearnOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal;

// Import protocol from sutra-protocol crate
// Note: In production, add sutra-protocol as dependency in Cargo.toml
use serde::{Deserialize, Serialize};

// 🔥 PRODUCTION: Security limits to prevent DoS attacks
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024; // 10MB max content
const MAX_EMBEDDING_DIM: usize = 2048; // Max embedding dimension
const MAX_BATCH_SIZE: usize = 1000; // Max batch size
const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024; // 100MB max TCP message
const MAX_PATH_DEPTH: u32 = 20; // Max path finding depth
const MAX_SEARCH_K: u32 = 1000; // Max k for vector search

// Re-define protocol messages here for now (will use sutra-protocol crate)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnOptionsMsg {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}

impl From<LearnOptionsMsg> for LearnOptions {
    fn from(m: LearnOptionsMsg) -> Self {
        LearnOptions {
            generate_embedding: m.generate_embedding,
            embedding_model: m.embedding_model,
            extract_associations: m.extract_associations,
            min_association_confidence: m.min_association_confidence,
            max_associations_per_concept: m.max_associations_per_concept,
            strength: m.strength,
            confidence: m.confidence,
        }
    }
}

impl Default for LearnOptionsMsg {
    fn default() -> Self {
        let d = LearnOptions::default();
        LearnOptionsMsg {
            generate_embedding: d.generate_embedding,
            embedding_model: d.embedding_model,
            extract_associations: d.extract_associations,
            min_association_confidence: d.min_association_confidence,
            max_associations_per_concept: d.max_associations_per_concept,
            strength: d.strength,
            confidence: d.confidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageRequest {
    // Legacy explicit learn (still supported)
    // New unified learning API (V2)
    LearnConceptV2 {
        content: String,
        options: LearnOptionsMsg,
    },
    LearnBatch {
        contents: Vec<String>,
        options: LearnOptionsMsg,
    },
    LearnConcept {
        concept_id: String,
        content: String,
        embedding: Vec<f32>,
        strength: f32,
        confidence: f32,
    },
    LearnAssociation {
        source_id: String,
        target_id: String,
        assoc_type: u32,
        confidence: f32,
    },
    QueryConcept {
        concept_id: String,
    },
    GetNeighbors {
        concept_id: String,
    },
    FindPath {
        start_id: String,
        end_id: String,
        max_depth: u32,
    },
    VectorSearch {
        query_vector: Vec<f32>,
        k: u32,
        ef_search: u32,
    },
    GetStats,
    Flush,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageResponse {
    LearnConceptV2Ok { concept_id: String },
    LearnBatchOk { concept_ids: Vec<String> },
    LearnConceptOk { sequence: u64 },
    LearnAssociationOk { sequence: u64 },
    QueryConceptOk {
        found: bool,
        concept_id: String,
        content: String,
        strength: f32,
        confidence: f32,
    },
    GetNeighborsOk { neighbor_ids: Vec<String> },
    FindPathOk { found: bool, path: Vec<String> },
    VectorSearchOk { results: Vec<(String, f32)> },
    StatsOk {
        concepts: u64,
        edges: u64,
        vectors: u64,
        written: u64,
        dropped: u64,
        pending: u64,
        reconciliations: u64,
        uptime_seconds: u64,
    },
    FlushOk,
    HealthCheckOk {
        healthy: bool,
        status: String,
        uptime_seconds: u64,
    },
    Error { message: String },
}

/// Storage server state
pub struct StorageServer {
    storage: Arc<ConcurrentMemory>,
    start_time: std::time::Instant,
    pipeline: LearningPipeline,
}

impl StorageServer {
    /// Create new storage server
    pub async fn new(storage: ConcurrentMemory) -> Self {
        let pipeline = LearningPipeline::new().await
            .expect("Failed to init learning pipeline");
        Self {
            storage: Arc::new(storage),
            start_time: std::time::Instant::now(),
            pipeline,
        }
    }

    /// Start TCP server
    pub async fn serve(self: Arc<Self>, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        eprintln!("Storage server listening on {}", addr);

        // Graceful shutdown handler
        let shutdown = signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            let server = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_client(stream, peer_addr).await {
                                    eprintln!("Client error ({}): {}", peer_addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
                _ = &mut shutdown => {
                    eprintln!("Shutdown signal received, flushing storage...");
                    if let Err(e) = self.storage.flush() {
                        eprintln!("Flush error: {:?}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle single client connection
    async fn handle_client(
        &self,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> std::io::Result<()> {
        eprintln!("Client connected: {}", peer_addr);

        // Configure for low latency
        stream.set_nodelay(true)?;

        loop {
            // Read message length (4 bytes)
            let len = match stream.read_u32().await {
                Ok(len) => len,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Client disconnected
                    break;
                }
                Err(e) => return Err(e),
            };

            // ✅ PRODUCTION: Validate message size before allocating
            if len as usize > MAX_MESSAGE_SIZE {
                let error = StorageResponse::Error {
                    message: format!("Message too large: {} bytes (max: {})", len, MAX_MESSAGE_SIZE),
                };
                let response_bytes = rmp_serde::to_vec(&error)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                stream.write_u32(response_bytes.len() as u32).await?;
                stream.write_all(&response_bytes).await?;
                stream.flush().await?;
                continue; // Skip this message but keep connection open
            }

            // Read message payload
            let mut buf = vec![0u8; len as usize];
            stream.read_exact(&mut buf).await?;

            // Deserialize request (msgpack for Python clients)
            let request: StorageRequest = rmp_serde::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Handle request
            let response = self.handle_request(request).await;

            // Serialize response (msgpack for Python clients)
            let response_bytes = rmp_serde::to_vec(&response)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Write response
            stream.write_u32(response_bytes.len() as u32).await?;
            stream.write_all(&response_bytes).await?;
            stream.flush().await?;
        }

        eprintln!("Client disconnected: {}", peer_addr);
        Ok(())
    }

/// Handle storage request
    async fn handle_request(&self, request: StorageRequest) -> StorageResponse {
        use crate::types::{ConceptId, AssociationType};

        match request {
            StorageRequest::LearnConceptV2 { content, options } => {
                // ✅ PRODUCTION: Validate content size
                if content.len() > MAX_CONTENT_SIZE {
                    return StorageResponse::Error {
                        message: format!("Content too large: {} bytes (max: {})", content.len(), MAX_CONTENT_SIZE),
                    };
                }
                
                match self.pipeline.learn_concept(&self.storage, &content, &options.into()).await {
                    Ok(concept_id) => StorageResponse::LearnConceptV2Ok { concept_id },
                    Err(e) => StorageResponse::Error { message: format!("LearnConceptV2 failed: {}", e) },
                }
            }
            StorageRequest::LearnBatch { contents, options } => {
                // ✅ PRODUCTION: Validate batch size
                if contents.len() > MAX_BATCH_SIZE {
                    return StorageResponse::Error {
                        message: format!("Batch too large: {} items (max: {})", contents.len(), MAX_BATCH_SIZE),
                    };
                }
                
                // ✅ PRODUCTION: Validate content size for each item
                for (i, content) in contents.iter().enumerate() {
                    if content.len() > MAX_CONTENT_SIZE {
                        return StorageResponse::Error {
                            message: format!("Batch item {} too large: {} bytes (max: {})", i, content.len(), MAX_CONTENT_SIZE),
                        };
                    }
                }
                
                match self.pipeline.learn_batch(&self.storage, &contents, &options.into()).await {
                    Ok(concept_ids) => StorageResponse::LearnBatchOk { concept_ids },
                    Err(e) => StorageResponse::Error { message: format!("LearnBatch failed: {}", e) },
                }
            }
            StorageRequest::LearnConcept {
                concept_id,
                content,
                embedding,
                strength,
                confidence,
            } => {
                // ✅ PRODUCTION: Validate content size
                if content.len() > MAX_CONTENT_SIZE {
                    return StorageResponse::Error {
                        message: format!("Content too large: {} bytes (max: {})", content.len(), MAX_CONTENT_SIZE),
                    };
                }
                
                // ✅ PRODUCTION: Validate embedding dimension
                if !embedding.is_empty() && embedding.len() > MAX_EMBEDDING_DIM {
                    return StorageResponse::Error {
                        message: format!("Embedding dimension too large: {} (max: {})", embedding.len(), MAX_EMBEDDING_DIM),
                    };
                }
                
                let id = ConceptId::from_string(&concept_id);
                let content_bytes = content.into_bytes();
                let vector = if embedding.is_empty() { None } else { Some(embedding) };

                match self.storage.learn_concept(id, content_bytes, vector, strength, confidence) {
                    Ok(sequence) => StorageResponse::LearnConceptOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn concept failed: {:?}", e),
                    },
                }
            }

            StorageRequest::LearnAssociation {
                source_id,
                target_id,
                assoc_type,
                confidence,
            } => {
                let source = ConceptId::from_string(&source_id);
                let target = ConceptId::from_string(&target_id);
                let atype = AssociationType::from_u8(assoc_type as u8)
                    .unwrap_or(AssociationType::Semantic);

                match self.storage.learn_association(source, target, atype, confidence) {
                    Ok(sequence) => StorageResponse::LearnAssociationOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn association failed: {:?}", e),
                    },
                }
            }

            StorageRequest::QueryConcept { concept_id } => {
                let id = ConceptId::from_string(&concept_id);

                if let Some(node) = self.storage.query_concept(&id) {
                    StorageResponse::QueryConceptOk {
                        found: true,
                        concept_id: id.to_hex(),
                        content: String::from_utf8_lossy(&node.content).to_string(),
                        strength: node.strength,
                        confidence: node.confidence,
                    }
                } else {
                    StorageResponse::QueryConceptOk {
                        found: false,
                        concept_id: String::new(),
                        content: String::new(),
                        strength: 0.0,
                        confidence: 0.0,
                    }
                }
            }

            StorageRequest::GetNeighbors { concept_id } => {
                let id = ConceptId::from_string(&concept_id);
                let neighbors = self.storage.query_neighbors(&id);
                let neighbor_ids = neighbors.iter().map(|id| id.to_hex()).collect();

                StorageResponse::GetNeighborsOk { neighbor_ids }
            }

            StorageRequest::FindPath {
                start_id,
                end_id,
                max_depth,
            } => {
                // ✅ PRODUCTION: Validate path depth to prevent expensive queries
                if max_depth > MAX_PATH_DEPTH {
                    return StorageResponse::Error {
                        message: format!("Path depth too large: {} (max: {})", max_depth, MAX_PATH_DEPTH),
                    };
                }
                
                let start = ConceptId::from_string(&start_id);
                let end = ConceptId::from_string(&end_id);

                if let Some(path) = self.storage.find_path(start, end, max_depth as usize) {
                    let path_ids = path.iter().map(|id| id.to_hex()).collect();
                    StorageResponse::FindPathOk {
                        found: true,
                        path: path_ids,
                    }
                } else {
                    StorageResponse::FindPathOk {
                        found: false,
                        path: vec![],
                    }
                }
            }

            StorageRequest::VectorSearch {
                query_vector,
                k,
                ef_search,
            } => {
                // ✅ PRODUCTION: Validate query vector dimension
                if query_vector.len() > MAX_EMBEDDING_DIM {
                    return StorageResponse::Error {
                        message: format!("Query vector dimension too large: {} (max: {})", query_vector.len(), MAX_EMBEDDING_DIM),
                    };
                }
                
                // ✅ PRODUCTION: Validate k parameter
                if k > MAX_SEARCH_K {
                    return StorageResponse::Error {
                        message: format!("k too large: {} (max: {})", k, MAX_SEARCH_K),
                    };
                }
                
                let results = self
                    .storage
                    .vector_search(&query_vector, k as usize, ef_search as usize);
                let results_vec = results
                    .into_iter()
                    .map(|(id, sim)| (id.to_hex(), sim))
                    .collect();

                StorageResponse::VectorSearchOk { results: results_vec }
            }

            StorageRequest::GetStats => {
                let stats = self.storage.stats();
                let hnsw_stats = self.storage.hnsw_stats();
                let uptime = self.start_time.elapsed().as_secs();

                StorageResponse::StatsOk {
                    concepts: stats.snapshot.concept_count as u64,
                    edges: stats.snapshot.edge_count as u64,
                    vectors: hnsw_stats.indexed_vectors as u64,
                    written: stats.write_log.written,
                    dropped: stats.write_log.dropped,
                    pending: stats.write_log.pending as u64,
                    reconciliations: stats.reconciler.reconciliations,
                    uptime_seconds: uptime,
                }
            }

            StorageRequest::Flush => match self.storage.flush() {
                Ok(_) => StorageResponse::FlushOk,
                Err(e) => StorageResponse::Error {
                    message: format!("Flush failed: {:?}", e),
                },
            },

            StorageRequest::HealthCheck => {
                let uptime = self.start_time.elapsed().as_secs();
                StorageResponse::HealthCheckOk {
                    healthy: true,
                    status: "ok".to_string(),
                    uptime_seconds: uptime,
                }
            }
        }
    }
}

/// Sharded Storage Server - wraps ShardedStorage with TCP protocol
pub struct ShardedStorageServer {
    storage: Arc<ShardedStorage>,
    start_time: std::time::Instant,
    pipeline: LearningPipeline,
}

impl ShardedStorageServer {
    /// Create new sharded storage server
    pub async fn new(storage: ShardedStorage) -> Self {
        let pipeline = LearningPipeline::new().await
            .expect("Failed to init learning pipeline");
        Self {
            storage: Arc::new(storage),
            start_time: std::time::Instant::now(),
            pipeline,
        }
    }

    /// Start TCP server (same interface as StorageServer)
    pub async fn serve(self: Arc<Self>, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        eprintln!("Sharded storage server listening on {}", addr);

        // Graceful shutdown handler
        let shutdown = signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            let server = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_client(stream, peer_addr).await {
                                    eprintln!("Client error ({}): {}", peer_addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
                _ = &mut shutdown => {
                    eprintln!("Shutdown signal received, flushing sharded storage...");
                    if let Err(e) = self.storage.flush() {
                        eprintln!("Flush error: {:?}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle single client connection
    async fn handle_client(
        &self,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> std::io::Result<()> {
        eprintln!("Client connected: {}", peer_addr);

        // Configure for low latency
        stream.set_nodelay(true)?;

        loop {
            // Read message length (4 bytes)
            let len = match stream.read_u32().await {
                Ok(len) => len,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Client disconnected
                    break;
                }
                Err(e) => return Err(e),
            };

            // Read message payload
            let mut buf = vec![0u8; len as usize];
            stream.read_exact(&mut buf).await?;

            // Deserialize request (msgpack for Python clients)
            let request: StorageRequest = rmp_serde::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Handle request
            let response = self.handle_request(request).await;

            // Serialize response (msgpack for Python clients)
            let response_bytes = rmp_serde::to_vec(&response)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Write response
            stream.write_u32(response_bytes.len() as u32).await?;
            stream.write_all(&response_bytes).await?;
            stream.flush().await?;
        }

        eprintln!("Client disconnected: {}", peer_addr);
        Ok(())
    }

    /// Handle storage request (sharded version)
    async fn handle_request(&self, request: StorageRequest) -> StorageResponse {
        use crate::types::{ConceptId, AssociationType};

        match request {
            StorageRequest::LearnConceptV2 { content, options } => {
                // Convert LearnOptionsMsg to LearnOptions
                let learn_opts: LearnOptions = options.into();
                
                // Use learning pipeline (works with LearningStorage trait)
                match self.pipeline.learn_concept(&self.storage, &content, &learn_opts).await {
                    Ok(concept_id) => StorageResponse::LearnConceptV2Ok { concept_id },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learning pipeline failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnBatch { contents, options } => {
                // Convert LearnOptionsMsg to LearnOptions
                let learn_opts: LearnOptions = options.into();
                
                // Use learning pipeline batch method
                match self.pipeline.learn_batch(&self.storage, &contents, &learn_opts).await {
                    Ok(concept_ids) => StorageResponse::LearnBatchOk { concept_ids },
                    Err(e) => StorageResponse::Error {
                        message: format!("Batch learning failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnConcept {
                concept_id,
                content,
                embedding,
                strength,
                confidence,
            } => {
                let id = ConceptId::from_string(&concept_id);
                let content_bytes = content.into_bytes();
                let vector = if embedding.is_empty() { None } else { Some(embedding) };

                match self.storage.learn_concept(id, content_bytes, vector, strength, confidence) {
                    Ok(sequence) => StorageResponse::LearnConceptOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn concept failed: {:?}", e),
                    },
                }
            }

            StorageRequest::LearnAssociation {
                source_id,
                target_id,
                assoc_type,
                confidence,
            } => {
                let source = ConceptId::from_string(&source_id);
                let target = ConceptId::from_string(&target_id);
                let atype = AssociationType::from_u8(assoc_type as u8)
                    .unwrap_or(AssociationType::Semantic);

                match self.storage.create_association(source, target, atype, confidence) {
                    Ok(sequence) => StorageResponse::LearnAssociationOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn association failed: {:?}", e),
                    },
                }
            }

            StorageRequest::QueryConcept { concept_id } => {
                let id = ConceptId::from_string(&concept_id);

                if let Some(node) = self.storage.get_concept(id) {
                    StorageResponse::QueryConceptOk {
                        found: true,
                        concept_id: id.to_hex(),
                        content: String::from_utf8_lossy(&node.content).to_string(),
                        strength: node.strength,
                        confidence: node.confidence,
                    }
                } else {
                    StorageResponse::QueryConceptOk {
                        found: false,
                        concept_id: String::new(),
                        content: String::new(),
                        strength: 0.0,
                        confidence: 0.0,
                    }
                }
            }

            StorageRequest::GetNeighbors { concept_id } => {
                let id = ConceptId::from_string(&concept_id);
                let neighbors = self.storage.get_neighbors(id);
                let neighbor_ids = neighbors.iter().map(|id| id.to_hex()).collect();

                StorageResponse::GetNeighborsOk { neighbor_ids }
            }

            StorageRequest::FindPath {
                start_id,
                end_id,
                max_depth,
            } => {
                // Cross-shard path finding would require distributed BFS
                // For now, return error with guidance
                StorageResponse::Error {
                    message: format!(
                        "FindPath not yet implemented for sharded storage (start: {}, end: {}, depth: {}). \
                         Cross-shard path finding requires distributed BFS.",
                        start_id, end_id, max_depth
                    ),
                }
            }

            StorageRequest::VectorSearch {
                query_vector,
                k,
                ef_search: _,
            } => {
                // Parallel semantic search across all shards
                let results = self.storage.semantic_search(query_vector, k as usize);
                let results_vec = results
                    .into_iter()
                    .map(|(id, sim)| (id.to_hex(), sim))
                    .collect();

                StorageResponse::VectorSearchOk { results: results_vec }
            }

            StorageRequest::GetStats => {
                let stats = self.storage.stats();
                let uptime = self.start_time.elapsed().as_secs();

                StorageResponse::StatsOk {
                    concepts: stats.total_concepts as u64,
                    edges: stats.total_edges as u64,
                    vectors: stats.total_vectors as u64,
                    written: stats.total_writes,
                    dropped: 0, // TODO: aggregate from shards
                    pending: 0, // TODO: aggregate from shards
                    reconciliations: 0, // TODO: aggregate from shards
                    uptime_seconds: uptime,
                }
            }

            StorageRequest::Flush => match self.storage.flush() {
                Ok(_) => StorageResponse::FlushOk,
                Err(e) => StorageResponse::Error {
                    message: format!("Flush failed: {:?}", e),
                },
            },

            StorageRequest::HealthCheck => {
                let uptime = self.start_time.elapsed().as_secs();
                let stats = self.storage.stats();
                StorageResponse::HealthCheckOk {
                    healthy: true,
                    status: format!("sharded ({} shards, {} concepts)", stats.num_shards, stats.total_concepts),
                    uptime_seconds: uptime,
                }
            }
        }
    }
}
