/// Example: Mamba Inference
///
/// Demonstrates linear-time state space model with 5x throughput vs Transformers
use sutra_mamba::{MambaConfig, MambaModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mamba State Space Model Demo ===\n");

    // Use small demo configuration instead of massive 3B model
    let config = MambaConfig::new(6, 256, 1000); // 6 layers, 256 hidden, 1K vocab

    println!("Mamba Demo Configuration:");
    println!("  Layers: {}", config.num_layers);
    println!("  Hidden size: {}", config.hidden_size);
    println!("  State size: {}", config.state_size);
    println!("  Expand factor: {}x", config.expand_factor);

    // Memory estimation
    let memory = config.estimate_memory();
    println!(
        "\nMemory estimate: {:.2} MB",
        memory as f64 / 1_048_576.0
    );

    // Complexity analysis
    println!("\n=== Complexity Advantage ===");
    let seq_lengths = [512, 1024, 2048, 4096];

    println!("\nThroughput vs Transformer:");
    for &seq_len in &seq_lengths {
        let speedup = config.throughput_multiplier(seq_len);
        println!(
            "  Sequence {}: {:.0}x faster",
            seq_len,
            speedup.min(10000.0)
        );
    }

    println!("\n=== Key Features ===");
    println!("✓ Linear O(n) time complexity");
    println!("✓ 5x higher inference throughput");
    println!("✓ Selective state space mechanism");
    println!("✓ Content-aware processing");

    // Create model
    let model = MambaModel::new(config)?;

    println!("\n=== Model Architecture ===");
    println!("State Space Model with:");
    println!("  - Selective parameters (input-dependent A, B, C)");
    println!("  - Linear scan operation");
    println!("  - Gated projections");
    println!("  - No attention mechanism needed");

    // Simulate inference
    println!("\n=== Inference Example ===");
    let prompt = vec![1, 2, 3, 4, 5];
    let max_tokens = 50;
    let temperature = 0.8;

    println!("Generating {} tokens...", max_tokens);
    let _generated = model.generate(&prompt, max_tokens, temperature)?;

    println!("✓ Generation complete!");

    println!("\n=== Performance Comparison ===");
    println!("Mamba-3B vs Transformer:");
    println!("  - Parameter efficiency: ~2x smaller for same quality");
    println!("  - Speed: 5x faster inference");
    println!("  - Memory: Linear growth vs quadratic");
    println!("  - Perfect for MacBook Air 16GB!");

    Ok(())
}
