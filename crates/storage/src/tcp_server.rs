//! Production-grade TCP storage server using custom binary protocol
//!
//! Replaces gRPC server while maintaining distributed architecture.
//! Runs as standalone service - API/Hybrid connect over network.

use crate::concurrent_memory::ConcurrentMemory;
use crate::learning_pipeline::{LearnOptions, LearningPipeline};
use crate::namespace_manager::NamespaceManager;
use crate::nl_parser::NlParser; // 🔥 NEW
use crate::semantic::{CausalType, DomainContext, SemanticType};
use crate::sharded_storage::ShardedStorage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}; // BufRead for lines
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tracing::info;

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

// 🔥 NEW: Semantic filter for TCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFilterMsg {
    pub semantic_type: Option<String>, // "rule", "event", "entity", etc.
    pub domain_context: Option<String>, // "medical", "legal", "financial", etc.
    pub temporal_after: Option<i64>,   // Unix timestamp
    pub temporal_before: Option<i64>,  // Unix timestamp
    pub has_causal_relation: bool,
    pub min_confidence: f32,
    pub required_terms: Vec<String>,
}

impl Default for SemanticFilterMsg {
    fn default() -> Self {
        Self {
            semantic_type: None,
            domain_context: None,
            temporal_after: None,
            temporal_before: None,
            has_causal_relation: false,
            min_confidence: 0.0,
            required_terms: Vec::new(),
        }
    }
}

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
            analyze_semantics: std::env::var("SUTRA_SEMANTIC_ANALYSIS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
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
            // analyze_semantics is always true, not exposed in message (internal only)
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
    /// Learn a concept with optional automatic embedding generation
    LearnConceptV2 {
        namespace: Option<String>,
        content: String,
        options: LearnOptionsMsg,
    },
    /// Learn a batch of concepts
    LearnBatch {
        namespace: Option<String>,
        contents: Vec<String>,
        options: LearnOptionsMsg,
    },
    /// 🔥 NEW: Learn with precomputed embedding (Requested for Sutra)
    LearnWithEmbedding {
        id: Option<String>,
        namespace: String,
        content: String,
        embedding: Vec<f32>,
        metadata: std::collections::HashMap<String, String>,
        timestamp: Option<i64>,
    },
    LearnConcept {
        namespace: Option<String>,
        concept_id: String,
        content: String,
        embedding: Vec<f32>,
        strength: f32,
        confidence: f32,
    },
    /// Learn association between concepts
    LearnAssociation {
        namespace: Option<String>,
        source_id: String,
        target_id: String,
        assoc_type: u32,
        confidence: f32,
    },
    /// Get concept by ID
    QueryConcept {
        namespace: Option<String>,
        concept_id: String,
    },
    /// 🔥 NEW: Delete concept by ID (Requested for Sutra)
    DeleteConcept {
        namespace: String,
        id: String,
    },
    /// 🔥 NEW: Clear an entire collection (Requested for Sutra)
    ClearCollection {
        namespace: String,
    },
    /// Get neighbor IDs
    GetNeighbors {
        namespace: Option<String>,
        concept_id: String,
    },
    FindPath {
        namespace: Option<String>,
        start_id: String,
        end_id: String,
        max_depth: u32,
    },
    VectorSearch {
        namespace: Option<String>,
        query_vector: Vec<f32>,
        k: u32,
        ef_search: u32,
    },
    /// 🔥 NEW: List recent items without vector search (Requested for Sutra)
    ListRecent {
        namespace: String,
        limit: u32,
    },
    // 🔥 NEW: Semantic query operations
    FindPathSemantic {
        namespace: Option<String>,
        start_id: String,
        end_id: String,
        filter: SemanticFilterMsg,
        max_depth: u32,
        max_paths: u32,
    },
    FindTemporalChain {
        namespace: Option<String>,
        domain: Option<String>, // "medical", "legal", etc.
        start_time: i64,
        end_time: i64,
    },
    FindCausalChain {
        namespace: Option<String>,
        start_id: String,
        causal_type: String, // "direct", "indirect", "enabling", etc.
        max_depth: u32,
    },
    FindContradictions {
        namespace: Option<String>,
        domain: String,
    },
    QueryBySemantic {
        namespace: Option<String>,
        filter: SemanticFilterMsg,
        limit: Option<usize>,
    },
    TextSearch {
        namespace: Option<String>,
        query: String,
        limit: u32,
    },
    GetStats {
        namespace: Option<String>,
    },
    Flush,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticPathMsg {
    pub concepts: Vec<String>,
    pub confidence: f32,
    pub type_distribution: std::collections::HashMap<String, usize>,
    pub domains: Vec<String>,
    pub is_temporally_ordered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptWithSemanticMsg {
    pub concept_id: String,
    pub content: String,
    pub semantic_type: String,
    pub domain: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageResponse {
    LearnConceptV2Ok {
        concept_id: String,
    },
    LearnBatchOk {
        concept_ids: Vec<String>,
    },
    LearnConceptOk {
        sequence: u64,
    },
    LearnAssociationOk {
        sequence: u64,
    },
    DeleteConceptOk {
        id: String,
    },
    ClearCollectionOk {
        namespace: String,
    },
    QueryConceptOk {
        found: bool,
        concept_id: String,
        content: String,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    },
    GetNeighborsOk {
        neighbor_ids: Vec<String>,
    },
    FindPathOk {
        found: bool,
        path: Vec<String>,
    },
    VectorSearchOk {
        results: Vec<(String, f32)>,
    },
    ListRecentOk {
        items: Vec<RecentItemMsg>,
    },
    // 🔥 NEW: Semantic query responses
    FindPathSemanticOk {
        paths: Vec<SemanticPathMsg>,
    },
    FindTemporalChainOk {
        paths: Vec<SemanticPathMsg>,
    },
    FindCausalChainOk {
        paths: Vec<SemanticPathMsg>,
    },
    FindContradictionsOk {
        contradictions: Vec<(String, String, String)>, // (id1, id2, reason)
    },
    QueryBySemanticOk {
        concepts: Vec<ConceptWithSemanticMsg>,
    },
    TextSearchOk {
        results: Vec<(String, f32)>, // (concept_id, score)
    },
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
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItemMsg {
    pub id: String,
    pub content_preview: String,
    pub created: u64,
    pub attributes: std::collections::HashMap<String, String>,
}

/// Storage server state
pub struct StorageServer {
    namespaces: Arc<NamespaceManager>,
    start_time: std::time::Instant,
    pipeline: LearningPipeline,
}

impl StorageServer {
    /// Create new storage server
    pub async fn new(storage: ConcurrentMemory) -> Self {
        let config = storage.config().clone();
        let base_path = config
            .storage_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        // Fix: correctly pass 2 args to NamespaceManager::new
        let manager = NamespaceManager::new(base_path, config.clone())
            .expect("Failed to init namespace manager");

        // Wrap existing storage into "default" namespace
        manager.add_namespace("default", Arc::new(storage));

        let pipeline = LearningPipeline::new()
            .await
            .expect("Failed to init learning pipeline");

        Self {
            namespaces: Arc::new(manager),
            start_time: std::time::Instant::now(),
            pipeline,
        }
    }

    /// Create new storage server with a pre-built pipeline (for tests or custom providers)
    pub fn new_with_pipeline(storage: ConcurrentMemory, pipeline: LearningPipeline) -> Self {
        let config = storage.config().clone();
        let base_path = config
            .storage_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        let manager = NamespaceManager::new(base_path, config.clone())
            .expect("Failed to init namespace manager");

        manager.add_namespace("default", Arc::new(storage));

        Self {
            namespaces: Arc::new(manager),
            start_time: std::time::Instant::now(),
            pipeline,
        }
    }

    /// Get storage for a namespace (falls back to "default")
    fn get_storage(&self, ns: Option<String>) -> Arc<ConcurrentMemory> {
        self.namespaces
            .get_namespace(ns.as_deref().unwrap_or("default"))
    }

    /// Start TCP server
    pub async fn serve(self: Arc<Self>, addr: SocketAddr) -> std::io::Result<()> {
        self.serve_with_shutdown(addr, async {
            let _ = signal::ctrl_c().await;
        })
        .await
    }

    /// Start TCP server with a custom shutdown signal (test-friendly)
    pub async fn serve_with_shutdown<F>(
        self: Arc<Self>,
        addr: SocketAddr,
        shutdown: F,
    ) -> std::io::Result<()>
    where
        F: std::future::Future<Output = ()> + Send,
    {
        let listener = TcpListener::bind(addr).await?;
        eprintln!("Storage server listening on {}", addr);

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
                    if let Err(e) = self.namespaces.flush_all() {
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
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> std::io::Result<()> {
        eprintln!("Client connected: {}", peer_addr);

        // Configure for low latency and better throughput
        stream.set_nodelay(true)?;

        let mut request_count = 0u64;

        // Wrap stream in BufReader for line-based reading support
        let mut reader = BufReader::new(stream);

        loop {
            let _request_start = std::time::Instant::now();

            // Peek first byte to sniff protocol
            let start_byte = match reader.fill_buf().await {
                Ok(buf) => {
                    if buf.is_empty() {
                         // Client disconnected
                         if request_count > 0 {
                             eprintln!("Client {} disconnected", peer_addr);
                         }
                         break;
                    }
                    buf[0]
                }
                Err(e) => return Err(e),
            };

            if start_byte == 0 {
                // BINARY PROTOCOL (Length Prefixed)
                // First byte is 0, so likely a u32 length < 16MB (00 xx xx xx)
                let len = match reader.read_u32().await {
                   Ok(l) => l,
                   Err(_) => break,
                };

                if len as usize > MAX_MESSAGE_SIZE {
                    let error = StorageResponse::Error {
                        message: format!("Message too large: {}", len),
                    };
                    let response_bytes = rmp_serde::to_vec_named(&error).unwrap();
                    reader.write_u32(response_bytes.len() as u32).await?;
                    reader.write_all(&response_bytes).await?;
                    reader.flush().await?;
                    continue;
                }

                let mut buf = vec![0u8; len as usize];
                reader.read_exact(&mut buf).await?;

                let request: StorageRequest = match rmp_serde::from_slice(&buf) {
                    Ok(req) => req,
                    Err(e) => {
                         eprintln!("Deserialization error: {}", e);
                         continue;
                    }
                };

                let response = self.handle_request(request).await;
                
                let response_bytes = rmp_serde::to_vec_named(&response).unwrap();
                reader.write_u32(response_bytes.len() as u32).await?;
                reader.write_all(&response_bytes).await?;
                reader.flush().await?;

            } else {
                // TEXT PROTOCOL (Natural Language)
                // First byte is NOT 0, assume it is ASCII text command
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let line = line.trim();
                        if !line.is_empty() {
                            info!("🗣️ NL Command: '{}'", line);
                            if let Some(req) = NlParser::parse(line) {
                                let response = self.handle_request(req).await;
                                
                                // Serialize response as JSON/Text for the human
                                let json = serde_json::to_string_pretty(&response).unwrap_or_default();
                                reader.write_all(json.as_bytes()).await?;
                                reader.write_all(b"\n").await?;
                                reader.flush().await?;
                            } else {
                                reader.write_all(b"Error: Command not understood. Try 'Remember that X', 'Find Y', or 'List'.\n").await?;
                                reader.flush().await?;
                            }
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            
            request_count += 1;
        }

        eprintln!("Client disconnected: {}", peer_addr);
        Ok(())
    }

    /// Handle storage request
    pub async fn handle_request(&self, request: StorageRequest) -> StorageResponse {
        use crate::types::{AssociationType, ConceptId};

        match request {
            StorageRequest::LearnConceptV2 {
                namespace,
                content,
                options,
            } => {
                let storage = self.get_storage(namespace);
                // ✅ PRODUCTION: Validate content size
                if content.len() > MAX_CONTENT_SIZE {
                    return StorageResponse::Error {
                        message: format!(
                            "Content too large: {} bytes (max: {})",
                            content.len(),
                            MAX_CONTENT_SIZE
                        ),
                    };
                }

                match self
                    .pipeline
                    .learn_concept(&storage, &content, &options.into())
                    .await
                {
                    Ok(concept_id) => StorageResponse::LearnConceptV2Ok { concept_id },
                    Err(e) => StorageResponse::Error {
                        message: format!("LearnConceptV2 failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnBatch {
                namespace,
                contents,
                options,
            } => {
                let storage = self.get_storage(namespace);
                // ✅ PRODUCTION: Validate batch size
                if contents.len() > MAX_BATCH_SIZE {
                    return StorageResponse::Error {
                        message: format!(
                            "Batch too large: {} items (max: {})",
                            contents.len(),
                            MAX_BATCH_SIZE
                        ),
                    };
                }

                // ✅ PRODUCTION: Validate content size for each item
                for (i, content) in contents.iter().enumerate() {
                    if content.len() > MAX_CONTENT_SIZE {
                        return StorageResponse::Error {
                            message: format!(
                                "Batch item {} too large: {} bytes (max: {})",
                                i,
                                content.len(),
                                MAX_CONTENT_SIZE
                            ),
                        };
                    }
                }

                match self
                    .pipeline
                    .learn_batch(&storage, &contents, &options.into())
                    .await
                {
                    Ok(concept_ids) => StorageResponse::LearnBatchOk { concept_ids },
                    Err(e) => StorageResponse::Error {
                        message: format!("LearnBatch failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnWithEmbedding {
                id,
                namespace,
                content,
                embedding,
                metadata,
                timestamp: _,
            } => {
                let storage = self.get_storage(Some(namespace));
                let concept_id = id
                    .map(|s| ConceptId::from_string(&s))
                    .unwrap_or_else(|| ConceptId::from_string(&content));

                match storage.learn_concept(
                    concept_id,
                    content.into_bytes(),
                    Some(embedding),
                    1.0,
                    1.0,
                    metadata,
                ) {
                    Ok(_) => StorageResponse::LearnConceptV2Ok {
                        concept_id: concept_id.to_hex(),
                    },
                    Err(e) => StorageResponse::Error {
                        message: format!("LearnWithEmbedding failed: {:?}", e),
                    },
                }
            }
            StorageRequest::LearnConcept {
                namespace,
                concept_id,
                content,
                embedding,
                strength,
                confidence,
            } => {
                let storage = self.get_storage(namespace);
                // ✅ PRODUCTION: Validate content size
                if content.len() > MAX_CONTENT_SIZE {
                    return StorageResponse::Error {
                        message: format!(
                            "Content too large: {} bytes (max: {})",
                            content.len(),
                            MAX_CONTENT_SIZE
                        ),
                    };
                }

                // ✅ PRODUCTION: Validate embedding dimension
                if !embedding.is_empty() && embedding.len() > MAX_EMBEDDING_DIM {
                    return StorageResponse::Error {
                        message: format!(
                            "Embedding dimension too large: {} (max: {})",
                            embedding.len(),
                            MAX_EMBEDDING_DIM
                        ),
                    };
                }

                let id = ConceptId::from_string(&concept_id);
                let content_bytes = content.into_bytes();
                let vector = if embedding.is_empty() {
                    None
                } else {
                    Some(embedding)
                };

                match storage.learn_concept(
                    id,
                    content_bytes,
                    vector,
                    strength,
                    confidence,
                    std::collections::HashMap::new(),
                ) {
                    Ok(sequence) => StorageResponse::LearnConceptOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn concept failed: {:?}", e),
                    },
                }
            }

            StorageRequest::LearnAssociation {
                namespace,
                source_id,
                target_id,
                assoc_type,
                confidence,
            } => {
                let storage = self.get_storage(namespace);
                let source = ConceptId::from_string(&source_id);
                let target = ConceptId::from_string(&target_id);
                let atype =
                    AssociationType::from_u8(assoc_type as u8).unwrap_or(AssociationType::Semantic);

                match storage.learn_association(source, target, atype, confidence) {
                    Ok(sequence) => StorageResponse::LearnAssociationOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn association failed: {:?}", e),
                    },
                }
            }

            StorageRequest::QueryConcept {
                namespace,
                concept_id,
            } => {
                let storage = self.get_storage(namespace);
                let id = ConceptId::from_string(&concept_id);

                if let Some(node) = storage.query_concept(&id) {
                    StorageResponse::QueryConceptOk {
                        found: true,
                        concept_id: id.to_hex(),
                        content: String::from_utf8_lossy(&node.content).to_string(),
                        strength: node.strength,
                        confidence: node.confidence,
                        attributes: node.attributes.clone(),
                    }
                } else {
                    StorageResponse::QueryConceptOk {
                        found: false,
                        concept_id: String::new(),
                        content: String::new(),
                        strength: 0.0,
                        confidence: 0.0,
                        attributes: std::collections::HashMap::new(),
                    }
                }
            }

            StorageRequest::DeleteConcept { namespace, id } => {
                let storage = self.get_storage(Some(namespace));
                let concept_id = ConceptId::from_string(&id);
                match storage.delete_concept(concept_id) {
                    Ok(_) => StorageResponse::DeleteConceptOk { id: id.to_string() },
                    Err(e) => StorageResponse::Error {
                        message: format!("Delete failed: {:?}", e),
                    },
                }
            }

            StorageRequest::ClearCollection { namespace } => {
                let storage = self.get_storage(Some(namespace.clone()));
                match storage.clear() {
                    Ok(_) => StorageResponse::ClearCollectionOk {
                        namespace: namespace.to_string(),
                    },
                    Err(e) => StorageResponse::Error {
                        message: format!("Clear failed: {:?}", e),
                    },
                }
            }

            StorageRequest::GetNeighbors {
                namespace,
                concept_id,
            } => {
                let storage = self.get_storage(namespace);
                let id = ConceptId::from_string(&concept_id);
                let neighbors = storage.query_neighbors(&id);
                let neighbor_ids = neighbors.iter().map(|id: &ConceptId| id.to_hex()).collect();

                StorageResponse::GetNeighborsOk { neighbor_ids }
            }

            StorageRequest::FindPath {
                namespace,
                start_id,
                end_id,
                max_depth,
            } => {
                let storage = self.get_storage(namespace);
                // ✅ PRODUCTION: Validate path depth to prevent expensive queries
                if max_depth > MAX_PATH_DEPTH {
                    return StorageResponse::Error {
                        message: format!(
                            "Path depth too large: {} (max: {})",
                            max_depth, MAX_PATH_DEPTH
                        ),
                    };
                }

                let start = ConceptId::from_string(&start_id);
                let end = ConceptId::from_string(&end_id);

                if let Some(path) = storage.find_path(start, end, max_depth as usize) {
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
                namespace,
                query_vector,
                k,
                ef_search,
            } => {
                let storage = self.get_storage(namespace);
                // ✅ PRODUCTION: Validate query vector dimension
                if query_vector.len() > MAX_EMBEDDING_DIM {
                    return StorageResponse::Error {
                        message: format!(
                            "Query vector dimension too large: {} (max: {})",
                            query_vector.len(),
                            MAX_EMBEDDING_DIM
                        ),
                    };
                }

                // ✅ PRODUCTION: Validate k parameter
                if k > MAX_SEARCH_K {
                    return StorageResponse::Error {
                        message: format!("k too large: {} (max: {})", k, MAX_SEARCH_K),
                    };
                }

                let results = storage.vector_search(&query_vector, k as usize, ef_search as usize);
                let results_vec = results
                    .into_iter()
                    .map(|(id, sim)| (id.to_hex(), sim))
                    .collect();

                StorageResponse::VectorSearchOk {
                    results: results_vec,
                }
            }

            StorageRequest::ListRecent { namespace, limit } => {
                let storage = self.get_storage(Some(namespace));
                let snapshot = storage.get_snapshot();
                let mut items: Vec<RecentItemMsg> = snapshot
                    .concepts
                    .values()
                    .map(|node| RecentItemMsg {
                        id: node.id.to_hex(),
                        content_preview: String::from_utf8_lossy(&node.content)
                            .chars()
                            .take(200)
                            .collect(),
                        created: node.created,
                        attributes: node.attributes.clone(),
                    })
                    .collect();

                // Sort by created timestamp descending
                items.sort_by(|a, b| b.created.cmp(&a.created));
                items.truncate(limit as usize);

                StorageResponse::ListRecentOk { items }
            }

            StorageRequest::GetStats { namespace } => {
                let storage = self.get_storage(namespace);
                let stats = storage.stats();
                let hnsw_stats = storage.hnsw_stats();
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

            StorageRequest::Flush => match self.namespaces.flush_all() {
                Ok(_) => StorageResponse::FlushOk,
                Err(e) => StorageResponse::Error {
                    message: format!("Flush failed: {:?}", e),
                },
            },

            StorageRequest::HealthCheck => {
                let uptime = self.start_time.elapsed().as_secs();
                StorageResponse::HealthCheckOk {
                    healthy: true,
                    status: format!(
                        "Multi-namespace active, namespaces: {}",
                        self.namespaces.list_namespaces().len()
                    ),
                    uptime_seconds: uptime,
                }
            }

            // 🔥 NEW: Semantic query handlers
            StorageRequest::FindPathSemantic {
                namespace,
                start_id,
                end_id,
                filter,
                max_depth,
                max_paths,
            } => self.handle_find_path_semantic(
                namespace, start_id, end_id, filter, max_depth, max_paths,
            ),

            StorageRequest::FindTemporalChain {
                namespace,
                domain,
                start_time,
                end_time,
            } => self.handle_find_temporal_chain(namespace, domain, start_time, end_time),

            StorageRequest::FindCausalChain {
                namespace,
                start_id,
                causal_type,
                max_depth,
            } => self.handle_find_causal_chain(namespace, start_id, causal_type, max_depth),

            StorageRequest::FindContradictions { namespace, domain } => {
                self.handle_find_contradictions(namespace, domain)
            }

            StorageRequest::QueryBySemantic {
                namespace,
                filter,
                limit,
            } => self.handle_query_by_semantic(namespace, filter, limit),
            StorageRequest::TextSearch {
                namespace,
                query,
                limit,
            } => {
                let storage = self.get_storage(namespace);
                match self.pipeline.search(&storage, &query, limit as usize).await {
                    Ok(results) => StorageResponse::TextSearchOk {
                        results: results
                            .into_iter()
                            .map(|(id, score)| (id.to_hex(), score))
                            .collect(),
                    },
                    Err(e) => StorageResponse::Error {
                        message: format!("TextSearch failed: {}", e),
                    },
                }
            }
        }
    }

    // 🔥 NEW: Semantic query implementation methods
    fn handle_find_path_semantic(
        &self,
        namespace: Option<String>,
        start_id: String,
        end_id: String,
        filter_msg: SemanticFilterMsg,
        max_depth: u32,
        max_paths: u32,
    ) -> StorageResponse {
        let storage = self.get_storage(namespace);
        use crate::semantic::{
            CausalFilter, SemanticFilter, SemanticPathFinder, TemporalConstraint,
        };

        let start = ConceptId::from_string(&start_id);
        let end = ConceptId::from_string(&end_id);

        // Convert message filter to internal filter
        let mut filter = SemanticFilter::new();

        if let Some(ref st) = filter_msg.semantic_type {
            if let Some(semantic_type) = parse_semantic_type(st) {
                filter = filter.with_type(semantic_type);
            }
        }

        if let Some(ref dc) = filter_msg.domain_context {
            if let Some(domain) = parse_domain_context(dc) {
                filter = filter.with_domain(domain);
            }
        }

        if let Some(after) = filter_msg.temporal_after {
            filter = filter.with_temporal(TemporalConstraint::After(after));
        }

        if let Some(before) = filter_msg.temporal_before {
            filter = filter.with_temporal(TemporalConstraint::Before(before));
        }

        if filter_msg.has_causal_relation {
            filter = filter.with_causal(CausalFilter::HasCausalRelation);
        }

        filter = filter.with_min_confidence(filter_msg.min_confidence);

        for term in filter_msg.required_terms {
            filter = filter.with_term(term);
        }

        // Create pathfinder
        let pathfinder = SemanticPathFinder::new(max_depth as usize, max_paths as usize);
        let snapshot = storage.get_snapshot();

        // Find paths
        let paths = pathfinder.find_paths_filtered(snapshot, start, end, &filter);

        // Convert to message format
        let path_msgs: Vec<SemanticPathMsg> = paths
            .into_iter()
            .map(|p| SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p
                    .type_distribution
                    .into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p
                    .domains
                    .into_iter()
                    .map(|d| d.as_str().to_string())
                    .collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            })
            .collect();

        StorageResponse::FindPathSemanticOk { paths: path_msgs }
    }

    fn handle_find_temporal_chain(
        &self,
        namespace: Option<String>,
        domain: Option<String>,
        start_time: i64,
        end_time: i64,
    ) -> StorageResponse {
        let storage = self.get_storage(namespace);
        use crate::semantic::SemanticPathFinder;

        let domain_ctx = domain.and_then(|d| parse_domain_context(&d));
        let pathfinder = SemanticPathFinder::default();
        let snapshot = storage.get_snapshot();

        let paths = pathfinder.find_temporal_chain(snapshot, domain_ctx, start_time, end_time);

        let path_msgs: Vec<SemanticPathMsg> = paths
            .into_iter()
            .map(|p| SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p
                    .type_distribution
                    .into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p
                    .domains
                    .into_iter()
                    .map(|d| d.as_str().to_string())
                    .collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            })
            .collect();

        StorageResponse::FindTemporalChainOk { paths: path_msgs }
    }

    fn handle_find_causal_chain(
        &self,
        namespace: Option<String>,
        start_id: String,
        causal_type: String,
        max_depth: u32,
    ) -> StorageResponse {
        let storage = self.get_storage(namespace);
        use crate::semantic::SemanticPathFinder;

        let start = ConceptId::from_string(&start_id);
        let causal = parse_causal_type(&causal_type).unwrap_or(CausalType::Direct);

        let pathfinder = SemanticPathFinder::new(max_depth as usize, 100);
        let snapshot = storage.get_snapshot();

        let paths = pathfinder.find_causal_chain(snapshot, start, causal);

        let path_msgs: Vec<SemanticPathMsg> = paths
            .into_iter()
            .map(|p| SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p
                    .type_distribution
                    .into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p
                    .domains
                    .into_iter()
                    .map(|d| d.as_str().to_string())
                    .collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            })
            .collect();

        StorageResponse::FindCausalChainOk { paths: path_msgs }
    }

    fn handle_find_contradictions(
        &self,
        namespace: Option<String>,
        domain: String,
    ) -> StorageResponse {
        let storage = self.get_storage(namespace);
        use crate::semantic::SemanticPathFinder;

        let domain_ctx = parse_domain_context(&domain).unwrap_or(DomainContext::General);
        let pathfinder = SemanticPathFinder::default();
        let snapshot = storage.get_snapshot();

        let contradictions = pathfinder.find_contradictions(snapshot, domain_ctx);

        let contradiction_msgs: Vec<(String, String, String)> = contradictions
            .into_iter()
            .map(|(id1, id2, reason)| (id1.to_hex(), id2.to_hex(), reason))
            .collect();

        StorageResponse::FindContradictionsOk {
            contradictions: contradiction_msgs,
        }
    }

    fn handle_query_by_semantic(
        &self,
        namespace: Option<String>,
        filter_msg: SemanticFilterMsg,
        limit: Option<usize>,
    ) -> StorageResponse {
        let storage = self.get_storage(namespace);
        use crate::semantic::{CausalFilter, SemanticFilter, TemporalConstraint};

        // Convert message filter to internal filter
        let mut filter = SemanticFilter::new();

        if let Some(ref st) = filter_msg.semantic_type {
            if let Some(semantic_type) = parse_semantic_type(st) {
                filter = filter.with_type(semantic_type);
            }
        }

        if let Some(ref dc) = filter_msg.domain_context {
            if let Some(domain) = parse_domain_context(dc) {
                filter = filter.with_domain(domain);
            }
        }

        if let Some(after) = filter_msg.temporal_after {
            filter = filter.with_temporal(TemporalConstraint::After(after));
        }

        if let Some(before) = filter_msg.temporal_before {
            filter = filter.with_temporal(TemporalConstraint::Before(before));
        }

        if filter_msg.has_causal_relation {
            filter = filter.with_causal(CausalFilter::HasCausalRelation);
        }

        filter = filter.with_min_confidence(filter_msg.min_confidence);

        for term in filter_msg.required_terms {
            filter = filter.with_term(term);
        }

        // Query concepts
        let snapshot = storage.get_snapshot();
        let mut concepts = Vec::new();

        for concept in snapshot.all_concepts() {
            if let Some(ref semantic) = concept.semantic {
                let content = String::from_utf8_lossy(&concept.content);
                if filter.matches(semantic, &content, &concept.id) {
                    concepts.push(ConceptWithSemanticMsg {
                        concept_id: concept.id.to_hex(),
                        content: content.to_string(),
                        semantic_type: semantic.semantic_type.as_str().to_string(),
                        domain: semantic.domain_context.as_str().to_string(),
                        confidence: semantic.classification_confidence,
                    });

                    if let Some(lim) = limit {
                        if concepts.len() >= lim {
                            break;
                        }
                    }
                }
            }
        }

        StorageResponse::QueryBySemanticOk { concepts }
    }
}

// Helper functions for parsing semantic types from strings
use crate::types::ConceptId;

fn parse_semantic_type(s: &str) -> Option<SemanticType> {
    match s.to_lowercase().as_str() {
        "entity" => Some(SemanticType::Entity),
        "event" => Some(SemanticType::Event),
        "rule" => Some(SemanticType::Rule),
        "temporal" => Some(SemanticType::Temporal),
        "negation" => Some(SemanticType::Negation),
        "condition" => Some(SemanticType::Condition),
        "causal" => Some(SemanticType::Causal),
        "quantitative" => Some(SemanticType::Quantitative),
        "definitional" => Some(SemanticType::Definitional),
        _ => None,
    }
}

fn parse_domain_context(s: &str) -> Option<DomainContext> {
    match s.to_lowercase().as_str() {
        "medical" => Some(DomainContext::Medical),
        "legal" => Some(DomainContext::Legal),
        "financial" => Some(DomainContext::Financial),
        "technical" => Some(DomainContext::Technical),
        "scientific" => Some(DomainContext::Scientific),
        "business" => Some(DomainContext::Business),
        "general" => Some(DomainContext::General),
        _ => None,
    }
}

fn parse_causal_type(s: &str) -> Option<CausalType> {
    match s.to_lowercase().as_str() {
        "direct" => Some(CausalType::Direct),
        "indirect" => Some(CausalType::Indirect),
        "enabling" => Some(CausalType::Enabling),
        "preventing" => Some(CausalType::Preventing),
        "correlation" => Some(CausalType::Correlation),
        _ => None,
    }
}

/// Sharded Storage Server - wraps NamespaceManager with TCP protocol
pub struct ShardedStorageServer {
    namespaces: Arc<NamespaceManager>,
    start_time: std::time::Instant,
    pipeline: LearningPipeline,
}

impl ShardedStorageServer {
    /// Create new sharded storage server
    pub async fn new(storage: ShardedStorage) -> Self {
        // Use first shard config as template for namespaces
        let config = storage.get_shard_by_index(0).config().clone();
        let base_path = config
            .storage_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        let manager = NamespaceManager::new(base_path, config.clone())
            .expect("Failed to init NamespaceManager");

        // Note: For sharded server, the namespaces are actually individual ConcurrentMemory instances for now.
        // Distributed sharding across namespaces is a future enhancement.

        let pipeline = LearningPipeline::new()
            .await
            .expect("Failed to init learning pipeline");

        Self {
            namespaces: Arc::new(manager),
            start_time: std::time::Instant::now(),
            pipeline,
        }
    }

    /// Helper to get storage for a namespace
    fn get_storage(&self, namespace: Option<String>) -> Arc<ConcurrentMemory> {
        let ns = namespace.unwrap_or_else(|| "default".to_string());
        self.namespaces.get_namespace(&ns)
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
                    eprintln!("Shutdown signal received, flushing all namespaces...");
                    if let Err(e) = self.namespaces.flush_all() {
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
            let response_bytes = rmp_serde::to_vec_named(&response)
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
        use crate::types::{AssociationType, ConceptId};

        match request {
            StorageRequest::LearnConceptV2 { namespace, content, options } => {
                let storage = self.get_storage(namespace);
                let learn_opts: LearnOptions = options.into();

                match self.pipeline.learn_concept(&storage, &content, &learn_opts).await {
                    Ok(concept_id) => StorageResponse::LearnConceptV2Ok { concept_id },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learning pipeline failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnBatch { namespace, contents, options } => {
                let storage = self.get_storage(namespace);
                let learn_opts: LearnOptions = options.into();

                match self.pipeline.learn_batch(&storage, &contents, &learn_opts).await {
                    Ok(concept_ids) => StorageResponse::LearnBatchOk { concept_ids },
                    Err(e) => StorageResponse::Error {
                        message: format!("Batch learning failed: {}", e),
                    },
                }
            }
            StorageRequest::LearnConcept {
                namespace,
                concept_id,
                content,
                embedding,
                strength,
                confidence,
            } => {
                let storage = self.get_storage(namespace);
                let id = ConceptId::from_string(&concept_id);
                let content_bytes = content.into_bytes();
                let vector = if embedding.is_empty() { None } else { Some(embedding) };

                match storage.learn_concept(id, content_bytes, vector, strength, confidence, std::collections::HashMap::new()) {
                    Ok(sequence) => StorageResponse::LearnConceptOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn concept failed: {:?}", e),
                    },
                }
            }

            StorageRequest::LearnAssociation {
                namespace,
                source_id,
                target_id,
                assoc_type,
                confidence,
            } => {
                let storage = self.get_storage(namespace);
                let source = ConceptId::from_string(&source_id);
                let target = ConceptId::from_string(&target_id);
                let atype = AssociationType::from_u8(assoc_type as u8)
                    .unwrap_or(AssociationType::Semantic);

                match storage.learn_association(source, target, atype, confidence) {
                    Ok(sequence) => StorageResponse::LearnAssociationOk { sequence },
                    Err(e) => StorageResponse::Error {
                        message: format!("Learn association failed: {:?}", e),
                    },
                }
            }

            StorageRequest::QueryConcept { namespace, concept_id } => {
                let storage = self.get_storage(namespace);
                let id = ConceptId::from_string(&concept_id);

                if let Some(node) = storage.query_concept(&id) {
                    StorageResponse::QueryConceptOk {
                        found: true,
                        concept_id: id.to_hex(),
                        content: String::from_utf8_lossy(&node.content).to_string(),
                        strength: node.strength,
                        confidence: node.confidence,
                        attributes: node.attributes.clone(),
                    }
                } else {
                    StorageResponse::QueryConceptOk {
                        found: false,
                        concept_id: String::new(),
                        content: String::new(),
                        strength: 0.0,
                        confidence: 0.0,
                        attributes: std::collections::HashMap::new(),
                    }
                }
            }

            StorageRequest::GetNeighbors { namespace, concept_id } => {
                let storage = self.get_storage(namespace);
                let id = ConceptId::from_string(&concept_id);
                let neighbors = storage.query_neighbors(&id);
                let neighbor_ids = neighbors.iter().map(|id: &ConceptId| id.to_hex()).collect();

                StorageResponse::GetNeighborsOk { neighbor_ids }
            }

            StorageRequest::FindPath {
                namespace,
                start_id,
                end_id,
                max_depth,
            } => {
                let storage = self.get_storage(namespace);
                if let Some(path) = storage.find_path(ConceptId::from_string(&start_id), ConceptId::from_string(&end_id), max_depth as usize) {
                    StorageResponse::FindPathOk {
                        found: true,
                        path: path.iter().map(|id: &ConceptId| id.to_hex()).collect(),
                    }
                } else {
                    StorageResponse::FindPathOk {
                        found: false,
                        path: vec![],
                    }
                }
            }

            StorageRequest::VectorSearch {
                namespace,
                query_vector,
                k,
                ef_search,
            } => {
                let storage = self.get_storage(namespace);
                let results = storage.vector_search(&query_vector, k as usize, ef_search as usize);
                let results_vec = results
                    .into_iter()
                    .map(|(id, sim)| (id.to_hex(), sim))
                    .collect();

                StorageResponse::VectorSearchOk { results: results_vec }
            }

            StorageRequest::GetStats { namespace } => {
                let storage = self.get_storage(namespace);
                let stats = storage.stats();
                let hnsw_stats = storage.hnsw_stats();
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

            StorageRequest::Flush => match self.namespaces.flush_all() {
                Ok(_) => StorageResponse::FlushOk,
                Err(e) => StorageResponse::Error {
                    message: format!("Flush failed: {:?}", e),
                },
            },

            StorageRequest::HealthCheck => {
                let uptime = self.start_time.elapsed().as_secs();
                StorageResponse::HealthCheckOk {
                    healthy: true,
                    status: format!("sharded-namespaces ({} active)", self.namespaces.list_namespaces().len()),
                    uptime_seconds: uptime,
                }
            }

            StorageRequest::DeleteConcept { namespace, id } => {
                let storage = self.get_storage(Some(namespace));
                let concept_id = ConceptId::from_string(&id);
                match storage.delete_concept(concept_id) {
                    Ok(_) => StorageResponse::DeleteConceptOk { id: id.to_string() },
                    Err(e) => StorageResponse::Error { message: format!("Delete failed: {:?}", e) },
                }
            }

            StorageRequest::ClearCollection { namespace } => {
                let storage = self.get_storage(Some(namespace.clone()));
                match storage.clear() {
                    Ok(_) => StorageResponse::ClearCollectionOk { namespace: namespace.to_string() },
                    Err(e) => StorageResponse::Error { message: format!("Clear failed: {:?}", e) },
                }
            }

            StorageRequest::ListRecent { namespace, limit } => {
                let storage = self.get_storage(Some(namespace));
                let snapshot = storage.get_snapshot();
                let mut items: Vec<RecentItemMsg> = snapshot.concepts.values().map(|node| {
                    RecentItemMsg {
                        id: node.id.to_hex(),
                        content_preview: String::from_utf8_lossy(&node.content).chars().take(200).collect(),
                        created: node.created,
                        attributes: node.attributes.clone(),
                    }
                }).collect();

                items.sort_by(|a, b| b.created.cmp(&a.created));
                items.truncate(limit as usize);

                StorageResponse::ListRecentOk { items }
            }

            StorageRequest::LearnWithEmbedding { id, namespace, content, embedding, metadata, timestamp: _ } => {
                let storage = self.get_storage(Some(namespace));
                let concept_id = id.map(|s| ConceptId::from_string(&s))
                    .unwrap_or_else(|| ConceptId::from_string(&content));

                match storage.learn_concept(
                    concept_id,
                    content.into_bytes(),
                    Some(embedding),
                    1.0, 1.0,
                    metadata
                ) {
                    Ok(_) => StorageResponse::LearnConceptV2Ok { concept_id: concept_id.to_hex() },
                    Err(e) => StorageResponse::Error { message: format!("LearnWithEmbedding failed: {:?}", e) },
                }
            }

            // Semantic query handlers (limited support in sharded mode)
            StorageRequest::FindPathSemantic { .. } => {
                StorageResponse::Error {
                    message: "Semantic pathfinding not yet implemented for sharded storage. Use single-shard mode.".to_string(),
                }
            }

            StorageRequest::FindTemporalChain { .. } => {
                StorageResponse::Error {
                    message: "Temporal chain queries not yet implemented for sharded storage. Use single-shard mode.".to_string(),
                }
            }

            StorageRequest::FindCausalChain { .. } => {
                StorageResponse::Error {
                    message: "Causal chain queries not yet implemented for sharded storage. Use single-shard mode.".to_string(),
                }
            }

            StorageRequest::FindContradictions { .. } => {
                StorageResponse::Error {
                    message: "Contradiction detection not yet implemented for sharded storage. Use single-shard mode.".to_string(),
                }
            }

            StorageRequest::QueryBySemantic { .. } => {
                StorageResponse::Error {
                    message: "Semantic queries not yet implemented for sharded storage. Use single-shard mode.".to_string(),
                }
            }
            StorageRequest::TextSearch { namespace, query, limit } => {
                let storage = self.get_storage(namespace);
                match self.pipeline.search(&storage, &query, limit as usize).await {
                    Ok(results) => StorageResponse::TextSearchOk {
                        results: results.into_iter().map(|(id, score)| (id.to_hex(), score)).collect()
                    },
                    Err(e) => StorageResponse::Error { message: format!("Sharded TextSearch failed: {}", e) },
                }
            }
        }
    }
}
