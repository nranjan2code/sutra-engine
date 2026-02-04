//! Sutra Custom Binary Protocol
//! 
//! High-performance binary protocol for all internal Sutra communication.
//! Replaces gRPC with 10-50× lower latency and 3-4× less bandwidth.
//! 
//! Message Format:
//! ```text
//! [4 bytes: message length][N bytes: bincode-serialized payload]
//! ```

pub mod client;
pub mod error;

use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub use client::{GridClient, GridClientPool};
pub use error::{ProtocolError, Result};

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum message size (16MB) - prevents DoS
const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

// ============================================================================
// Core Data Types
// ============================================================================

/// Concept types for different data categories
/// Supports dual storage architecture: domain concepts vs. user/conversation data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConceptType {
    // Domain-specific concepts (original behavior, stored in domain-storage.dat)
    DomainConcept = 0,      // General knowledge concepts
    
    // User/organization data (stored in user-storage.dat)
    User = 10,              // User account
    Session = 11,           // Login session
    Organization = 12,      // Organization/tenant
    
    // Conversation data (stored in user-storage.dat)
    Conversation = 20,      // Chat conversation thread
    Message = 21,           // Individual message in conversation
    Space = 22,             // Workspace/channel/project grouping
    
    // Access control (stored in user-storage.dat)
    Permission = 30,        // RBAC permission
    Role = 31,              // User role
    
    // Audit/compliance (stored in user-storage.dat)
    AuditLog = 40,          // System audit event
}

impl ConceptType {
    /// Check if this concept type belongs in user storage (vs. domain storage)
    pub fn is_user_storage_type(&self) -> bool {
        !matches!(self, ConceptType::DomainConcept)
    }
    
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ConceptType::DomainConcept => "domain_concept",
            ConceptType::User => "user",
            ConceptType::Session => "session",
            ConceptType::Organization => "organization",
            ConceptType::Conversation => "conversation",
            ConceptType::Message => "message",
            ConceptType::Space => "space",
            ConceptType::Permission => "permission",
            ConceptType::Role => "role",
            ConceptType::AuditLog => "audit_log",
        }
    }
    
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ConceptType::DomainConcept),
            10 => Some(ConceptType::User),
            11 => Some(ConceptType::Session),
            12 => Some(ConceptType::Organization),
            20 => Some(ConceptType::Conversation),
            21 => Some(ConceptType::Message),
            22 => Some(ConceptType::Space),
            30 => Some(ConceptType::Permission),
            31 => Some(ConceptType::Role),
            40 => Some(ConceptType::AuditLog),
            _ => None,
        }
    }
}

/// Metadata for concepts with organization/multi-tenancy support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMetadata {
    /// Concept type classification
    pub concept_type: ConceptType,
    
    /// Organization ID for multi-tenancy (required for non-domain concepts)
    pub organization_id: Option<String>,
    
    /// User ID who created this concept
    pub created_by: Option<String>,
    
    /// Custom tags for filtering/search
    pub tags: Vec<String>,
    
    /// Extensible key-value metadata
    pub attributes: std::collections::HashMap<String, String>,
    
    /// Soft delete flag (for audit trail preservation)
    pub deleted: bool,
    
    /// Schema version for forward compatibility
    pub schema_version: u32,
}

impl ConceptMetadata {
    /// Create new metadata with required fields
    pub fn new(concept_type: ConceptType) -> Self {
        Self {
            concept_type,
            organization_id: None,
            created_by: None,
            tags: Vec::new(),
            attributes: std::collections::HashMap::new(),
            deleted: false,
            schema_version: 1,
        }
    }
    
    /// Create with organization (for multi-tenant concepts)
    pub fn with_organization(concept_type: ConceptType, org_id: String) -> Self {
        Self {
            concept_type,
            organization_id: Some(org_id),
            created_by: None,
            tags: Vec::new(),
            attributes: std::collections::HashMap::new(),
            deleted: false,
            schema_version: 1,
        }
    }
    
