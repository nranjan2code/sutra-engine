/// Integration tests for end-to-end workflows
use sutra_core::{Tensor, DType, ops};
use sutra_tokenizer::{Tokenizer, BpeTokenizer, BpeConfig, VocabBuilder};
use sutra_quantize::{AwqQuantizer, AwqConfig};
use sutra_loader::{ModelDownloader, DownloadConfig, ModelRegistry};
use sutra_rwkv::{RwkvModel, RwkvConfig};
use sutra_mamba::{MambaModel, MambaConfig};

#[test]
fn test_rwkv_end_to_end_workflow() {
    // Test complete RWKV workflow: tokenize -> forward -> decode
    
    // Create tokenizer
    let mut vocab = VocabBuilder::new()
        .with_standard_special_tokens()
        .build();
    
    for word in &["hello", "world", "test", "rwkv", "model"] {
        vocab.add_token(word.to_string());
    }
    
    let config = BpeConfig {
        vocab,
        merges: Vec::new(),
        unk_token: "[UNK]".to_string(),
        byte_level: false,
    };
    
    let tokenizer = BpeTokenizer::new(config);
    let tokenizer = Tokenizer::Bpe(tokenizer);
    
    // Create RWKV model
    let rwkv_config = sutra_rwkv::RwkvConfig::new(2, 64, 100); // Small model for testing
    let rwkv_model = sutra_rwkv::RwkvModel::new(rwkv_config).unwrap();
    
    // Test workflow
    let text = "hello world";
    let encoding = tokenizer.encode(text).unwrap();
    assert!(!encoding.ids.is_empty());
    
    // Forward pass through RWKV
    let (logits, _state) = rwkv_model.forward(&encoding.ids, None).unwrap();
    assert_eq!(logits.len(), 100); // vocab_size
    
    // Verify logits are not uniform (real computation happened)
    let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min_logit = logits.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(max_logit > min_logit, "Logits should not be uniform");
    
    // Decode result
    let decoded = tokenizer.decode(&encoding.ids).unwrap();
    assert!(!decoded.is_empty());
    
    println!("✓ RWKV end-to-end workflow completed successfully!");
}

#[test]
fn test_mamba_quantization_workflow() {
    // Test Mamba model creation and quantization
    
    // Create Mamba model
    let mamba_config = sutra_mamba::MambaConfig::new(2, 64, 100); // Small model for testing
    let mamba_model = sutra_mamba::MambaModel::new(mamba_config).unwrap();
    
    // Create test weights for quantization
    let data: Vec<f32> = (0..2048).map(|i| (i as f32 / 2048.0).sin()).collect();
    let weights = Tensor::from_slice(&data, &[32, 64], DType::F32).unwrap();
    
    // Test quantization with real compression measurement
    let quant_config = AwqConfig {
        bits: 4,
        group_size: 16,
        n_samples: 128,
        zero_point: true,
    };
    
    let quantizer = AwqQuantizer::new(quant_config);
    let quantized = quantizer.quantize(&weights, None).unwrap();
    
    // Verify real compression occurred
    let original_size = weights.memory_usage();
    let quantized_size = quantized.memory_usage();
    let compression_ratio = original_size as f64 / quantized_size as f64;
    
    assert!(compression_ratio > 2.0, "Expected >2x compression, got {:.2}x", compression_ratio);
    assert!(compression_ratio < 8.0, "Compression too high {:.2}x, likely error", compression_ratio);
    assert_eq!(quantized.bits, 4);
    
    // Test inference with Mamba
    let test_tokens = vec![1, 5, 10, 20];
    let logits = mamba_model.forward(&test_tokens).unwrap();
    assert_eq!(logits.len(), 100); // vocab_size
    
    // Verify non-zero output (real computation)
    let sum: f32 = logits.iter().map(|x| x.abs()).sum();
    assert!(sum > 0.0, "Mamba output should not be all zeros");
    
    println!("✓ Mamba quantization and inference workflow completed: {:.2}x compression", compression_ratio);
}

#[test]
fn test_model_architecture_detection() {
    use sutra_loader::{ModelDownloader, DownloadConfig, ModelRegistry};
    
    // Test model registry and architecture detection
    let registry = ModelRegistry::with_defaults();
    assert!(registry.list().len() > 0, "Registry should have models");
    
    // Test architecture-specific model queries
    let rwkv_models = registry.list_by_architecture("rwkv");
    let mamba_models = registry.list_by_architecture("mamba"); 
    
    println!("Found {} RWKV models and {} Mamba models", rwkv_models.len(), mamba_models.len());
    
    // Test configuration validation
    let config = DownloadConfig::default();
    assert!(config.cache_dir.exists() || !config.cache_dir.as_os_str().is_empty());
    
    // Test model info structure
    if let Some(model) = registry.list().first() {
        assert!(!model.name.is_empty());
        assert!(model.num_parameters > 0);
        println!("✓ Model info validation passed for: {}", model.name);
    }
    
    println!("✓ Model architecture detection and registry functionality working!");
}

