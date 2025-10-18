//! Production-grade TCP storage server using custom binary protocol
//! 
//! Replaces gRPC server while maintaining distributed architecture.
//! Runs as standalone service - API/Hybrid connect over network.

use crate::concurrent_memory::ConcurrentMemory;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal;

// Import protocol from sutra-protocol crate
// Note: In production, add sutra-protocol as dependency in Cargo.toml
use serde::{Deserialize, Serialize};

// Re-define protocol messages here for now (will use sutra-protocol crate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageRequest {
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
}

impl StorageServer {
    /// Create new storage server
    pub fn new(storage: ConcurrentMemory) -> Self {
        Self {
            storage: Arc::new(storage),
            start_time: std::time::Instant::now(),
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
                let uptime = self.start_time.elapsed().as_secs();

                StorageResponse::StatsOk {
                    concepts: stats.snapshot.concept_count as u64,
                    edges: stats.snapshot.edge_count as u64,
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
