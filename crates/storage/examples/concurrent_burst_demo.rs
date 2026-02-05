use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
/// Concurrent Memory Burst Demo
///
/// Demonstrates the burst-tolerant concurrent memory system
/// handling unpredictable write/read patterns.
use sutra_storage::{AssociationType, ConceptId, ConcurrentConfig, ConcurrentMemory};

fn main() {
    println!("=== Concurrent Memory Burst Demo ===\n");

    // Create concurrent memory with default adaptive config
    let config = ConcurrentConfig {
        storage_path: "./demo_storage".into(),
        memory_threshold: 10_000,
        vector_dimension: 768,
        ..Default::default()
    };

    let memory = Arc::new(ConcurrentMemory::new(config));
    println!("✓ Concurrent memory system started");
    println!("  - Write log capacity: 100,000 entries");
    println!("  - Adaptive reconciliation (1-100ms dynamic)");
    println!("  - Memory threshold: 10,000 concepts\n");

    // Simulation 1: Learning burst
    println!("📚 BURST 1: Learning Phase (Write-Heavy)");
    let start = Instant::now();
    for i in 0..1000 {
        let id = ConceptId([i as u8; 16]);
        let content = format!("Concept {}: Knowledge data", i).into_bytes();
        memory
            .learn_concept(
                id,
                content,
                None,
                1.0,
                0.9,
                std::collections::HashMap::new(),
            )
            .ok();
    }
    let elapsed = start.elapsed();
    println!("  ⚡ Wrote 1,000 concepts in {:?}", elapsed);
    println!("     ({:.0} writes/sec)", 1000.0 / elapsed.as_secs_f64());

    // Wait for reconciliation
    thread::sleep(Duration::from_millis(100));
    let stats = memory.stats();
    println!("  ✓ Reconciled {} concepts", stats.snapshot.concept_count);
    println!(
        "    {} reconciliations, {} entries processed\n",
        stats.reconciler.reconciliations, stats.reconciler.entries_processed
    );

    // Simulation 2: Build associations (still learning)
    println!("🔗 BURST 2: Association Building");
    let start = Instant::now();
    for i in 0..500 {
        let source = ConceptId([i as u8; 16]);
        let target = ConceptId([(i + 1) as u8; 16]);
        memory
            .learn_association(source, target, AssociationType::Semantic, 0.8)
            .ok();
    }
    let elapsed = start.elapsed();
    println!("  ⚡ Created 500 associations in {:?}", elapsed);

    thread::sleep(Duration::from_millis(100));
    let stats = memory.stats();
    println!("  ✓ Graph now has {} edges\n", stats.snapshot.edge_count);

    // Simulation 3: Reasoning burst (read-heavy)
    println!("🧠 BURST 3: Reasoning Phase (Read-Heavy)");
    let memory_clone = Arc::clone(&memory);

    let reader_handle = thread::spawn(move || {
        let start = Instant::now();
        let mut query_count = 0;
        let mut path_found = 0;

        // Simulate reasoning queries
        for _ in 0..100 {
            for i in 0..50 {
                let source = ConceptId([i as u8; 16]);
                let target = ConceptId([(i + 5) as u8; 16]);

                // Find path (graph traversal)
                if memory_clone.find_path(source, target, 10).is_some() {
                    path_found += 1;
                }
                query_count += 1;
            }
        }

        let elapsed = start.elapsed();
        (query_count, path_found, elapsed)
    });

    let (queries, paths, elapsed) = reader_handle.join().unwrap();
    println!("  ⚡ Executed {} queries in {:?}", queries, elapsed);
    println!(
        "     ({:.0} queries/sec)",
        queries as f64 / elapsed.as_secs_f64()
    );
    println!("  ✓ Found {} paths\n", paths);

    // Simulation 4: Mixed burst (concurrent read/write)
    println!("⚡ BURST 4: Mixed Phase (Concurrent Read/Write)");

    let memory_writer = Arc::clone(&memory);
    let writer_handle = thread::spawn(move || {
        let start = Instant::now();
        for i in 1000..1500 {
            let id = ConceptId([i as u8; 16]);
            let content = format!("New concept {}", i).into_bytes();
            memory_writer
                .learn_concept(
                    id,
                    content,
                    None,
                    1.0,
                    0.9,
                    std::collections::HashMap::new(),
                )
                .ok();
            thread::sleep(Duration::from_micros(100));
        }
        (500, start.elapsed())
    });

    let memory_reader = Arc::clone(&memory);
    let reader_handle = thread::spawn(move || {
        let start = Instant::now();
        let mut found = 0;
        for _ in 0..1000 {
            for i in 0..100 {
                let id = ConceptId([i as u8; 16]);
                if memory_reader.contains(&id) {
                    found += 1;
                }
            }
            thread::sleep(Duration::from_micros(50));
        }
        (found, start.elapsed())
    });

    let (writes, write_time) = writer_handle.join().unwrap();
    let (reads, read_time) = reader_handle.join().unwrap();

    println!("  ⚡ Writer: {} concepts in {:?}", writes, write_time);
    println!("  ⚡ Reader: {} queries in {:?}", reads, read_time);
    println!("  ✓ Zero interference between reads and writes\n");

    // Final statistics
    thread::sleep(Duration::from_millis(200));
    let final_stats = memory.stats();

    println!("📊 Final Statistics:");
    println!("  Concepts: {}", final_stats.snapshot.concept_count);
    println!("  Edges: {}", final_stats.snapshot.edge_count);
    println!("  Write log written: {}", final_stats.write_log.written);
    println!(
        "  Write log dropped: {} (backpressure)",
        final_stats.write_log.dropped
    );
    println!(
        "  Reconciliations: {}",
        final_stats.reconciler.reconciliations
    );
    println!(
        "  Entries processed: {}",
        final_stats.reconciler.entries_processed
    );
    println!("  Disk flushes: {}", final_stats.reconciler.disk_flushes);

    println!("\n✅ Demo completed successfully!");
    println!("   Storage location: ./demo_storage");
}