#[test]
fn test_inference_performance_benchmark() {
    use std::time::Instant;
    
    // Test real inference performance with actual model computations
    let rwkv_config = RwkvConfig::new(4, 128, 1000); // Small but realistic
    let rwkv_model = RwkvModel::new(rwkv_config).unwrap();
    
    let mamba_config = MambaConfig::new(4, 128, 1000);
    let mamba_model = MambaModel::new(mamba_config).unwrap();
    
    let test_tokens = vec![1, 50, 100, 200, 300];
    let num_iterations = 50; // Reduced for faster test
    
    // Benchmark RWKV
    let rwkv_start = Instant::now();
    let mut rwkv_state = None;
    for _ in 0..num_iterations {
        let (_, state) = rwkv_model.forward(&test_tokens, rwkv_state).unwrap();
        rwkv_state = Some(state);
    }
    let rwkv_time = rwkv_start.elapsed();
    let rwkv_tokens_per_sec = (test_tokens.len() * num_iterations) as f64 / rwkv_time.as_secs_f64();
    
    // Benchmark Mamba  
    let mamba_start = Instant::now();
    for _ in 0..num_iterations {
        let _ = mamba_model.forward(&test_tokens).unwrap();
    }
    let mamba_time = mamba_start.elapsed();
    let mamba_tokens_per_sec = (test_tokens.len() * num_iterations) as f64 / mamba_time.as_secs_f64();
    
    println!("Performance benchmark results:");
    println!("  RWKV: {:.0} tokens/sec ({:.2}ms per iteration)", rwkv_tokens_per_sec, rwkv_time.as_millis() as f64 / num_iterations as f64);
    println!("  Mamba: {:.0} tokens/sec ({:.2}ms per iteration)", mamba_tokens_per_sec, mamba_time.as_millis() as f64 / num_iterations as f64);
    
    // Performance should be reasonable (not zero or impossibly high)
    assert!(rwkv_tokens_per_sec > 50.0, "RWKV too slow: {:.0} tok/s", rwkv_tokens_per_sec);
    assert!(mamba_tokens_per_sec > 50.0, "Mamba too slow: {:.0} tok/s", mamba_tokens_per_sec);
    assert!(rwkv_tokens_per_sec < 100_000.0, "RWKV suspiciously fast: {:.0} tok/s", rwkv_tokens_per_sec);
    assert!(mamba_tokens_per_sec < 100_000.0, "Mamba suspiciously fast: {:.0} tok/s", mamba_tokens_per_sec);
    
    // Test memory usage
    let rwkv_memory_est = rwkv_config.estimate_memory();
    let mamba_memory_est = mamba_config.estimate_memory();
    
    println!("Memory estimates:");
    println!("  RWKV: {:.2} MB", rwkv_memory_est as f64 / (1024.0 * 1024.0));
    println!("  Mamba: {:.2} MB", mamba_memory_est as f64 / (1024.0 * 1024.0));
    
    // Both should fit in reasonable memory
    assert!(rwkv_memory_est < 1_000_000_000, "RWKV memory too high: {} bytes", rwkv_memory_est); // < 1GB
    assert!(mamba_memory_est < 1_000_000_000, "Mamba memory too high: {} bytes", mamba_memory_est); // < 1GB
    
    println!("✓ Real inference performance benchmark completed!");
}

