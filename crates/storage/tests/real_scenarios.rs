use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

use sutra_storage::embedding_provider::EmbeddingProvider;
use sutra_storage::learning_pipeline::{LearnOptions, LearningPipeline};
use sutra_storage::semantic::SemanticType;
use sutra_storage::{ConceptId, ConcurrentConfig, ConcurrentMemory, LearningStorage};

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

async fn wait_for_concept(storage: &ConcurrentMemory, id: &ConceptId, should_exist: bool) {
    let timeout = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();
    loop {
        let exists = storage.query_concept(id).is_some();
        if exists == should_exist {
            break;
        }
        if start.elapsed() > timeout {
            panic!(
                "timeout waiting for concept {:?} to exist={}",
                id, should_exist
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn test_natural_language_semantic_pipeline() {
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

    let mut options = LearnOptions::default();
    options.generate_embedding = true;
    options.extract_associations = false;
    options.analyze_semantics = true;

    let text = "High blood pressure causes cardiovascular disease.";
    let concept_hex = pipeline
        .learn_concept(&storage, text, &options)
        .await
        .unwrap();
    let concept_id = ConceptId::from_string(&concept_hex);

    wait_for_concept(&storage, &concept_id, true).await;
    let node = storage.query_concept(&concept_id).unwrap();

    assert!(node.vector.is_some());
    assert!(node.semantic.is_some());
    assert_eq!(node.semantic.unwrap().semantic_type, SemanticType::Causal);
}

#[tokio::test]
async fn test_persistence_recovery_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();

    let config = ConcurrentConfig {
        storage_path: base_path.clone(),
        vector_dimension: 8,
        memory_threshold: 1000,
        ..Default::default()
    };

    let storage = ConcurrentMemory::new(config.clone());
    let id = ConceptId::from_string("persisted-concept");
    let mut attributes = HashMap::new();
    attributes.insert("domain".to_string(), "finance".to_string());

    storage
        .learn_concept(
            id,
            b"Inflation impacts pricing.".to_vec(),
            None,
            1.0,
            0.9,
            attributes,
        )
        .unwrap();

    wait_for_concept(&storage, &id, true).await;
    storage.flush().unwrap();
    drop(storage);

    let storage_reloaded = ConcurrentMemory::new(config);
    wait_for_concept(&storage_reloaded, &id, true).await;
    let node = storage_reloaded.query_concept(&id).unwrap();

    assert_eq!(node.content.as_ref(), b"Inflation impacts pricing.");
    assert_eq!(node.attributes.get("domain").unwrap(), "finance");
}

#[tokio::test]
async fn test_concurrent_read_write_realistic() {
    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 8,
        memory_threshold: 2000,
        ..Default::default()
    };
    let storage = Arc::new(ConcurrentMemory::new(config));

    let writer = {
        let storage = Arc::clone(&storage);
        tokio::spawn(async move {
            for i in 0..200 {
                let id = ConceptId::from_string(&format!("nl-{}", i));
                let content = format!("Scenario {}: market volatility rises.", i).into_bytes();
                storage
                    .learn_concept(id, content, None, 1.0, 0.9, HashMap::new())
                    .unwrap();
            }
        })
    };

    let reader = {
        let storage = Arc::clone(&storage);
        tokio::spawn(async move {
            let mut found = 0usize;
            for i in 0..200 {
                let id = ConceptId::from_string(&format!("nl-{}", i));
                if storage.query_concept(&id).is_some() {
                    found += 1;
                }
                tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            }
            found
        })
    };

    let _ = writer.await;
    let found = reader.await.unwrap();

    assert!(found > 0);
}
