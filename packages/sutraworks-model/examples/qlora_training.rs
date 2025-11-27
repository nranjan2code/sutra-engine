/// Example: QLoRA Fine-Tuning
///
/// Demonstrates parameter-efficient fine-tuning with quantized base model
use sutra_peft::{LoraConfig, QLoraConfig, QLoraLayer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== QLoRA Fine-Tuning Demo ===\n");

    // Model dimensions
    let hidden_dim = 4096;

    // Configure LoRA
    let lora_config = LoraConfig {
        rank: 8,
        alpha: 16.0,
        dropout: 0.1,
        target_modules: vec!["q_proj".into(), "v_proj".into()],
    };

    // Configure QLoRA
    let qlora_config = QLoraConfig {
        lora: lora_config,
        quant_bits: 4,
        double_quant: true,
    };

    println!("QLoRA Configuration:");
    println!("  LoRA rank: {}", qlora_config.lora.rank);
    println!("  Scaling (alpha/rank): {:.2}", qlora_config.lora.scaling());
    println!("  Quantization: {}-bit", qlora_config.quant_bits);

    // Create QLoRA layer
    let layer = QLoraLayer::new(hidden_dim, hidden_dim, qlora_config)?;

    println!("\nLayer statistics:");
    println!("  Trainable parameters: {}", layer.trainable_parameters());
    println!(
        "  Memory usage: {:.2} MB",
        layer.memory_usage() as f64 / 1_048_576.0
    );

    // Memory estimation for full model
    println!("\n=== Memory Estimation ===");
    let model_params = 3_000_000_000u64; // 3B model
    let rank = 8u64;
    let num_layers = 32u64;

    // Simplified memory estimation
    let base_model_gb = (model_params * 4 / 8) as f64 / 1e9; // 4-bit quantized
    let lora_params = num_layers * hidden_dim as u64 * rank * 2; // A and B matrices
    let adapters_gb = (lora_params * 4) as f64 / 1e9; // f32
    let optimizer_gb = adapters_gb * 2.0; // Adam states
    let total_gb = base_model_gb + adapters_gb + optimizer_gb;

    println!("Fine-tuning 3B parameter model:");
    println!("  Base model (4-bit): {:.2} GB", base_model_gb);
    println!("  LoRA adapters: {:.2} GB", adapters_gb);
    println!("  Optimizer states: {:.2} GB", optimizer_gb);
    println!("  Total: {:.2} GB", total_gb);

    if total_gb < 16.0 {
        println!("\n✓ Fits in 16GB MacBook Air!");
    } else {
        println!("\n✗ Requires more than 16GB");
    }

    println!("\n=== Parameter Efficiency ===");
    let full_params = hidden_dim * hidden_dim;
    let lora_params = layer.trainable_parameters();
    let reduction = full_params as f32 / lora_params as f32;

    println!("Full fine-tuning: {} parameters", full_params);
    println!("QLoRA: {} parameters", lora_params);
    println!("Reduction: {:.1}x fewer parameters", reduction);

    Ok(())
}