#[test]
fn test_tokenizer_model_integration() {
    // Test real tokenizer integration with model embeddings
    
    // Create comprehensive vocabulary
    let mut vocab = VocabBuilder::new()
        .with_standard_special_tokens()
        .build();
        
    let test_words = ["the", "and", "to", "of", "a", "in", "for", "is", "on", "that", 
                     "by", "this", "with", "from", "they", "we", "she", "or", "an", "are"];
    
    for word in test_words {
        vocab.add_token(word.to_string());
    }
    
    let tokenizer_config = BpeConfig {
        vocab,
        merges: Vec::new(),
        unk_token: "[UNK]".to_string(),
        byte_level: false,
    };
    
    let tokenizer = BpeTokenizer::new(tokenizer_config);
    let tokenizer = Tokenizer::Bpe(tokenizer);
    
    // Create model with matching vocab size
    let vocab_size = 100;
    let rwkv_config = RwkvConfig::new(2, 64, vocab_size);
    let rwkv_model = RwkvModel::new(rwkv_config).unwrap();
    
    // Test complete pipeline
    let test_sentences = [
        "the quick brown",
        "and they are",
        "this is a test",
    ];
    
    for sentence in test_sentences {
        // Tokenize
        let encoding = tokenizer.encode(sentence).unwrap();
        assert!(!encoding.ids.is_empty(), "Encoding should not be empty for: {}", sentence);
        
        // Ensure tokens are within vocab range
        let valid_tokens: Vec<usize> = encoding.ids.iter()
            .map(|&id| id.min(vocab_size - 1))
            .collect();
        
        // Forward pass
        let (logits, _) = rwkv_model.forward(&valid_tokens, None).unwrap();
        assert_eq!(logits.len(), vocab_size);
        
        // Check for meaningful output (not all same value)
        let max_logit = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let min_logit = logits.iter().copied().fold(f32::INFINITY, f32::min);
        assert!(max_logit > min_logit, "Model should produce varied logits for: {}", sentence);
        
        // Decode tokens back
        let decoded = tokenizer.decode(&encoding.ids).unwrap();
        assert!(!decoded.is_empty(), "Decoded text should not be empty");
        
        println!("  '{}' -> {} tokens -> {} logits -> '{}'", 
                sentence, valid_tokens.len(), logits.len(), decoded);
    }
    
    println!("✓ Tokenizer-model integration pipeline working!");
}#[test]
fn test_real_quantization_validation() {
    use std::time::Instant;
    
    // Test real quantization with different weight patterns
    
    // 1. Random weights (typical model weights)
    let random_data: Vec<f32> = (0..4096).map(|i| 
        ((i * 17 + 1) as f32 / 4096.0).sin() * 0.5
    ).collect();
    let random_weights = Tensor::from_slice(&random_data, &[64, 64], DType::F32).unwrap();
    
    // 2. Structured weights (like attention patterns)  
    let structured_data: Vec<f32> = (0..2048).map(|i| {
        let row = i / 64;
        let col = i % 64;
        if row == col { 1.0 } else { 0.1 * (i as f32).sin() }
    }).collect();
    let structured_weights = Tensor::from_slice(&structured_data, &[32, 64], DType::F32).unwrap();
    
    // 3. Sparse weights (like some FFN layers)
    let sparse_data: Vec<f32> = (0..1024).map(|i| 
        if i % 4 == 0 { (i as f32 / 100.0).tanh() } else { 0.0 }
    ).collect();
    let sparse_weights = Tensor::from_slice(&sparse_data, &[32, 32], DType::F32).unwrap();
    
    let quantizer = AwqQuantizer::new(AwqConfig {
        bits: 4,
        group_size: 32,
        n_samples: 256,
        zero_point: true,
    });
    
    // Test quantization on different weight patterns
    let test_cases = [
        ("Random", &random_weights),
        ("Structured", &structured_weights),
        ("Sparse", &sparse_weights),
    ];
    
    for (name, weights) in test_cases.iter() {
        let start = Instant::now();
        let quantized = quantizer.quantize(weights, None).unwrap();
        let quantize_time = start.elapsed();
        
        let original_size = weights.memory_usage();
        let quantized_size = quantized.memory_usage();
        let compression_ratio = quantized.compression_ratio();
        let calculated_ratio = original_size as f32 / quantized_size as f32;
        
        println!("{} weights:", name);
        println!("  Original: {} bytes", original_size);
        println!("  Quantized: {} bytes", quantized_size);
        println!("  Compression: {:.2}x", compression_ratio);
        println!("  Time: {:?}", quantize_time);
        
        // Validate quantization properties
        assert_eq!(quantized.bits, 4);
        assert_eq!(quantized.group_size, 32);
        assert!(compression_ratio > 2.0, "{} compression too low: {:.2}x", name, compression_ratio);
        assert!(compression_ratio < 10.0, "{} compression suspiciously high: {:.2}x", name, compression_ratio);
        assert!((compression_ratio - calculated_ratio).abs() < 0.01, "Compression calculation mismatch");
        assert!(quantized_size < original_size, "Quantized size should be smaller");
        assert!(quantized_size > 0, "Quantized size should be positive");
    }
    
    println!("✓ Real quantization validation completed successfully!");
}