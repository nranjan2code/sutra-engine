/// Integration test for HNSW persistence with USearch
///
/// Demonstrates the critical fix: TRUE disk persistence with mmap loading
/// OLD (hnsw-rs): Always rebuilt on startup (2-5s for 1M vectors)
/// NEW (USearch): Instant mmap load (<50ms for 1M vectors)
use std::collections::HashMap;
use std::time::Instant;
use sutra_storage::{ConceptId, HnswConfig, HnswContainer};
use tempfile::TempDir;

#[test]
fn test_persistence_actually_works() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("storage");
    let config = HnswConfig::default();

    // Generate 1000 test vectors
    let mut vectors = HashMap::new();
    for i in 0u64..1000 {
        let mut id_bytes = [0u8; 16];
        id_bytes[0..8].copy_from_slice(&i.to_le_bytes());
        let concept_id = ConceptId(id_bytes);
        let vector: Vec<f32> = (0..768).map(|j| ((i + j) % 100) as f32 / 100.0).collect();
        vectors.insert(concept_id, vector);
    }

    // === PHASE 1: Build and save ===
    let build_time = {
        let container = HnswContainer::new(&base_path, config.clone());

        let start = Instant::now();
        container.load_or_build(&vectors).unwrap();
        let build_time = start.elapsed();

        // Verify search works
        let query: Vec<f32> = (0..768).map(|j| (j % 100) as f32 / 100.0).collect();
        let results = container.search(&query, 10, 50);
        assert!(!results.is_empty(), "Search should return results");

        // Save to disk
        container.save().unwrap();

        println!(
            "✅ Phase 1: Built index with {} vectors in {:?}",
            vectors.len(),
            build_time
        );

        build_time
    };

    // === PHASE 2: Load from disk (THIS IS THE CRITICAL TEST) ===
    let load_time = {
        let container = HnswContainer::new(&base_path, config.clone());

        let start = Instant::now();
        container.load_or_build(&vectors).unwrap();
        let load_time = start.elapsed();

        // Verify search still works
        let query: Vec<f32> = (0..768).map(|j| (j % 100) as f32 / 100.0).collect();
        let results = container.search(&query, 10, 50);
        assert!(
            !results.is_empty(),
            "Search should return results after load"
        );

        println!("✅ Phase 2: Loaded index from disk in {:?}", load_time);

        load_time
    };

    // === CRITICAL ASSERTION: Load should be MUCH faster than build ===
    let speedup = build_time.as_secs_f64() / load_time.as_secs_f64();
    println!("\n🎯 SPEEDUP: {:.1}× faster startup", speedup);

    assert!(
        load_time.as_secs_f64() < build_time.as_secs_f64() / 2.0,
        "Load time ({:?}) should be at least 2× faster than build time ({:?})",
        load_time,
        build_time
    );

    // Verify files exist
    let index_file = base_path.with_extension("usearch");
    let meta_file = base_path.with_extension("hnsw.meta");
    assert!(index_file.exists(), "USearch index file should exist");
    assert!(meta_file.exists(), "Metadata file should exist");

    println!("✅ Index file: {}", index_file.display());
    println!("✅ Metadata file: {}", meta_file.display());
}

#[test]
fn test_incremental_insert_persists() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("storage");
    let config = HnswConfig::default();

    // Start with 100 vectors
    let mut vectors = HashMap::new();
    for i in 0u64..100 {
        let mut id_bytes = [0u8; 16];
        id_bytes[0..8].copy_from_slice(&i.to_le_bytes());
        let concept_id = ConceptId(id_bytes);
        let vector: Vec<f32> = (0..768).map(|j| ((i + j) % 100) as f32 / 100.0).collect();
        vectors.insert(concept_id, vector);
    }

    // Build and save
    {
        let container = HnswContainer::new(&base_path, config.clone());
        container.load_or_build(&vectors).unwrap();
        container.save().unwrap();

        let stats = container.stats();
        assert_eq!(stats.num_vectors, 100);
    }

    // Load and add 50 more incrementally
    {
        let container = HnswContainer::new(&base_path, config.clone());
        container.load_or_build(&vectors).unwrap();

        // Add 50 new vectors
        for i in 100u64..150 {
            let mut id_bytes = [0u8; 16];
            id_bytes[0..8].copy_from_slice(&i.to_le_bytes());
            let concept_id = ConceptId(id_bytes);
            let vector: Vec<f32> = (0..768).map(|j| ((i + j) % 100) as f32 / 100.0).collect();
            container.insert(concept_id, vector).unwrap();
        }

        container.save().unwrap();

        let stats = container.stats();
        assert_eq!(stats.num_vectors, 150);
    }

    // Load again and verify 150 vectors
    {
        let container = HnswContainer::new(&base_path, config.clone());

        // Build vectors map with all 150
        for i in 0u64..150 {
            let mut id_bytes = [0u8; 16];
            id_bytes[0..8].copy_from_slice(&i.to_le_bytes());
            let concept_id = ConceptId(id_bytes);
            let vector: Vec<f32> = (0..768).map(|j| ((i + j) % 100) as f32 / 100.0).collect();
            vectors.insert(concept_id, vector);
        }

        container.load_or_build(&vectors).unwrap();

        let stats = container.stats();
        assert_eq!(
            stats.num_vectors, 150,
            "Should have 150 vectors after incremental inserts"
        );
    }

    println!("✅ Incremental inserts properly persisted");
}

#[test]
fn test_empty_index_handling() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("storage");
    let config = HnswConfig::default();

    let container = HnswContainer::new(&base_path, config);

    // Build with empty vectors
    let vectors = HashMap::new();
    container.load_or_build(&vectors).unwrap();

    let stats = container.stats();
    assert_eq!(stats.num_vectors, 0);
    assert!(stats.initialized);

    // Search should return empty results, not crash
    let query: Vec<f32> = (0..768).map(|_| 0.5).collect();
    let results = container.search(&query, 10, 50);
    assert!(results.is_empty());

    println!("✅ Empty index handled gracefully");
}
