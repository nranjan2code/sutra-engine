/// Advanced Features Examples
/// 
/// This file demonstrates usage of production-grade features:
/// - Flash Attention for long sequences
/// - Model Distillation
/// - Multi-GPU distributed inference
/// - Streaming embeddings

use sutra_embedder::*;
use anyhow::Result;

/// Example 1: Flash Attention for long documents
#[tokio::main]
async fn flash_attention_example() -> Result<()> {
    println!("=== Flash Attention Example ===\n");
    
    // Configure embedder with Flash Attention
    let mut config = EmbedderConfig::from_name("high-quality")?;
    config.use_flash_attention = true;
    config.max_sequence_length = 2048; // Long sequence support
    
    let mut embedder = Embedder::new(config)?;
    
    // Long document (>512 tokens)
    let long_document = "Lorem ipsum dolor sit amet...".repeat(100); // ~2000 tokens
    
    let embedding = embedder.embed(&long_document)?;
    
    println!("Generated embedding for {} character document", long_document.len());
    println!("Embedding dimension: {}", embedding.len());
    
    // Flash Attention automatically applied for sequences >512 tokens
    
    Ok(())
}

/// Example 2: Sliding Window Attention (alternative to Flash Attention)
fn sliding_window_example() -> Result<()> {
    println!("\n=== Sliding Window Attention Example ===\n");
    
    use sutra_embedder::flash_attention::{SlidingWindowAttention, AggregationMethod};
    
    // Create sliding window processor
    let window = SlidingWindowAttention::new(512, 128); // 512 tokens, 128 overlap
    
    // Very long document
    let long_text = "This is a very long document...".repeat(200);
    let tokens: Vec<i64> = (0..3000).collect(); // Simulate 3000 tokens
    
    // Split into windows
    let windows = window.create_windows(&tokens);
    println!("Split {} tokens into {} windows", tokens.len(), windows.len());
    
    // Process each window with embedder (simulated)
    let embeddings = vec![
        vec![1.0; 768],
        vec![2.0; 768],
        vec![3.0; 768],
    ];
    
    // Aggregate embeddings
    let final_embedding = window.aggregate_embeddings(embeddings, AggregationMethod::Weighted);
    println!("Final aggregated embedding dimension: {}", final_embedding.len());
    
    Ok(())
}

/// Example 3: Model Distillation - Create custom smaller models
#[tokio::main]
async fn model_distillation_example() -> Result<()> {
    println!("\n=== Model Distillation Example ===\n");
    
    // Configure distillation: 768D teacher -> 384D student
    let distill_config = DistillationConfig {
        teacher_dim: 768,
        student_dim: 384,
        temperature: 2.0,
        alpha: 0.5,
        num_iterations: 1000,
        batch_size: 32,
        learning_rate: 5e-4,
        use_cosine_loss: true,
        output_format: ModelFormat::ONNX,
    };
    
    let mut trainer = DistillationTrainer::new(distill_config);
    
    // Collect training data
    let training_texts = vec![
        "Machine learning is a subset of artificial intelligence.".to_string(),
        "Deep learning models require large amounts of data.".to_string(),
        "Natural language processing enables computers to understand text.".to_string(),
        // ... more training examples
    ];
    
    // Create teacher embedder (large model)
    let teacher_config = EmbedderConfig::from_name("high-quality")?; // 768D
    let mut teacher = Embedder::new(teacher_config)?;
    
    // Step 1: Collect teacher embeddings
    println!("Collecting teacher embeddings from {} samples...", training_texts.len());
    trainer.collect_teacher_embeddings(training_texts.clone(), &mut teacher)?;
    
    // Step 2: Train projection matrix
    println!("Training projection matrix (768D -> 384D)...");
    trainer.train_projection()?;
    
    // Step 3: Evaluate on test set
    let test_texts = vec![
        "Neural networks consist of interconnected layers.".to_string(),
        "Transformers revolutionized natural language processing.".to_string(),
    ];
    
    println!("Evaluating distilled model...");
    let metrics = trainer.evaluate(test_texts, &mut teacher)?;
    println!("{}", metrics);
    
    // Step 4: Export distilled model
    println!("Exporting distilled model...");
    trainer.export_model(std::path::Path::new("models/distilled_384d.onnx"))?;
    
    println!("\nDistillation complete! Model saved to models/distilled_384d.onnx");
    println!("Dimension reduction: {:.2}x", metrics.dimension_reduction);
    println!("Quality preserved: {:.1}%", metrics.cosine_similarity * 100.0);
    
    Ok(())
}

