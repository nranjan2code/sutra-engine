//! Sutra Custom Binary Protocol
//!
//! High-performance binary protocol for all internal Sutra communication.
//! Replaces gRPC with 10-50× lower latency and 3-4× less bandwidth.
//!
//! Message Format:
//! ```text
//! [4 bytes: message length][N bytes: bincode-serialized payload]
//! ```

pub mod error;

use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub use error::{ProtocolError, Result};

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum message size (16MB) - prevents DoS
const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

// ============================================================================
// Core Data Types
// ============================================================================

/// Extensible metadata for concepts (standalone engine).
pub type ConceptMetadata = std::collections::HashMap<String, String>;

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
        /// Optional metadata for concept classification
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
    },
    /// Query concepts by metadata filters
    QueryByMetadata {
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
    LearnConceptOk {
        sequence: u64,
    },
    LearnAssociationOk {
        sequence: u64,
    },
    QueryConceptOk {
        found: bool,
        concept_id: String,
        content: String,
        strength: f32,
        confidence: f32,
        metadata: Option<ConceptMetadata>,
    },
    GetNeighborsOk {
        neighbor_ids: Vec<String>,
    },
    FindPathOk {
        found: bool,
        path: Vec<String>,
    },
    VectorSearchOk {
        results: Vec<VectorMatch>,
    },
    QueryByMetadataOk {
        concepts: Vec<ConceptSummary>,
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
    Error {
        message: String,
    },
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
    pub content_preview: String, // First 200 chars
    pub metadata: ConceptMetadata,
    pub created: u64,
    pub last_accessed: u64,
}

// ============================================================================
// Protocol Implementation
// ============================================================================

/// Send a message over TCP with length prefix
pub async fn send_message<T: Serialize>(stream: &mut TcpStream, message: &T) -> io::Result<()> {
    // Serialize message
    let bytes =
        bincode::serialize(message).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

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
pub async fn recv_message<T: for<'de> Deserialize<'de>>(stream: &mut TcpStream) -> io::Result<T> {
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
    bincode::deserialize(&buf).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
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
            send_message(
                &mut socket,
                &StorageResponse::LearnConceptOk { sequence: 42 },
            )
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
    fn test_message_with_metadata() {
        let metadata = ConceptMetadata::from([
            ("type".to_string(), "message".to_string()),
            ("origin".to_string(), "unit-test".to_string()),
        ]);

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
            StorageMessage::LearnConcept {
                metadata: Some(meta),
                ..
            } => {
                assert_eq!(meta.get("type"), Some(&"message".to_string()));
                assert_eq!(meta.get("origin"), Some(&"unit-test".to_string()));
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
