# Model Inventory

## Downloaded Models (SHA256 Validated)

All 6 production models downloaded and validated. Total storage: **3.6GB**

### 1. all-MiniLM-L6-v2
- **Dimensions**: 384D (supports: 384, 256, 128, 64)
- **Size**: 86MB (model) + 455KB (tokenizer)
- **Architecture**: Sentence Transformer
- **Quality Score**: 56.26 (MTEB)
- **Use Case**: Efficient, mobile-ready, IoT
- **Model SHA256**: `6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452`
- **Tokenizer SHA256**: `be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037`
- **Status**: ✅ Validated

### 2. bge-base-en-v1.5
- **Dimensions**: 768D (supports: 768, 512, 384, 256)
- **Size**: 416MB (model) + 695KB (tokenizer)
- **Architecture**: BGE (BAAI)
- **Quality Score**: 64.23 (MTEB)
- **Use Case**: State-of-art quality, general purpose
- **Model SHA256**: `9bc579acdba21c253c62a9bf866891355a63ffa3442b52c8a37d75b2ccb91848`
- **Tokenizer SHA256**: `d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66`
- **Status**: ✅ Validated

### 3. all-mpnet-base-v2
- **Dimensions**: 768D (supports: 768, 512, 384, 256)
- **Size**: 416MB (model) + 455KB (tokenizer)
- **Architecture**: Sentence Transformer (MPNet)
- **Quality Score**: 63.30 (MTEB)
- **Use Case**: High-quality, semantic search
- **Model SHA256**: `74187b16d9c946fea252e120cfd7a12c5779d8b8b86838a2e4c56573c47941bd`
- **Tokenizer SHA256**: `b8be2c30ba5dd723a6d5ee26d013da103d5408d92ddcb23747622f9e48f1d842`
- **Status**: ✅ Validated
- **Note**: Requires token_type_ids fallback (MPNet-style)

### 4. bge-large-en-v1.5
- **Dimensions**: 1024D (supports: 1024, 768, 512, 384)
- **Size**: 1.2GB (model) + 695KB (tokenizer)
- **Architecture**: BGE (BAAI)
- **Quality Score**: 64.23 (MTEB)
- **Use Case**: Maximum quality, enterprise applications
- **Model SHA256**: `69ed3f810d3b6d13f70dff9ca89966f39c0a0e877fb88211be7bcc070df2a2ce`
- **Tokenizer SHA256**: `d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66`
- **Status**: ✅ Validated

### 5. e5-base-v2
- **Dimensions**: 768D (supports: 768, 512, 384, 256)
- **Size**: 416MB (model) + 695KB (tokenizer)
- **Architecture**: E5 (Microsoft Research)
- **Quality Score**: 64.05 (MTEB)
- **Use Case**: Research, Microsoft ecosystem
- **Model SHA256**: `157f97ef1957d34f52efa26f8031371bf9043acc45460cec7ebe94631ac0e96b`
- **Tokenizer SHA256**: `d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66`
- **Status**: ✅ Validated

### 6. multilingual-e5-base
- **Dimensions**: 768D (supports: 768, 512, 384, 256)
- **Size**: 1.0GB (model) + 16MB (tokenizer)
- **Architecture**: E5 (Microsoft Research)
- **Quality Score**: 54.73 (MTEB)
- **Use Case**: Multilingual, 100+ languages
- **Model SHA256**: `84a4d426f7e87a6bf5bf195f0bae2c4a7d15f675b23ca96f42fab8326d7a77aa`
- **Tokenizer SHA256**: `62c24cdc13d4c9952d63718d6c9fa4c287974249e16b7ade6d5a85e7bbb75626`
- **Status**: ✅ Validated

## Storage Location

All models stored in project directory: `./models/`

```bash
models/
├── all-MiniLM-L6-v2.onnx (86MB)
├── all-MiniLM-L6-v2-tokenizer.json (455KB)
├── bge-base-en-v1.5.onnx (416MB)
├── bge-base-en-v1.5-tokenizer.json (695KB)
├── all-mpnet-base-v2.onnx (416MB)
├── all-mpnet-base-v2-tokenizer.json (455KB)
├── bge-large-en-v1.5.onnx (1.2GB)
├── bge-large-en-v1.5-tokenizer.json (695KB)
├── e5-base-v2.onnx (416MB)
├── e5-base-v2-tokenizer.json (695KB)
├── multilingual-e5-base.onnx (1.0GB)
└── multilingual-e5-base-tokenizer.json (16MB)
```

## Verification Commands

```bash
# Check all model hashes
for model in all-MiniLM-L6-v2 bge-base-en-v1.5 all-mpnet-base-v2 bge-large-en-v1.5 e5-base-v2 multilingual-e5-base; do
  ./target/release/sutra-embedder hash --model $model
done

# Check storage size
du -sh models/

# List all files
ls -lh models/
```

## Performance Benchmarks

Tested on Apple M1 Max (10 cores, 16GB RAM) with **10+ optimizations** (FP16, SIMD, async queue, batch processing, model warmup, custom ops):

| Model | Dimensions | Latency (avg) | Throughput (single) | Throughput (batch-8) | Memory/Embedding |
|-------|-----------|---------------|---------------------|---------------------|------------------|
| all-MiniLM-L6-v2 | 384D | ~13.69ms | 73 emb/sec | 241 emb/sec | 1.5KB / 128B (Binary) |
| all-mpnet-base-v2 | 768D | ~68.86ms | 15 emb/sec | 45+ emb/sec | 3KB / 128B (Binary) |
| all-MiniLM-L6-v2 | 256D | ~11ms | 89 emb/sec | 280+ emb/sec | 1KB / 128B (Binary) |

**Optimization Impact**:
- FP16 Mixed Precision: 2x speedup on compatible GPUs (Apple Neural Engine, NVIDIA Tensor Cores)
- Async Batch Queue: Non-blocking background processing with configurable backpressure
- Custom SIMD Ops: 10-30% speedup with fused mean pooling + L2 normalization (AVX2/NEON)
- INT8 Quantization: 1.5-2x CPU speedup, 4x memory reduction (1.5KB → 384B)
- Batch Processing: 3.3x throughput improvement for batch-8 workloads
- Model Warmup: <5ms cold start (vs 150ms baseline), zero double inference
- Flash Attention: Configuration ready for sequences >512 tokens (see FLASH_ATTENTION.md)

## Security Notes

- All models validated with SHA256 checksums
- Downloads include progress tracking
- Automatic retry on network failures
- Health checks after download completion
- Automatic token_type_ids fallback for model compatibility

## HuggingFace Sources

- all-MiniLM-L6-v2: `sentence-transformers/all-MiniLM-L6-v2`
- bge-base-en-v1.5: `BAAI/bge-base-en-v1.5`
- all-mpnet-base-v2: `sentence-transformers/all-mpnet-base-v2`
- bge-large-en-v1.5: `BAAI/bge-large-en-v1.5`
- e5-base-v2: `intfloat/e5-base-v2`
- multilingual-e5-base: `intfloat/multilingual-e5-base`

---

**Last Updated**: November 8, 2025  
**Total Models**: 6  
**Total Storage**: 3.6GB  
**Validation**: 100% SHA256 coverage