/// Example 4: Multi-GPU Distributed Inference
#[tokio::main]
async fn multi_gpu_example() -> Result<()> {
    println!("\n=== Multi-GPU Distributed Inference Example ===\n");
    
    // Configure multi-GPU pool
    let gpu_config = MultiGPUConfig {
        device_ids: vec![], // Auto-detect all GPUs
        load_balancing: LoadBalancingStrategy::LeastLoaded,
        max_concurrent_per_gpu: 4,
        enable_health_checks: true,
        health_check_interval_secs: 30,
        retry_on_failure: true,
        max_retries: 3,
    };
    
    // Create embedder config
    let embedder_config = EmbedderConfig::from_name("efficient")?;
    
    // Initialize multi-GPU pool
    println!("Initializing multi-GPU pool...");
    let pool = MultiGPUPool::new(gpu_config, embedder_config).await?;
    
    // Prepare large batch of texts
    let texts: Vec<String> = (0..1000)
        .map(|i| format!("Document {} with embedding content", i))
        .collect();
    
    println!("Processing {} texts across multiple GPUs...", texts.len());
    
    let start = std::time::Instant::now();
    let embeddings = pool.embed_batch_distributed(texts).await?;
    let elapsed = start.elapsed();
    
    println!("\nResults:");
    println!("  Embeddings generated: {}", embeddings.len());
    println!("  Total time: {:.2}s", elapsed.as_secs_f32());
    println!("  Throughput: {:.1} embeddings/sec", embeddings.len() as f32 / elapsed.as_secs_f32());
    
    // Get statistics
    let stats = pool.get_stats().await;
    println!("\n{}", stats);
    
    Ok(())
}

/// Example 5: Streaming Embeddings for Real-time Applications
#[tokio::main]
async fn streaming_example() -> Result<()> {
    println!("\n=== Streaming Embeddings Example ===\n");
    
    // Configure streaming
    let stream_config = StreamingConfig {
        buffer_size: 100,
        chunk_size: 512,
        chunk_overlap: 64,
        batch_size: 8,
        timeout_secs: 30,
        auto_batch: true,
        max_latency_ms: 100, // 100ms real-time target
    };
    
    let embedder_config = EmbedderConfig::from_name("efficient")?;
    
    // Create streaming embedder
    println!("Initializing streaming embedder...");
    let streaming = StreamingEmbedder::new(stream_config, embedder_config)?;
    
    // Example 1: Single text streaming
    println!("\nStreaming single text...");
    let text = "Real-time embedding generation for streaming applications";
    let embedding = streaming.embed_stream(text.to_string()).await?;
    println!("Generated embedding with {} dimensions", embedding.len());
    
    // Example 2: Multiple texts (simulating real-time stream)
    println!("\nStreaming multiple texts...");
    let texts = vec![
        "First streaming message",
        "Second streaming message",
        "Third streaming message",
    ];
    
    let mut embeddings = Vec::new();
    for text in texts {
        let embedding = streaming.embed_stream(text.to_string()).await?;
        embeddings.push(embedding);
        println!("  Processed: {}", text);
    }
    
    println!("\nGenerated {} embeddings", embeddings.len());
    
    // Example 3: Long document chunking
    println!("\nProcessing long document with chunking...");
    let long_text = "This is a very long document that needs to be processed in chunks...".repeat(50);
    
    use futures::StreamExt;
    let mut chunk_stream = streaming.embed_chunks_stream(long_text).await?;
    
    let mut chunk_count = 0;
    while let Some(result) = chunk_stream.next().await {
        match result {
            Ok(embedding) => {
                chunk_count += 1;
                println!("  Chunk {} embedded ({} dims)", chunk_count, embedding.len());
            }
            Err(e) => {
                eprintln!("  Error processing chunk: {}", e);
            }
        }
    }
    
    println!("\nProcessed {} chunks", chunk_count);
    
    // Get streaming statistics
    let stats = streaming.get_stats().await;
    println!("\n{}", stats);
    
    Ok(())
}

