//! Storage client for TCP communication with storage server
//! Uses unified learning API (embeddings + associations handled by storage)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value as JsonValue;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub content: String,
    pub metadata: HashMap<String, JsonValue>,
    pub embedding: Option<Vec<f32>>,  // Deprecated: storage server generates embeddings
}

// Learning options for unified API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnOptions {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}

impl Default for LearnOptions {
    fn default() -> Self {
        Self {
            generate_embedding: true,
            embedding_model: None,  // Use server default
            extract_associations: true,
            min_association_confidence: 0.5,
            max_associations_per_concept: 10,
            strength: 1.0,
            confidence: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpStorageClient {
    server_address: String,
    client: Option<StorageClientWrapper>,
}

// Protocol messages (mirror of storage server's TCP protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LearnOptionsWire {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}

impl From<LearnOptions> for LearnOptionsWire {
    fn from(o: LearnOptions) -> Self {
        Self {
            generate_embedding: o.generate_embedding,
            embedding_model: o.embedding_model,
            extract_associations: o.extract_associations,
            min_association_confidence: o.min_association_confidence,
            max_associations_per_concept: o.max_associations_per_concept,
            strength: o.strength,
            confidence: o.confidence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StorageRequest {
    LearnConceptV2 { content: String, options: LearnOptionsWire },
    LearnBatch { contents: Vec<String>, options: LearnOptionsWire },
    GetStats,
    Flush,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StorageResponse {
    LearnConceptV2Ok { concept_id: String },
    LearnBatchOk { concept_ids: Vec<String> },
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
    HealthCheckOk { healthy: bool, status: String, uptime_seconds: u64 },
    Error { message: String },
}

// Wrapper for the actual storage client
#[derive(Debug, Clone)]
struct StorageClientWrapper {
    // This will use the existing sutra-storage-client-tcp
    // For now, we'll simulate it
    connected: bool,
}

impl TcpStorageClient {
    pub async fn new(server_address: &str) -> Result<Self> {
        info!("Connecting to TCP storage server: {}", server_address);
        
        // Try to connect to the storage server
        match Self::try_connect(server_address).await {
            Ok(client) => {
                info!("Successfully connected to storage server");
                Ok(Self {
                    server_address: server_address.to_string(),
                    client: Some(client),
                })
            }
            Err(e) => {
                warn!("Failed to connect to storage server: {}", e);
                warn!("Running in mock mode for testing");
                Ok(Self {
                    server_address: server_address.to_string(),
                    client: None,
                })
            }
        }
    }
    
    async fn try_connect(server_address: &str) -> Result<StorageClientWrapper> {
        // Simple connectivity check
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        if let Ok(mut stream) = TcpStream::connect(server_address).await {
            stream.set_nodelay(true)?;
            Ok(StorageClientWrapper { connected: true })
        } else {
            Err(anyhow::anyhow!("Cannot connect to storage server"))
        }
    }
    
    /// Batch learn concepts using unified learning API
    /// Storage server handles: embedding generation + association extraction + storage
    pub async fn batch_learn_concepts(&mut self, concepts: Vec<Concept>) -> Result<Vec<String>> {
        if let Some(client) = &self.client {
            // Real TCP storage communication with unified API
            self.batch_learn_real_v2(concepts).await
        } else {
            // Mock mode for testing
            self.batch_learn_mock(concepts).await
        }
    }
    
    /// Real TCP batch learning using unified API (v2)
    async fn batch_learn_real_v2(&mut self, concepts: Vec<Concept>) -> Result<Vec<String>> {
        info!("Learning {} concepts via unified TCP API (embeddings + associations)", concepts.len());
        
        // Extract contents
        let contents: Vec<String> = concepts.iter().map(|c| c.content.clone()).collect();
        let options_wire: LearnOptionsWire = LearnOptions::default().into();
        
        // Connect
        let addr = self.server_address.clone();
        let mut stream = TcpStream::connect(&addr).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to storage at {}: {}", addr, e))?;
        stream.set_nodelay(true)?;
        
        // Build request
        let request = StorageRequest::LearnBatch { contents, options: options_wire };
        let bytes = rmp_serde::to_vec(&request)?;
        
        // Send length-prefixed MsgPack
        stream.write_u32(bytes.len() as u32).await?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;
        
        // Read response
        let len = stream.read_u32().await?;
        let mut buf = vec![0u8; len as usize];
        stream.read_exact(&mut buf).await?;
        
        // Deserialize response
        let resp: StorageResponse = rmp_serde::from_slice(&buf)
            .map_err(|e| anyhow::anyhow!("Invalid response from storage: {}", e))?;
        
        match resp {
            StorageResponse::LearnBatchOk { concept_ids } => Ok(concept_ids),
            StorageResponse::Error { message } => Err(anyhow::anyhow!("Storage error: {}", message)),
            other => Err(anyhow::anyhow!("Unexpected response: {:?}", other)),
        }
    }
    
    async fn batch_learn_mock(&mut self, concepts: Vec<Concept>) -> Result<Vec<String>> {
        warn!("Using mock storage - concepts not persisted!");
        
        let concept_ids: Vec<String> = concepts
            .iter()
            .map(|concept| self.generate_concept_id(&concept.content))
            .collect();
            
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(concept_ids)
    }
    
    fn generate_concept_id(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("concept_{:016x}", hash)
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        match &self.client {
            Some(_) => {
                // Try to ping the storage server
                match tokio::net::TcpStream::connect(&self.server_address).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            None => Ok(false), // Mock mode
        }
    }
}
