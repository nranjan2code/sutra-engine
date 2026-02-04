use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use sutra_storage::semantic::SemanticAnalyzer;
use sutra_storage::{ConceptId, ConcurrentConfig, ConcurrentMemory};

fn embed(text: &str, dim: usize) -> Vec<f32> {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    (0..dim)
        .map(|i| ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0)
        .collect()
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn soak_multi_hour_stability() {
    if std::env::var("SUTRA_SOAK_RUN").ok().as_deref() != Some("1") {
        eprintln!("Skipping soak test (set SUTRA_SOAK_RUN=1 to enable).");
        return;
    }

    let minutes = env_u64("SUTRA_SOAK_MINUTES", 120);
    let concurrency = env_usize("SUTRA_SOAK_CONCURRENCY", 4);
    let sleep_ms = env_u64("SUTRA_SOAK_SLEEP_MS", 5);
    let flush_every = env_u64("SUTRA_SOAK_FLUSH_EVERY", 1000);

    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 8,
        memory_threshold: 10_000,
        ..Default::default()
    };

    let storage = Arc::new(ConcurrentMemory::new(config));
    let analyzer = Arc::new(SemanticAnalyzer::new());
    let counter = Arc::new(AtomicU64::new(0));

    let deadline = Instant::now() + Duration::from_secs(minutes * 60);

    let mut handles = Vec::with_capacity(concurrency);
    for worker in 0..concurrency {
        let storage = Arc::clone(&storage);
        let analyzer = Arc::clone(&analyzer);
        let counter = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            while Instant::now() < deadline {
                let idx = counter.fetch_add(1, Ordering::Relaxed);
                let content = format!(
                    "Soak concept {} {}: natural language systems evolve under load.",
                    worker, idx
                );
                let embedding = embed(&content, 8);
                let semantic = analyzer.analyze(&content);
                let id = ConceptId::from_string(&format!("soak-{}-{}", worker, idx));

                storage
                    .learn_concept_with_semantic(
                        id,
                        content.as_bytes().to_vec(),
                        Some(embedding.clone()),
                        0.9,
                        0.9,
                        semantic,
                    )
                    .unwrap();

                if idx % 10 == 0 {
                    let _ = storage.query_concept(&id);
                }
                if idx % 25 == 0 {
                    let _ = storage.vector_search(&embedding, 1, 32);
                }
                if idx % flush_every == 0 {
                    let _ = storage.flush();
                }

                if sleep_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                }
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let stats = storage.stats();
    assert!(stats.snapshot.concept_count > 0);
    storage.shutdown();
}
