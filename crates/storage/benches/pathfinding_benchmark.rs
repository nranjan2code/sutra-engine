use std::time::Instant;
/// Benchmark: Sequential vs Parallel Pathfinding
///
/// Compares performance of sequential BFS vs Rayon-based parallel pathfinding
/// on a diamond graph pattern with varying complexity.
use sutra_storage::{
    concurrent_memory::{ConcurrentConfig, ConcurrentMemory},
    parallel_paths::ParallelPathFinder,
    types::{AssociationType, ConceptId},
};
use tempfile::TempDir;

fn create_diamond_graph(memory: &ConcurrentMemory, layers: usize) -> (ConceptId, ConceptId) {
    let start = ConceptId([1; 16]);
    let end = ConceptId([255; 16]);

    memory
        .learn_concept(start, b"start".to_vec(), None, 1.0, 0.9)
        .unwrap();
    memory
        .learn_concept(end, b"end".to_vec(), None, 1.0, 0.9)
        .unwrap();

    let mut current_layer = vec![start];

    for layer in 0..layers {
        let mut next_layer = Vec::new();

        for (i, &parent) in current_layer.iter().enumerate() {
            // Create 2 children per parent (exponential growth)
            for child_idx in 0..2 {
                let mut id_bytes = [0u8; 16];
                let node_id = (layer * 100 + i * 2 + child_idx) as u64;
                id_bytes[0..8].copy_from_slice(&node_id.to_le_bytes());
                let child_id = ConceptId(id_bytes);

                memory
                    .learn_concept(
                        child_id,
                        format!("layer{}_node{}", layer, node_id)
                            .as_bytes()
                            .to_vec(),
                        None,
                        1.0,
                        0.9,
                    )
                    .unwrap();
                memory
                    .learn_association(parent, child_id, AssociationType::Semantic, 0.8)
                    .unwrap();

                next_layer.push(child_id);
            }
        }

        current_layer = next_layer;
    }

    // Connect last layer to end
    for &node in &current_layer {
        memory
            .learn_association(node, end, AssociationType::Semantic, 0.9)
            .unwrap();
    }

    // Wait for reconciliation
    std::thread::sleep(std::time::Duration::from_millis(500));

    (start, end)
}

fn main() {
    println!("🚀 Pathfinding Performance Benchmark");
    println!("=====================================\n");

    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        reconcile_interval_ms: 50,
        memory_threshold: 10_000,
        vector_dimension: 768,
    };

    let memory = ConcurrentMemory::new(config);

    // Test different graph sizes
    let test_cases = vec![
        (3, "Small (3 layers, 14 nodes)"),
        (4, "Medium (4 layers, 30 nodes)"),
        (5, "Large (5 layers, 62 nodes)"),
    ];

    for (layers, description) in test_cases {
        println!("Graph: {}", description);
        println!("{}", "-".repeat(50));

        let (start, end) = create_diamond_graph(&memory, layers);

        // Sequential pathfinding (find 10 paths)
        let seq_start = Instant::now();
        let mut paths_found = 0;
        for _ in 0..10 {
            if let Some(_) = memory.find_path(start, end, 10) {
                paths_found += 1;
            }
        }
        let seq_time = seq_start.elapsed();

        // Parallel pathfinding (find 10 paths)
        let par_start = Instant::now();
        let par_paths = memory.find_paths_parallel(start, end, 10, 10);
        let par_time = par_start.elapsed();

        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();

        println!("  Sequential: {} paths in {:?}", paths_found, seq_time);
        println!("  Parallel:   {} paths in {:?}", par_paths.len(), par_time);
        println!("  Speedup:    {:.2}×\n", speedup);

        // Clear for next test
        memory.shutdown().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("✅ Benchmark Complete");
    println!("\nKey Insights:");
    println!("- Parallel pathfinding scales with graph fanout (# of first-hop neighbors)");
    println!("- Diamond graphs exploit parallelism naturally (2^n paths)");
    println!("- Expected speedup: 4-8× on 8-core systems");
    println!("- Best case: High fanout + independent paths + multi-core CPU");
}