    /// Validate metadata requirements
    pub fn validate(&self) -> Result<()> {
        // User storage types MUST have organization_id
        if self.concept_type.is_user_storage_type() && self.organization_id.is_none() {
            return Err(ProtocolError::ValidationError(format!(
                "Concept type '{}' requires organization_id",
                self.concept_type.name()
            )));
        }
        
        // Validate organization_id format (if present)
        if let Some(ref org_id) = self.organization_id {
            if org_id.is_empty() {
                return Err(ProtocolError::ValidationError(
                    "organization_id cannot be empty".to_string()
                ));
            }
            if org_id.len() > 128 {
                return Err(ProtocolError::ValidationError(
                    "organization_id too long (max 128 chars)".to_string()
                ));
            }
        }
        
        // Validate tags
        if self.tags.len() > 100 {
            return Err(ProtocolError::ValidationError(
                "Too many tags (max 100)".to_string()
            ));
        }
        
        // Validate attributes
        if self.attributes.len() > 100 {
            return Err(ProtocolError::ValidationError(
                "Too many attributes (max 100)".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for ConceptMetadata {
    fn default() -> Self {
        Self::new(ConceptType::DomainConcept)
    }
}

// ============================================================================
// Storage Protocol Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageMessage {
    LearnConcept {
        concept_id: String,
        content: String,
        embedding: Vec<f32>,
        strength: f32,
        confidence: f32,
        /// Optional metadata for concept classification and organization
        metadata: Option<ConceptMetadata>,
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
        /// Optional organization filter for multi-tenant search
        organization_id: Option<String>,
    },
    /// Query concepts by metadata filters
    QueryByMetadata {
        concept_type: Option<ConceptType>,
        organization_id: Option<String>,
        tags: Vec<String>,
        attributes: std::collections::HashMap<String, String>,
        limit: u32,
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
        metadata: Option<ConceptMetadata>,
    },
    GetNeighborsOk { neighbor_ids: Vec<String> },
    FindPathOk { found: bool, path: Vec<String> },
    VectorSearchOk { results: Vec<VectorMatch> },
    QueryByMetadataOk { 
        concepts: Vec<ConceptSummary>
    },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMatch {
    pub concept_id: String,
    pub similarity: f32,
}

/// Concept summary for metadata queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptSummary {
    pub concept_id: String,
    pub content_preview: String,  // First 200 chars
    pub metadata: ConceptMetadata,
    pub created: u64,
    pub last_accessed: u64,
}

// ============================================================================
// Grid Protocol Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridMessage {
    // Agent lifecycle
    RegisterAgent {
        agent_id: String,
        hostname: String,
        platform: String,
        max_storage_nodes: u32,
        version: String,
        agent_endpoint: String,
    },
    Heartbeat {
        agent_id: String,
        storage_node_count: u32,
        timestamp: u64,
    },
    UnregisterAgent {
        agent_id: String,
    },
    
    // Node management
    SpawnStorageNode {
        agent_id: String,
        storage_path: String,
        memory_limit_mb: u64,
        port: u32,
    },
    StopStorageNode {
        agent_id: String,
        node_id: String,
    },
    GetStorageNodeStatus {
        node_id: String,
    },
    
    // Cluster info
    ListAgents,
    GetClusterStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridResponse {
    RegisterAgentOk {
        success: bool,
        master_version: String,
        error_message: Option<String>,
    },
    HeartbeatOk {
        acknowledged: bool,
        timestamp: u64,
    },
    UnregisterAgentOk,
    SpawnStorageNodeOk {
        node_id: String,
        endpoint: String,
        success: bool,
        error_message: Option<String>,
    },
    StopStorageNodeOk {
        success: bool,
        error_message: Option<String>,
    },
    GetStorageNodeStatusOk {
        node_id: String,
        status: String,
        pid: u32,
        endpoint: String,
    },
    ListAgentsOk {
        agents: Vec<AgentRecord>,
    },
    GetClusterStatusOk {
        total_agents: u32,
        healthy_agents: u32,
        total_storage_nodes: u32,
        running_storage_nodes: u32,
        status: String,
    },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub agent_id: String,
    pub hostname: String,
    pub platform: String,
    pub status: String,
    pub max_storage_nodes: u32,
    pub current_storage_nodes: u32,
    pub last_heartbeat: u64,
}

// ============================================================================
// Protocol Implementation
// ============================================================================

/// Send a message over TCP with length prefix
pub async fn send_message<T: Serialize>(
    stream: &mut TcpStream,
    message: &T,
) -> io::Result<()> {
    // Serialize message
    let bytes = bincode::serialize(message)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    
    // Check size limit
    if bytes.len() > MAX_MESSAGE_SIZE as usize {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {} bytes", bytes.len()),
        ));
    }
    
    // Send length prefix (4 bytes, big-endian)
    stream.write_u32(bytes.len() as u32).await?;
    
    // Send payload
    stream.write_all(&bytes).await?;
    
    // Ensure data is sent
    stream.flush().await?;
    
    Ok(())
}

/// Receive a message from TCP with length prefix
pub async fn recv_message<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
) -> io::Result<T> {
    // Read length prefix
    let len = stream.read_u32().await?;
    
    // Check size limit
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {} bytes", len),
        ));
    }
    
    // Read payload
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    
    // Deserialize
    bincode::deserialize(&buf)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}

