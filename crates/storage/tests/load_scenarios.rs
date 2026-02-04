use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

use sutra_storage::{ConceptId, ConcurrentConfig, ConcurrentMemory};

#[tokio::test]
async fn test_configurable_concurrency_load() {
    let concurrency: usize = std::env::var("SUTRA_TEST_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);

    let ops_per_task: usize = std::env::var("SUTRA_TEST_OPS_PER_TASK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 8,
        memory_threshold: 2000,
        ..Default::default()
    };

    let storage = Arc::new(ConcurrentMemory::new(config));

    let mut handles = Vec::with_capacity(concurrency);
    for t in 0..concurrency {
        let storage = Arc::clone(&storage);
        handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let id = ConceptId::from_string(&format!("load-{}-{}", t, i));
                let content = format!("Load test concept {} {}", t, i).into_bytes();
                storage
                    .learn_concept(id, content, None, 1.0, 0.9, HashMap::new())
                    .unwrap();
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Give reconciler a moment to catch up
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let stats = storage.stats();

    assert!(stats.snapshot.concept_count >= concurrency * ops_per_task);
}
