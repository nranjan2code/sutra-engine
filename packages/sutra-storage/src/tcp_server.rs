//! Production-grade TCP storage server using custom binary protocol
//! 
//! Replaces gRPC server while maintaining distributed architecture.
//! Runs as standalone service - API/Hybrid connect over network.

use crate::concurrent_memory::ConcurrentMemory;
use crate::sharded_storage::ShardedStorage;
use crate::learning_pipeline::{LearningPipeline, LearnOptions};
use crate::semantic::{SemanticType, DomainContext, CausalType};
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

// 🔥 NEW: Semantic filter for TCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFilterMsg {
    pub semantic_type: Option<String>,  // "rule", "event", "entity", etc.
    pub domain_context: Option<String>, // "medical", "legal", "financial", etc.
    pub temporal_after: Option<i64>,    // Unix timestamp
    pub temporal_before: Option<i64>,   // Unix timestamp
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
            analyze_semantics: std::env::var("SUTRA_SEMANTIC_ANALYSIS").ok().and_then(|s| s.parse().ok()).unwrap_or(true),
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
    // 🔥 NEW: Semantic query operations
    FindPathSemantic {
        start_id: String,
        end_id: String,
        filter: SemanticFilterMsg,
        max_depth: u32,
        max_paths: u32,
    },
    FindTemporalChain {
        domain: Option<String>,  // "medical", "legal", etc.
        start_time: i64,
        end_time: i64,
    },
    FindCausalChain {
        start_id: String,
        causal_type: String,  // "direct", "indirect", "enabling", etc.
        max_depth: u32,
    },
    FindContradictions {
        domain: String,
    },
    QueryBySemantic {
        filter: SemanticFilterMsg,
        limit: Option<usize>,
    },
    TextSearch {
        query: String,
        limit: u32,
    },
    GetStats,
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

        // Configure for low latency and better throughput
        stream.set_nodelay(true)?;

        let mut request_count = 0u64;
        
        loop {
            let request_start = std::time::Instant::now();
            
            // Read message length (4 bytes)
            let len = match stream.read_u32().await {
                Ok(len) => len,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Client disconnected
                    eprintln!("Client {} disconnected after {} requests", peer_addr, request_count);
                    break;
                }
                Err(e) => return Err(e),
            };

            // ✅ PRODUCTION: Validate message size before allocating
            if len as usize > MAX_MESSAGE_SIZE {
                let error = StorageResponse::Error {
                    message: format!("Message too large: {} bytes (max: {})", len, MAX_MESSAGE_SIZE),
                };
                let response_bytes = rmp_serde::to_vec_named(&error)
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

            // Handle request (already async and non-blocking)
            let response = self.handle_request(request).await;

            // Serialize response (msgpack for Python clients)
            let response_bytes = rmp_serde::to_vec_named(&response)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            // Write response
            stream.write_u32(response_bytes.len() as u32).await?;
            stream.write_all(&response_bytes).await?;
            stream.flush().await?;
            
            request_count += 1;
            let request_duration = request_start.elapsed();
            
            // Log slow requests (> 1s)
            if request_duration.as_millis() > 1000 {
                eprintln!("⚠️  Slow request from {}: {}ms (total: {})", 
                    peer_addr, request_duration.as_millis(), request_count);
            }
        }

        eprintln!("Client disconnected: {}", peer_addr);
        Ok(())
    }

