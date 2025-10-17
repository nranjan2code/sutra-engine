/// Standalone storage server
/// 
/// Runs ConcurrentMemory as a service that multiple clients can connect to.
/// Uses gRPC for high-performance RPC with streaming support.

use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::concurrent_memory::{ConcurrentMemory, ConcurrentConfig, ConcurrentStats};
use crate::types::{ConceptId, AssociationType};

pub mod storage_proto {
    tonic::include_proto!("sutra.storage");
}

use storage_proto::storage_service_server::{StorageService, StorageServiceServer};
use storage_proto::*;

/// Storage server implementation
pub struct StorageServer {
    /// Shared storage instance
    storage: Arc<RwLock<ConcurrentMemory>>,
}

impl StorageServer {
    pub fn new(config: ConcurrentConfig) -> Self {
        let storage = ConcurrentMemory::new(config);
        Self {
            storage: Arc::new(RwLock::new(storage)),
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
                ..Default::default()
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
        
        Ok(Response::new(StatsResponse {
            concepts: stats.snapshot.concept_count as u64,
            edges: stats.snapshot.edge_count as u64,
            written: stats.write_log.written,
            dropped: stats.write_log.dropped,
            pending: stats.write_log.pending,
            reconciliations: stats.reconciler.reconciliations,
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
}

/// Run the storage server
pub async fn run_server(config: ConcurrentConfig, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let storage_server = StorageServer::new(config);
    
    println!("🚀 Storage server starting at {}", addr);
    println!("📁 Storage path: {:?}", config.storage_path);
    
    Server::builder()
        .add_service(StorageServiceServer::new(storage_server))
        .serve(addr)
        .await?;
    
    Ok(())
}
