use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sutra_embedder::embedder::{Embedder, EmbedderConfig};

fn benchmark_embedding(c: &mut Criterion) {
    let config = EmbedderConfig::from_name("efficient").unwrap();
    let mut embedder = Embedder::new(config).unwrap();
    let test_text = "This is a test sentence for benchmarking embedding performance.";

    c.bench_function("embed_efficient", |b| {
        b.iter(|| embedder.embed(black_box(test_text)))
    });

    let config_hq = EmbedderConfig::from_name("high-quality").unwrap();
    let mut embedder_hq = Embedder::new(config_hq).unwrap();

    c.bench_function("embed_high_quality", |b| {
        b.iter(|| embedder_hq.embed(black_box(test_text)))
    });

    let config_ue = EmbedderConfig::from_name("ultra-efficient").unwrap();
    let mut embedder_ue = Embedder::new(config_ue).unwrap();

    c.bench_function("embed_ultra_efficient", |b| {
        b.iter(|| embedder_ue.embed(black_box(test_text)))
    });
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    
    let config = EmbedderConfig::from_name("efficient").unwrap();
    let mut embedder = Embedder::new(config).unwrap();
    
    let test_texts = vec![
        "Machine learning enables computers to learn from data.",
        "Natural language processing helps computers understand human language.",
        "Deep learning uses neural networks with multiple layers.",
        "Embeddings capture semantic meaning in vector space.",
        "Transformers revolutionized the field of NLP.",
        "Attention mechanisms help models focus on relevant information.",
        "Pre-trained models can be fine-tuned for specific tasks.",
        "Transfer learning allows knowledge to be reused across tasks.",
    ];

    for batch_size in [1, 2, 4, 8].iter() {
        let texts: Vec<String> = test_texts[..*batch_size].iter().map(|s| s.to_string()).collect();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &texts,
            |b, texts| {
                b.iter(|| embedder.embed_batch(black_box(texts)))
            },
        );
    }
    
    group.finish();
}

fn benchmark_simd_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_optimizations");
    
    // Test different embedding dimensions to show SIMD benefits
    for dims in [256, 384, 768, 1024].iter() {
        let config = EmbedderConfig::for_dimension(*dims, "auto").unwrap();
        let mut embedder = Embedder::new(config).unwrap();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(dims),
            dims,
            |b, _| {
                b.iter(|| {
                    embedder.embed(black_box("SIMD-optimized pooling and normalization test"))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_embedding,
    benchmark_batch_processing,
    benchmark_simd_optimizations
);
criterion_main!(benches);