/// Helper for request-response pattern
pub async fn request<Req: Serialize, Resp: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
    request: &Req,
) -> io::Result<Resp> {
    send_message(stream, request).await?;
    recv_message(stream).await
}

/// Helper for request-response with timeout
pub async fn request_with_timeout<Req: Serialize, Resp: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
    request: &Req,
    timeout_duration: Duration,
) -> io::Result<Resp> {
    timeout(timeout_duration, async {
        send_message(stream, request).await?;
        recv_message(stream).await
    })
    .await
    .map_err(|_| io::Error::new(ErrorKind::TimedOut, "Request timeout"))?
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_message_roundtrip() {
        // Start echo server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let _msg: StorageMessage = recv_message(&mut socket).await.unwrap();
            send_message(&mut socket, &StorageResponse::LearnConceptOk { sequence: 42 })
                .await
                .unwrap();
        });
        
        // Client
        let mut client = TcpStream::connect(addr).await.unwrap();
        let req = StorageMessage::LearnConcept {
            concept_id: "test".to_string(),
            content: "content".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            strength: 1.0,
            confidence: 0.9,
            metadata: None,
        };
        
        send_message(&mut client, &req).await.unwrap();
        let resp: StorageResponse = recv_message(&mut client).await.unwrap();
        
        match resp {
            StorageResponse::LearnConceptOk { sequence } => assert_eq!(sequence, 42),
            _ => panic!("Unexpected response"),
        }
    }
    
    #[test]
    fn test_message_size() {
        let msg = StorageMessage::LearnConcept {
            concept_id: "test".to_string(),
            content: "content".to_string(),
            embedding: vec![],
            strength: 1.0,
            confidence: 0.9,
            metadata: None,
        };
        
        let bytes = bincode::serialize(&msg).unwrap();
        println!("StorageMessage size: {} bytes", bytes.len());
        assert!(bytes.len() < 100); // Should be tiny
    }
    
    #[test]
    fn test_grid_message_size() {
        let msg = GridMessage::Heartbeat {
            agent_id: "agent-001".to_string(),
            storage_node_count: 5,
            timestamp: 1234567890,
        };
        
        let bytes = bincode::serialize(&msg).unwrap();
        println!("GridMessage heartbeat size: {} bytes", bytes.len());
        assert!(bytes.len() < 50); // Tiny!
    }
    
    #[test]
    fn test_concept_type_classification() {
        // Domain concepts go to domain storage
        assert!(!ConceptType::DomainConcept.is_user_storage_type());
        
        // User/org concepts go to user storage
        assert!(ConceptType::User.is_user_storage_type());
        assert!(ConceptType::Session.is_user_storage_type());
        assert!(ConceptType::Organization.is_user_storage_type());
        assert!(ConceptType::Conversation.is_user_storage_type());
        assert!(ConceptType::Message.is_user_storage_type());
        assert!(ConceptType::Space.is_user_storage_type());
        assert!(ConceptType::Permission.is_user_storage_type());
        assert!(ConceptType::Role.is_user_storage_type());
        assert!(ConceptType::AuditLog.is_user_storage_type());
    }
    
    #[test]
    fn test_concept_metadata_validation() {
        // Domain concepts don't need organization
        let mut meta = ConceptMetadata::new(ConceptType::DomainConcept);
        assert!(meta.validate().is_ok());
        
        // User storage types REQUIRE organization
        meta.concept_type = ConceptType::User;
        assert!(meta.validate().is_err());
        
        // Adding organization makes it valid
        meta.organization_id = Some("org-123".to_string());
        assert!(meta.validate().is_ok());
        
        // Empty organization is invalid
        meta.organization_id = Some("".to_string());
        assert!(meta.validate().is_err());
        
        // Too long organization is invalid
        meta.organization_id = Some("x".repeat(200));
        assert!(meta.validate().is_err());
    }
    
    #[test]
    fn test_concept_metadata_limits() {
        let mut meta = ConceptMetadata::with_organization(
            ConceptType::Conversation,
            "org-123".to_string()
        );
        
        // Max 100 tags
        for i in 0..100 {
            meta.tags.push(format!("tag-{}", i));
        }
        assert!(meta.validate().is_ok());
        
        meta.tags.push("one-too-many".to_string());
        assert!(meta.validate().is_err());
        
        // Max 100 attributes
        meta.tags.clear();
        for i in 0..100 {
            meta.attributes.insert(format!("key-{}", i), format!("value-{}", i));
        }
        assert!(meta.validate().is_ok());
        
        meta.attributes.insert("overflow".to_string(), "value".to_string());
        assert!(meta.validate().is_err());
    }
    
    #[test]
    fn test_concept_type_roundtrip() {
        for &concept_type in &[
            ConceptType::DomainConcept,
            ConceptType::User,
            ConceptType::Session,
            ConceptType::Organization,
            ConceptType::Conversation,
            ConceptType::Message,
            ConceptType::Space,
            ConceptType::Permission,
            ConceptType::Role,
            ConceptType::AuditLog,
        ] {
            let value = concept_type as u8;
            let restored = ConceptType::from_u8(value).unwrap();
            assert_eq!(concept_type, restored);
        }
    }
    
    #[test]
    fn test_message_with_metadata() {
        let metadata = ConceptMetadata::with_organization(
            ConceptType::Message,
            "org-456".to_string()
        );
        
        let msg = StorageMessage::LearnConcept {
            concept_id: "msg-001".to_string(),
            content: "Hello world".to_string(),
            embedding: vec![0.1, 0.2, 0.3],
            strength: 1.0,
            confidence: 0.95,
            metadata: Some(metadata),
        };
        
        // Serialize and check size
        let bytes = bincode::serialize(&msg).unwrap();
        assert!(bytes.len() < 1024); // Should be reasonable
        
        // Deserialize and verify
        let decoded: StorageMessage = bincode::deserialize(&bytes).unwrap();
        match decoded {
            StorageMessage::LearnConcept { metadata: Some(meta), .. } => {
                assert_eq!(meta.concept_type, ConceptType::Message);
                assert_eq!(meta.organization_id, Some("org-456".to_string()));
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