/// Handle storage request
    pub async fn handle_request(&self, request: StorageRequest) -> StorageResponse {
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
            
            // 🔥 NEW: Semantic query handlers
            StorageRequest::FindPathSemantic { start_id, end_id, filter, max_depth, max_paths } => {
                self.handle_find_path_semantic(start_id, end_id, filter, max_depth, max_paths)
            }
            
            StorageRequest::FindTemporalChain { domain, start_time, end_time } => {
                self.handle_find_temporal_chain(domain, start_time, end_time)
            }
            
            StorageRequest::FindCausalChain { start_id, causal_type, max_depth } => {
                self.handle_find_causal_chain(start_id, causal_type, max_depth)
            }
            
            StorageRequest::FindContradictions { domain } => {
                self.handle_find_contradictions(domain)
            }
            
            StorageRequest::QueryBySemantic { filter, limit } => {
                self.handle_query_by_semantic(filter, limit)
            }
            StorageRequest::TextSearch { query, limit } => {
                match self.pipeline.search(&self.storage, &query, limit as usize).await {
                    Ok(results) => StorageResponse::TextSearchOk { 
                        results: results.into_iter().map(|(id, score)| (id.to_hex(), score)).collect() 
                    },
                    Err(e) => StorageResponse::Error { message: format!("TextSearch failed: {}", e) },
                }
            }
        }
    }
    
    // 🔥 NEW: Semantic query implementation methods
    fn handle_find_path_semantic(
        &self,
        start_id: String,
        end_id: String,
        filter_msg: SemanticFilterMsg,
        max_depth: u32,
        max_paths: u32,
    ) -> StorageResponse {
        use crate::semantic::{SemanticPathFinder, SemanticFilter, TemporalConstraint, CausalFilter};
        
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
        let snapshot = self.storage.get_snapshot();
        
        // Find paths
        let paths = pathfinder.find_paths_filtered(snapshot, start, end, &filter);
        
        // Convert to message format
        let path_msgs: Vec<SemanticPathMsg> = paths.into_iter().map(|p| {
            SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p.type_distribution.into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p.domains.into_iter().map(|d| d.as_str().to_string()).collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            }
        }).collect();
        
        StorageResponse::FindPathSemanticOk { paths: path_msgs }
    }
    
    fn handle_find_temporal_chain(
        &self,
        domain: Option<String>,
        start_time: i64,
        end_time: i64,
    ) -> StorageResponse {
        use crate::semantic::SemanticPathFinder;
        
        let domain_ctx = domain.and_then(|d| parse_domain_context(&d));
        let pathfinder = SemanticPathFinder::default();
        let snapshot = self.storage.get_snapshot();
        
        let paths = pathfinder.find_temporal_chain(snapshot, domain_ctx, start_time, end_time);
        
        let path_msgs: Vec<SemanticPathMsg> = paths.into_iter().map(|p| {
            SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p.type_distribution.into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p.domains.into_iter().map(|d| d.as_str().to_string()).collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            }
        }).collect();
        
        StorageResponse::FindTemporalChainOk { paths: path_msgs }
    }
    
    fn handle_find_causal_chain(
        &self,
        start_id: String,
        causal_type: String,
        max_depth: u32,
    ) -> StorageResponse {
        use crate::semantic::SemanticPathFinder;
        
        let start = ConceptId::from_string(&start_id);
        let causal = parse_causal_type(&causal_type).unwrap_or(CausalType::Direct);
        
        let pathfinder = SemanticPathFinder::new(max_depth as usize, 100);
        let snapshot = self.storage.get_snapshot();
        
        let paths = pathfinder.find_causal_chain(snapshot, start, causal);
        
        let path_msgs: Vec<SemanticPathMsg> = paths.into_iter().map(|p| {
            SemanticPathMsg {
                concepts: p.concepts.iter().map(|id| id.to_hex()).collect(),
                confidence: p.confidence,
                type_distribution: p.type_distribution.into_iter()
                    .map(|(t, c)| (t.as_str().to_string(), c))
                    .collect(),
                domains: p.domains.into_iter().map(|d| d.as_str().to_string()).collect(),
                is_temporally_ordered: p.is_temporally_ordered,
            }
        }).collect();
        
        StorageResponse::FindCausalChainOk { paths: path_msgs }
    }
    
    fn handle_find_contradictions(&self, domain: String) -> StorageResponse {
        use crate::semantic::SemanticPathFinder;
        
        let domain_ctx = parse_domain_context(&domain).unwrap_or(DomainContext::General);
        let pathfinder = SemanticPathFinder::default();
        let snapshot = self.storage.get_snapshot();
        
        let contradictions = pathfinder.find_contradictions(snapshot, domain_ctx);
        
        let contradiction_msgs: Vec<(String, String, String)> = contradictions.into_iter()
            .map(|(id1, id2, reason)| (id1.to_hex(), id2.to_hex(), reason))
            .collect();
        
        StorageResponse::FindContradictionsOk { contradictions: contradiction_msgs }
    }
    
    fn handle_query_by_semantic(
        &self,
        filter_msg: SemanticFilterMsg,
        limit: Option<usize>,
    ) -> StorageResponse {
        use crate::semantic::{SemanticFilter, TemporalConstraint, CausalFilter};
        
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
        let snapshot = self.storage.get_snapshot();
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

                // Aggregate metrics from all shards
                let (total_dropped, total_pending, total_reconciliations) = stats.shard_stats.iter()
                    .fold((0u64, 0usize, 0u64), |(dropped, pending, recon), shard| {
                        (
                            dropped + shard.write_log.dropped,
                            pending + shard.write_log.pending,
                            recon + shard.reconciler.reconciliations,
                        )
                    });

                StorageResponse::StatsOk {
                    concepts: stats.total_concepts as u64,
                    edges: stats.total_edges as u64,
                    vectors: stats.total_vectors as u64,
                    written: stats.total_writes,
                    dropped: total_dropped,
                    pending: total_pending as u64,
                    reconciliations: total_reconciliations,
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
            
            // Semantic query handlers (not yet implemented for sharded storage)
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
            StorageRequest::TextSearch { query, limit } => {
                match self.pipeline.search(&self.storage, &query, limit as usize).await {
                    Ok(results) => StorageResponse::TextSearchOk { 
                        results: results.into_iter().map(|(id, score)| (id.to_hex(), score)).collect() 
                    },
                    Err(e) => StorageResponse::Error { message: format!("Sharded TextSearch failed: {}", e) },
                }
            }
        }
    }
}