/// Example 6: Combined Advanced Features
#[tokio::main]
async fn combined_example() -> Result<()> {
    println!("\n=== Combined Advanced Features Example ===\n");
    
    // Scenario: High-throughput real-time embedding service
    // - Multi-GPU for parallel processing
    // - Streaming for real-time requests
    // - Flash Attention for long documents
    
    println!("Initializing advanced embedding service...");
    
    // 1. Multi-GPU setup
    let gpu_config = MultiGPUConfig::default();
    let embedder_config = EmbedderConfig {
        name: "production".to_string(),
        dimensions: 768,
        max_sequence_length: 2048, // Support long documents
        quantization: QuantizationType::Int8,
        batch_size: 16,
        matryoshka_dims: Some(vec![768, 512, 384, 256]),
        binary_quantization: false,
        model_id: Some("bge-base-en-v1.5".to_string()),
        target_dimension: Some(768),
        use_fp16: true,
        use_fused_ops: true,
        use_flash_attention: true, // Enable for long sequences
    };
    
    let pool = MultiGPUPool::new(gpu_config, embedder_config.clone()).await?;
    
    // 2. Streaming setup for real-time processing
    let stream_config = StreamingConfig {
        buffer_size: 200,
        batch_size: 16,
        max_latency_ms: 50, // Ultra-low latency
        ..Default::default()
    };
    
    let streaming = StreamingEmbedder::new(stream_config, embedder_config)?;
    
    println!("\nProduction service ready!");
    println!("  Multi-GPU: Enabled");
    println!("  Streaming: Enabled");
    println!("  Flash Attention: Enabled for sequences >512 tokens");
    println!("  Target Latency: <50ms");
    
    // Simulate real-time requests
    println!("\nProcessing real-time requests...");
    
    let requests = vec![
        "Short query",
        "Medium length document with more content".repeat(5),
        "Very long document that will trigger Flash Attention...".repeat(100),
    ];
    
    for (i, request) in requests.iter().enumerate() {
        let start = std::time::Instant::now();
        let embedding = streaming.embed_stream(request.clone()).await?;
        let latency = start.elapsed();
        
        println!("  Request {}: {} chars, {} dims, {:.2}ms",
            i + 1,
            request.len(),
            embedding.len(),
            latency.as_millis()
        );
    }
    
    // Get final statistics
    let stream_stats = streaming.get_stats().await;
    let gpu_stats = pool.get_stats().await;
    
    println!("\n=== Performance Summary ===");
    println!("{}", stream_stats);
    println!("\n{}", gpu_stats);
    
    Ok(())
}

fn main() {
    println!("Sutra-Embedder Advanced Features Examples\n");
    println!("Choose an example to run:");
    println!("1. Flash Attention for Long Sequences");
    println!("2. Sliding Window Attention");
    println!("3. Model Distillation");
    println!("4. Multi-GPU Distributed Inference");
    println!("5. Streaming Embeddings");
    println!("6. Combined Advanced Features");
    
    // Note: These examples require async runtime
    // Run with: cargo run --example advanced_features
}
