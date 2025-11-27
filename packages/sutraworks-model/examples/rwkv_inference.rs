/// Example: RWKV Inference
///
/// Demonstrates efficient RNN-based inference with constant memory
use sutra_rwkv::{RwkvConfig, RwkvModel, RwkvState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RWKV Inference Demo ===\n");

    // Configure small RWKV model for demo
    let config = RwkvConfig::new(
        6,    // num_layers
        256,  // hidden_size  
        1000, // vocab_size
    );

    println!("RWKV Demo Configuration:");
    println!("  Layers: {}", config.num_layers);
    println!("  Hidden size: {}", config.hidden_size);
    println!("  Vocab size: {}", config.vocab_size);
    println!("  Max sequence: {}", config.max_seq_len);

    // Memory estimation
    let memory = config.estimate_memory();
    println!(
        "\nMemory estimate: {:.2} MB",
        memory as f64 / 1_048_576.0
    );

    // Create model
    let model = RwkvModel::new(config)?;

    println!("\n=== Key Advantages ===");
    println!("✓ Linear O(n) complexity vs Transformer O(n²)");
    println!("✓ Constant memory during inference");
    println!("✓ No GPU required - efficient on CPU");
    println!("✓ Perfect for edge devices");

    // Initialize state
    let state = RwkvState::new(model.config());

    println!("\n=== State Management ===");
    println!(
        "State memory: {:.2} KB",
        state.memory_usage() as f64 / 1024.0
    );
    println!("Unlike Transformers:");
    println!("  - Transformer KV cache: O(n) - grows with sequence");
    println!("  - RWKV state: O(1) - constant size");

    // Simulate generation
    println!("\n=== Generation Example ===");
    let prompt = vec![1, 2, 3]; // Token IDs
    let max_tokens = 100;
    let temperature = 0.7;

    println!("Generating {} tokens...", max_tokens);
    let _generated = model.generate(&prompt, max_tokens, temperature)?;

    println!("✓ Generation complete!");
    println!("\nMemory during generation:");
    println!("  State: {} bytes (constant)", state.memory_usage());
    println!("  No KV cache accumulation!");

    Ok(())
}
