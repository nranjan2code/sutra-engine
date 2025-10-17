/// Simplified Sutra Storage Server
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::signal;
use tonic::{transport::Server, Request, Response, Status};

use sutra_storage::{ConcurrentMemory, ConcurrentConfig, ConceptId, AssociationType};

// Generated protobuf code
pub mod storage {
    tonic::include_proto!("sutra.storage");
}

use storage::storage_service_server::{StorageService, StorageServiceServer};
use storage::*;

/// Storage server state
pub struct StorageServer {
    storage: Arc<RwLock<ConcurrentMemory>>,
    start_time: std::time::Instant,
}

impl StorageServer {
    pub fn new(config: ConcurrentConfig) -> Self {
        log::info!("🚀 Initializing storage server");
        log::info!("   Storage path: {:?}", config.storage_path);
        
        let storage = ConcurrentMemory::new(config);
        let stats = storage.stats();
        
        log::info!("✅ Storage initialized");
        log::info!("   Concepts loaded: {}", stats.snapshot.concept_count);
        
        Self {
            storage: Arc::new(RwLock::new(storage)),
            start_time: std::time::Instant::now(),
        }
    }
}

#[tonic::async_trait]
impl StorageService for StorageServer {
    async fn learn_concept(
        &self,
        request: Request<LearnConceptRequest>,
    ) -> Result<Response<LearnConceptResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let id = ConceptId::from_string(&req.concept_id);
        let content = req.content.into_bytes();
        let vector = if req.embedding.is_empty() {
            None
        } else {
            Some(req.embedding)
        };
        
        match storage.learn_concept(id, content, vector, req.strength, req.confidence) {
            Ok(seq) => Ok(Response::new(LearnConceptResponse { sequence: seq })),
            Err(e) => Err(Status::internal(format!("Failed to learn concept: {:?}", e))),
        }
    }
    
    async fn learn_association(
        &self,
        request: Request<LearnAssociationRequest>,
    ) -> Result<Response<LearnAssociationResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let source = ConceptId::from_string(&req.source_id);
        let target = ConceptId::from_string(&req.target_id);
        let assoc_type = AssociationType::from_u8(req.assoc_type as u8)
            .unwrap_or(AssociationType::Semantic);
        
        match storage.learn_association(source, target, assoc_type, req.confidence) {
            Ok(seq) => Ok(Response::new(LearnAssociationResponse { sequence: seq })),
            Err(e) => Err(Status::internal(format!("Failed to learn association: {:?}", e))),
        }
    }
    
    async fn query_concept(
        &self,
        request: Request<QueryConceptRequest>,
    ) -> Result<Response<QueryConceptResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let id = ConceptId::from_string(&req.concept_id);
        
        match storage.query_concept(&id) {
            Some(node) => Ok(Response::new(QueryConceptResponse {
                found: true,
                concept_id: id.to_hex(),
                content: String::from_utf8_lossy(&node.content).to_string(),
                strength: node.strength,
                confidence: node.confidence,
            })),
            None => Ok(Response::new(QueryConceptResponse {
                found: false,
                concept_id: String::new(),
                content: String::new(),
                strength: 0.0,
                confidence: 0.0,
            })),
        }
    }
    
    async fn get_neighbors(
        &self,
        request: Request<GetNeighborsRequest>,
    ) -> Result<Response<GetNeighborsResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let id = ConceptId::from_string(&req.concept_id);
        let neighbors = storage.query_neighbors(&id);
        let neighbor_ids = neighbors.iter().map(|id| id.to_hex()).collect();
        
        Ok(Response::new(GetNeighborsResponse { neighbor_ids }))
    }
    
    async fn find_path(
        &self,
        request: Request<FindPathRequest>,
    ) -> Result<Response<FindPathResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let start = ConceptId::from_string(&req.start_id);
        let end = ConceptId::from_string(&req.end_id);
        
        match storage.find_path(start, end, req.max_depth as usize) {
            Some(path) => {
                let path_ids = path.iter().map(|id| id.to_hex()).collect();
                Ok(Response::new(FindPathResponse {
                    found: true,
                    path: path_ids,
                }))
            }
            None => Ok(Response::new(FindPathResponse {
                found: false,
                path: vec![],
            })),
        }
    }
    
    async fn vector_search(
        &self,
        request: Request<VectorSearchRequest>,
    ) -> Result<Response<VectorSearchResponse>, Status> {
        let req = request.into_inner();
        let storage = self.storage.read();
        
        let results = storage.vector_search(&req.query_vector, req.k as usize, req.ef_search as usize);
        let matches = results
            .into_iter()
            .map(|(id, similarity)| VectorMatch {
                concept_id: id.to_hex(),
                similarity,
            })
            .collect();
        
        Ok(Response::new(VectorSearchResponse { results: matches }))
    }
    
    async fn get_stats(
        &self,
        _request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        let storage = self.storage.read();
        let stats = storage.stats();
        let uptime = self.start_time.elapsed().as_secs();
        
        Ok(Response::new(StatsResponse {
            concepts: stats.snapshot.concept_count as u64,
            edges: stats.snapshot.edge_count as u64,
            written: stats.write_log.written as u64,
            dropped: stats.write_log.dropped as u64,
            pending: stats.write_log.pending as u64,
            reconciliations: stats.reconciler.reconciliations as u64,
            uptime_seconds: uptime,
        }))
    }
    
    async fn flush(
        &self,
        _request: Request<FlushRequest>,
    ) -> Result<Response<FlushResponse>, Status> {
        let storage = self.storage.read();
        
        match storage.flush() {
            Ok(_) => Ok(Response::new(FlushResponse { success: true })),
            Err(e) => Err(Status::internal(format!("Flush failed: {:?}", e))),
        }
    }
    
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let uptime = self.start_time.elapsed().as_secs();
        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            status: "OK".to_string(),
            uptime_seconds: uptime,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    // Parse configuration from environment
    let storage_path = std::env::var("STORAGE_PATH")
        .unwrap_or_else(|_| "/data".to_string());
    let host = std::env::var("STORAGE_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("STORAGE_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()?;
    
    let config = ConcurrentConfig {
        storage_path: PathBuf::from(storage_path),
        vector_dimension: 384,
        reconcile_interval_ms: 100,
        memory_threshold: 50_000,
    };
    
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let storage_server = StorageServer::new(config);
    
    log::info!("🚀 Storage server listening on {}", addr);
    
    Server::builder()
        .add_service(StorageServiceServer::new(storage_server))
        .serve_with_shutdown(addr, async {
            signal::ctrl_c().await.ok();
            log::info!("Shutting down gracefully...");
        })
        .await?;
    
    Ok(())
}
