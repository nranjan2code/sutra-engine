use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use sutra_storage::embedding_provider::EmbeddingProvider;
use sutra_storage::learning_pipeline::LearningPipeline;
use sutra_storage::tcp_server::{StorageRequest, StorageResponse, StorageServer};
use sutra_storage::{ConcurrentConfig, ConcurrentMemory};

struct MockEmbeddingProvider {
    dim: usize,
}

impl MockEmbeddingProvider {
    fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn embed(&self, text: &str, normalize: bool) -> Vec<f32> {
        let mut hash: u64 = 0xcbf29ce484222325;
        for b in text.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        let mut vec = Vec::with_capacity(self.dim);
        for i in 0..self.dim {
            let v = (hash.wrapping_add((i as u64) * 31) % 1000) as f32 / 1000.0;
            vec.push(v);
        }

        if normalize {
            let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut vec {
                    *v /= norm;
                }
            }
        }

        vec
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate(&self, text: &str, normalize: bool) -> anyhow::Result<Vec<f32>> {
        Ok(self.embed(text, normalize))
    }

    async fn generate_batch(&self, texts: &[String], normalize: bool) -> Vec<Option<Vec<f32>>> {
        texts
            .iter()
            .map(|t| Some(self.embed(t, normalize)))
            .collect()
    }
}

async fn send_request(
    stream: &mut TcpStream,
    request: &StorageRequest,
) -> anyhow::Result<StorageResponse> {
    let bytes = rmp_serde::to_vec_named(request)?;
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    let len = stream.read_u32().await?;
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    let response: StorageResponse = rmp_serde::from_slice(&buf)?;
    Ok(response)
}

#[tokio::test]
async fn test_tcp_learn_query_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 8,
        memory_threshold: 1000,
        ..Default::default()
    };

    let storage = ConcurrentMemory::new(config);
    let provider = Arc::new(MockEmbeddingProvider::new(8));
    let pipeline = LearningPipeline::new_with_provider(provider).await.unwrap();

    let server = Arc::new(StorageServer::new_with_pipeline(storage, pipeline));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server_task = tokio::spawn(server.clone().serve_with_shutdown(addr, async {
        let _ = shutdown_rx.await;
    }));

    let mut stream = {
        let start = std::time::Instant::now();
        loop {
            match TcpStream::connect(addr).await {
                Ok(stream) => break stream,
                Err(_) => {
                    if start.elapsed() > std::time::Duration::from_secs(1) {
                        panic!("timeout waiting for tcp server to accept connections");
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }
    };

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "tcp_test".to_string());

    let request = StorageRequest::LearnWithEmbedding {
        id: None,
        namespace: "default".to_string(),
        content: "Natural language systems reason over context.".to_string(),
        embedding: vec![0.1; 8],
        metadata,
        timestamp: None,
    };

    let response = send_request(&mut stream, &request).await.unwrap();
    let concept_id = match response {
        StorageResponse::LearnConceptV2Ok { concept_id } => concept_id,
        other => panic!("Unexpected response: {:?}", other),
    };

    let query = StorageRequest::QueryConcept {
        namespace: Some("default".to_string()),
        concept_id: concept_id.clone(),
    };

    let start = std::time::Instant::now();
    let response = loop {
        let resp = send_request(&mut stream, &query).await.unwrap();
        match &resp {
            StorageResponse::QueryConceptOk { found, .. } if *found => break resp,
            _ => {
                if start.elapsed() > std::time::Duration::from_secs(1) {
                    break resp;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    };

    match response {
        StorageResponse::QueryConceptOk {
            found,
            content,
            attributes,
            ..
        } => {
            assert!(found);
            assert!(content.contains("Natural language systems"));
            assert_eq!(attributes.get("source").unwrap(), "tcp_test");
        }
        other => panic!("Unexpected response: {:?}", other),
    }

    let _ = shutdown_tx.send(());
    let _ = server_task.await;
}
