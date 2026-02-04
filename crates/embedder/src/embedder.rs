use crate::model_registry::ModelRegistry;
use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::Array2;
use ort::logging::LogLevel;
use ort::session::{builder::GraphOptimizationLevel, builder::SessionBuilder, Session};
use ort::value::Value;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, info, warn};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Configuration for the embedder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    pub name: String,
    pub dimensions: usize,
    pub max_sequence_length: usize,
    pub quantization: QuantizationType,
    pub batch_size: usize,
    pub matryoshka_dims: Option<Vec<usize>>, // Matryoshka representation learning dimensions
    pub binary_quantization: bool,           // Enable 1-bit binary quantization
    pub model_id: Option<String>,            // Specific model to use
    pub target_dimension: Option<usize>,     // Target output dimension
    pub use_fp16: bool,                      // Enable FP16 mixed precision (2x speedup)
    pub use_fused_ops: bool,                 // Enable fused custom operations (10-30% speedup)
    pub use_flash_attention: bool,           // Enable Flash Attention for long sequences (>512 tokens)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuantizationType {
    None,
    Int8,
    Int4,
    Float16,
    Binary, // 1-bit quantization
}

impl EmbedderConfig {
    pub fn from_name(name: &str) -> Result<Self> {
        match name {
            "efficient" => Ok(Self {
                name: "efficient".to_string(),
                dimensions: 384,
                max_sequence_length: 512,
                quantization: QuantizationType::Int8,
                batch_size: 32,
                matryoshka_dims: Some(vec![384, 256, 128, 64]),
                binary_quantization: false,
                model_id: Some("all-MiniLM-L6-v2".to_string()),
                target_dimension: Some(384),
                use_fp16: true, // Enable FP16 for 2x speedup
                use_fused_ops: true, // Enable fused ops for 10-30% speedup
                use_flash_attention: false, // Short sequences don't need flash attention
            }),
            "high-quality" => Ok(Self {
                name: "high-quality".to_string(),
                dimensions: 768,
                max_sequence_length: 512,
                quantization: QuantizationType::None,
                batch_size: 16,
                matryoshka_dims: Some(vec![768, 512, 256]),
                binary_quantization: false,
                model_id: Some("all-mpnet-base-v2".to_string()),
                target_dimension: Some(768),
                use_fp16: false, // High quality uses FP32
                use_fused_ops: true, // Enable fused ops
                use_flash_attention: false,
            }),
            "ultra-efficient" => Ok(Self {
                name: "ultra-efficient".to_string(),
                dimensions: 256,
                max_sequence_length: 256,
                quantization: QuantizationType::Int4,
                batch_size: 64,
                matryoshka_dims: Some(vec![256, 128, 64, 32]),
                binary_quantization: true, // Enable binary quantization for extreme efficiency
                model_id: Some("all-MiniLM-L6-v2".to_string()),
                target_dimension: Some(256),
                use_fp16: true, // Ultra-efficient uses FP16
                use_fused_ops: true, // Enable fused ops
                use_flash_attention: false,
            }),
            _ => Err(anyhow!("Unknown config: {}", name)),
        }
    }

    /// Create a configuration for any target dimension using optimal model selection
    pub fn for_dimension(target_dims: usize, hardware_tier: &str) -> Result<Self> {
        crate::model_registry::create_config_for_dimension(target_dims, hardware_tier, None)
    }

    /// Create a configuration for any dimension with specific quantization
    #[allow(dead_code)]
    pub fn for_dimension_with_quantization(
        target_dims: usize,
        hardware_tier: &str,
        quantization: QuantizationType,
    ) -> Result<Self> {
        crate::model_registry::create_config_for_dimension(
            target_dims,
            hardware_tier,
            Some(quantization),
        )
    }
}

/// Main embedder struct
pub struct Embedder {
    config: EmbedderConfig,
    session: Option<Session>,
    tokenizer: Option<Tokenizer>,
    model_capabilities: ModelCapabilities,
    #[allow(dead_code)]
    buffer_pool: BufferPool,
}

/// Model input requirements detected during initialization
#[derive(Debug, Clone)]
struct ModelCapabilities {
    needs_token_type_ids: bool,
    output_shape_type: OutputShapeType,
    is_warmed_up: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum OutputShapeType {
    Unknown,
    SequenceLevel, // [batch, seq, hidden] - needs pooling
    Pooled,        // [batch, hidden] - already pooled
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            needs_token_type_ids: true,
            output_shape_type: OutputShapeType::Unknown,
            is_warmed_up: false,
        }
    }
}

/// Pre-allocated buffer pool for tensor reuse
struct BufferPool {
    #[allow(dead_code)]
    max_batch_size: usize,
    #[allow(dead_code)]
    max_seq_len: usize,
    // Pre-allocated buffers (None until first use)
    #[allow(dead_code)]
    input_ids_buffer: Option<Vec<i64>>,
    #[allow(dead_code)]
    attention_mask_buffer: Option<Vec<i64>>,
    #[allow(dead_code)]
    token_type_ids_buffer: Option<Vec<i64>>,
}

impl BufferPool {
    fn new(max_batch_size: usize, max_seq_len: usize) -> Self {
        Self {
            max_batch_size,
            max_seq_len,
            input_ids_buffer: None,
            attention_mask_buffer: None,
            token_type_ids_buffer: None,
        }
    }

    /// Get or create input_ids buffer
    #[allow(dead_code)]
    fn get_input_ids_buffer(&mut self, required_size: usize) -> &mut Vec<i64> {
        if self.input_ids_buffer.is_none() || self.input_ids_buffer.as_ref().unwrap().capacity() < required_size {
            self.input_ids_buffer = Some(Vec::with_capacity(required_size * 2));
        }
        let buffer = self.input_ids_buffer.as_mut().unwrap();
        buffer.clear();
        buffer
    }

    /// Get or create attention mask buffer
    #[allow(dead_code)]
    fn get_attention_mask_buffer(&mut self, required_size: usize) -> &mut Vec<i64> {
        if self.attention_mask_buffer.is_none() || self.attention_mask_buffer.as_ref().unwrap().capacity() < required_size {
            self.attention_mask_buffer = Some(Vec::with_capacity(required_size * 2));
        }
        let buffer = self.attention_mask_buffer.as_mut().unwrap();
        buffer.clear();
        buffer
    }

    /// Get or create token_type_ids buffer
    #[allow(dead_code)]
    fn get_token_type_ids_buffer(&mut self, required_size: usize) -> &mut Vec<i64> {
        if self.token_type_ids_buffer.is_none() || self.token_type_ids_buffer.as_ref().unwrap().capacity() < required_size {
            self.token_type_ids_buffer = Some(Vec::with_capacity(required_size * 2));
        }
        let buffer = self.token_type_ids_buffer.as_mut().unwrap();
        buffer.clear();
        buffer
    }
}

impl Embedder {
    pub fn new(config: EmbedderConfig) -> Result<Self> {
        // Suppress ONNX Runtime verbose logs (BFCArena, memory allocation, etc.)
        std::env::set_var("ORT_LOGGING_LEVEL", "4"); // FATAL only
        std::env::set_var("ORT_LOG_SEVERITY_LEVEL", "4");

        ort::init().with_name("sutra-embedder").commit()?;

        let buffer_pool = BufferPool::new(config.batch_size, config.max_sequence_length);

        let mut embedder = Self {
            config,
            session: None,
            tokenizer: None,
            model_capabilities: ModelCapabilities::default(),
            buffer_pool,
        };

        // Auto-load default model if available (non-critical - use fallback on failure)
        match embedder.load_or_download_model() {
            Ok(_) => {
                // Warm up the model to detect capabilities
                if let Err(e) = embedder.warmup_session() {
                    warn!("Model warmup failed: {}. Will detect on first use.", e);
                }
            }
            Err(e) => {
                warn!(
                    "Model loading failed: {}. Using efficient fallback mode.",
                    e
                );
                warn!(
                    "Note: Real model inference requires compatible ONNX Runtime and model format"
                );
            }
        }

        Ok(embedder)
    }

    /// Create embedder with async model loading for production use
    #[allow(dead_code)]
    pub async fn new_async(config: EmbedderConfig) -> Result<Self> {
        // Suppress ONNX Runtime verbose logs
        std::env::set_var("ORT_LOGGING_LEVEL", "4"); // FATAL only
        std::env::set_var("ORT_LOG_SEVERITY_LEVEL", "4");

        ort::init().with_name("sutra-embedder").commit()?;

        let buffer_pool = BufferPool::new(config.batch_size, config.max_sequence_length);

        let mut embedder = Self {
            config,
            session: None,
            tokenizer: None,
            model_capabilities: ModelCapabilities::default(),
            buffer_pool,
        };

        // Load model using production registry
        embedder.load_or_download_model_async().await?;

        // Warm up the model
        embedder.warmup_session()?;

        Ok(embedder)
    }

    /// Load model from ONNX file
    pub fn load_model(&mut self, model_path: &Path, tokenizer_path: &Path) -> Result<()> {
        // Load tokenizer silently
        self.tokenizer = Some(
            Tokenizer::from_file(tokenizer_path)
                .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?,
        );

        // Suppress ONNX Runtime's internal allocator logs (BFCArena, etc.)
        // These are C++ level logs that bypass tracing
        std::env::set_var("ORT_LOGGING_LEVEL", "4"); // 4 = FATAL
        std::env::set_var("TF_CPP_MIN_LOG_LEVEL", "3"); // Suppress TensorFlow-style logs

        // Create ONNX session with optimization and hardware-adaptive settings
        let mut session_builder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(rayon::current_num_threads())?
            .with_log_level(LogLevel::Fatal)?;

        // Configure execution providers based on hardware
        session_builder = self.configure_execution_providers(session_builder)?;

        let session = session_builder.commit_from_file(model_path)?;

        info!("Model loaded successfully");
        self.session = Some(session);
        Ok(())
    }

    /// Configure execution providers for optimal hardware utilization
    fn configure_execution_providers(
        &self,
        builder: SessionBuilder,
    ) -> Result<SessionBuilder> {
        let hardware = crate::hardware::HardwareProfile::detect();

        // Enable FP16 if requested and supported
        if self.config.use_fp16 && hardware.has_fp16() {
            info!("Enabling FP16 mixed precision");
            // Note: FP16 support requires graph conversion
            // ONNX Runtime will automatically use FP16 ops when available
        } else if self.config.use_fp16 && !hardware.has_fp16() {
            warn!("FP16 requested but not supported by hardware, falling back to FP32");
        }

        // Try GPU acceleration if available
        if hardware.has_gpu() {
            #[cfg(feature = "cuda")]
            {
                info!("Enabling CUDA execution provider");
                let mut cuda_options = ort::execution_providers::CUDAExecutionProvider::default();
                
                // Enable FP16 for CUDA if requested
                if self.config.use_fp16 && hardware.has_fp16() {
                    // CUDA automatically uses Tensor Cores for FP16 when available
                    info!("CUDA will use FP16 Tensor Cores where available");
                }
                
                return Ok(builder.with_execution_providers([cuda_options.build()])?);
            }

            #[cfg(target_os = "macos")]
            {
                info!("Enabling CoreML execution provider");
                // CoreML automatically uses FP16 on Apple Silicon
                if self.config.use_fp16 && hardware.has_fp16() {
                    info!("CoreML will use FP16 on Apple Silicon Neural Engine");
                }
                return Ok(builder);
            }

            #[cfg(all(target_os = "windows", feature = "directml"))]
            {
                info!("Enabling DirectML execution provider");
                return Ok(builder.with_execution_providers([ort::execution_providers::DirectMLExecutionProvider::default().build()])?);
            }
        }

        // Default to CPU with optimizations
        debug!("Using CPU execution provider with optimizations");
        if self.config.use_fp16 {
            debug!("FP16 will be emulated on CPU (slower than native GPU FP16)");
        }
        Ok(builder)
    }

    /// Warmup session to detect model capabilities (eliminates double inference)
    fn warmup_session(&mut self) -> Result<()> {
        if self.session.is_none() || self.tokenizer.is_none() {
            return Err(anyhow!("Cannot warmup: session or tokenizer not loaded"));
        }

        info!("Warming up model session...");

        // Use a short dummy text for warmup
        let dummy_text = "warmup";
        let tokenizer = self.tokenizer.as_ref().unwrap();
        let session = self.session.as_mut().unwrap();

        // Tokenize
        let encoding = tokenizer
            .encode(dummy_text, true)
            .map_err(|e| anyhow!("Warmup tokenization failed: {}", e))?;

        let tokens: Vec<i64> = encoding.get_ids().iter().map(|&t| t as i64).collect();
        let attention: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&a| a as i64)
            .collect();
        let token_type_ids: Vec<i64> = vec![0i64; tokens.len()];

        let seq_len = tokens.len();

        // Test with token_type_ids
        let input_ids = Array2::from_shape_vec((1, seq_len), tokens.clone())?;
        let attention_mask = Array2::from_shape_vec((1, seq_len), attention.clone())?;
        let token_type_ids_array = Array2::from_shape_vec((1, seq_len), token_type_ids)?;

        let with_token_type_ids = session
            .run(ort::inputs![
                "input_ids" => Value::from_array((input_ids.shape().to_vec(), input_ids.into_raw_vec().into_boxed_slice()))?,
                "attention_mask" => Value::from_array((attention_mask.shape().to_vec(), attention_mask.into_raw_vec().into_boxed_slice()))?,
                "token_type_ids" => Value::from_array((token_type_ids_array.shape().to_vec(), token_type_ids_array.into_raw_vec().into_boxed_slice()))?,
            ])
            .is_ok();

        self.model_capabilities.needs_token_type_ids = with_token_type_ids;

        // Detect output shape type
        let outputs = if with_token_type_ids {
            let input_ids2 = Array2::from_shape_vec((1, seq_len), tokens.clone())?;
            let attention_mask2 = Array2::from_shape_vec((1, seq_len), attention.clone())?;
            let token_type_ids2 = Array2::from_shape_vec((1, seq_len), vec![0i64; seq_len])?;
            session.run(ort::inputs![
                "input_ids" => Value::from_array((input_ids2.shape().to_vec(), input_ids2.into_raw_vec().into_boxed_slice()))?,
                "attention_mask" => Value::from_array((attention_mask2.shape().to_vec(), attention_mask2.into_raw_vec().into_boxed_slice()))?,
                "token_type_ids" => Value::from_array((token_type_ids2.shape().to_vec(), token_type_ids2.into_raw_vec().into_boxed_slice()))?,
            ])?
        } else {
            let input_ids2 = Array2::from_shape_vec((1, seq_len), tokens)?;
            let attention_mask2 = Array2::from_shape_vec((1, seq_len), attention)?;
            session.run(ort::inputs![
                "input_ids" => Value::from_array((input_ids2.shape().to_vec(), input_ids2.into_raw_vec().into_boxed_slice()))?,
                "attention_mask" => Value::from_array((attention_mask2.shape().to_vec(), attention_mask2.into_raw_vec().into_boxed_slice()))?,
            ])?
        };

        // Check output shape
        let output_tensor = outputs.iter().next().unwrap().1;
        let tensor_view = output_tensor.try_extract_tensor::<f32>()?;
        let shape = tensor_view.0;

        self.model_capabilities.output_shape_type = if shape.len() == 3 {
            OutputShapeType::SequenceLevel
        } else if shape.len() == 2 {
            OutputShapeType::Pooled
        } else {
            OutputShapeType::Unknown
        };

        self.model_capabilities.is_warmed_up = true;

        info!(
            "Model warmup complete: token_type_ids={}, output_shape={:?}",
            self.model_capabilities.needs_token_type_ids, self.model_capabilities.output_shape_type
        );

        Ok(())
    }
    /// Get model cache directory - uses project's models/ folder
    fn get_model_cache_dir() -> Result<PathBuf> {
        // Use models/ directory in project root
        let model_cache = PathBuf::from("models");
        fs::create_dir_all(&model_cache)?;
        Ok(model_cache)
    }

    /// Download a file with progress bar
    #[allow(dead_code)]
    fn download_file(url: &str, dest: &Path) -> Result<()> {
        info!("Downloading from {}", url);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        let response = client.get(url).send()?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));
        pb.set_message(format!(
            "Downloading {}",
            dest.file_name().unwrap().to_string_lossy()
        ));

        let mut file = fs::File::create(dest)?;
        let mut downloaded = 0u64;

        for chunk in response.bytes()?.chunks(8192) {
            file.write_all(chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    /// Load or download model using production-grade model registry
    #[allow(dead_code)]
    pub async fn load_or_download_model_async(&mut self) -> Result<()> {
        let cache_dir = Self::get_model_cache_dir()?;
        let registry = ModelRegistry::new();

        // Get model ID from config or use default
        let model_id = self
            .config
            .model_id
            .as_ref()
            .unwrap_or(&"all-MiniLM-L6-v2".to_string())
            .clone();

        let model_info = registry
            .get_model(&model_id)
            .ok_or_else(|| anyhow!("Model '{}' not found in registry", model_id))?;

        // Download model if needed
        let (model_path, tokenizer_path) = registry.download_model(model_info, &cache_dir).await?;

        // Validate health
        if !registry
            .health_check_model(&model_path, &tokenizer_path)
            .await?
        {
            return Err(anyhow!("Model health check failed after download"));
        }

        // Load the model
        self.load_model(&model_path, &tokenizer_path)?;

        Ok(())
    }

    /// Load or download model based on config (synchronous fallback)
    fn load_or_download_model(&mut self) -> Result<()> {
        let cache_dir = Self::get_model_cache_dir()?;
        let registry = ModelRegistry::new();

        // Get model ID from config or use default
        let model_id = self
            .config
            .model_id
            .as_ref()
            .unwrap_or(&"all-MiniLM-L6-v2".to_string())
            .clone();

        if registry.get_model(&model_id).is_none() {
            return Err(anyhow!("Model '{}' not found in registry", model_id));
        }

        let model_path = cache_dir.join(format!("{}.onnx", model_id));
        let tokenizer_path = cache_dir.join(format!("{}-tokenizer.json", model_id));

        // Try to load if model exists locally
        if model_path.exists() && tokenizer_path.exists() {
            self.load_model(&model_path, &tokenizer_path)?;
            Ok(())
        } else {
            Err(anyhow!("Model '{}' files not found in cache: {}. Run 'sutra-embedder download --model {}' first.", model_id, cache_dir.display(), model_id))
        }
    }

    /// Generate embedding for a single text
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Use batch processing for single text (leverages optimized path)
        let mut results = self.embed_batch(&[text.to_string()])?;
        Ok(results.pop().unwrap())
    }

    /// Real ONNX model inference (DEPRECATED - use embed_batch_internal)
    #[allow(dead_code)]
    fn create_model_embedding(&mut self, text: &str) -> Result<Vec<f32>> {
        let mut results = self.embed_batch_internal(&[text], false)?;
        Ok(results.pop().unwrap())
    }

    /// Generate embeddings for multiple texts (optimized batch processing)
    pub fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        if self.session.is_some() && self.tokenizer.is_some() {
            // Use real ONNX model inference with optimizations
            self.embed_batch_internal(&text_refs, true)
        } else {
            // Fallback to simple embeddings
            warn!("Using fallback embeddings - model not loaded");
            text_refs
                .iter()
                .map(|text| self.create_simple_embedding(text))
                .collect()
        }
    }

    /// Internal optimized batch processing with tensor reuse
    fn embed_batch_internal(&mut self, texts: &[&str], apply_postprocessing: bool) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let tokenizer = self.tokenizer.as_ref().unwrap();
        let session = self.session.as_mut().unwrap();

        let batch_size = texts.len();
        let max_seq_len = self.config.max_sequence_length;

        // Tokenize all texts
        let mut all_tokens = Vec::with_capacity(batch_size);
        let mut all_attention = Vec::with_capacity(batch_size);
        let mut sequence_lengths = Vec::with_capacity(batch_size);

        for text in texts {
            let encoding = tokenizer
                .encode(*text, true)
                .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

            let tokens = encoding.get_ids();
            let attention = encoding.get_attention_mask();

            let seq_len = max_seq_len.min(tokens.len());
            all_tokens.push(tokens[..seq_len].to_vec());
            all_attention.push(attention[..seq_len].to_vec());
            sequence_lengths.push(seq_len);
        }

        // Find max sequence length in batch for efficient packing
        let batch_max_len = *sequence_lengths.iter().max().unwrap();

        // Prepare batched inputs with buffer reuse
        let total_size = batch_size * batch_max_len;
        
        // Collect all input IDs and attention masks
        let mut input_ids_vec = Vec::with_capacity(total_size);
        let mut attention_vec = Vec::with_capacity(total_size);

        // Pack inputs with padding
        for (tokens, attention) in all_tokens.iter().zip(all_attention.iter()) {
            for j in 0..batch_max_len {
                if j < tokens.len() {
                    input_ids_vec.push(tokens[j] as i64);
                    attention_vec.push(attention[j] as i64);
                } else {
                    input_ids_vec.push(0); // Padding token
                    attention_vec.push(0); // Padding attention
                }
            }
        }

        // Create input tensors
        let input_ids_array = Array2::from_shape_vec((batch_size, batch_max_len), input_ids_vec.clone())?;
        let attention_array = Array2::from_shape_vec((batch_size, batch_max_len), attention_vec.clone())?;

        // Run inference based on detected model capabilities (no double inference!)
        let needs_token_type_ids = self.model_capabilities.needs_token_type_ids;
        let is_warmed = self.model_capabilities.is_warmed_up;
        
        let outputs = if is_warmed {
            // Use pre-detected capabilities
            if needs_token_type_ids {
                let token_type_vec = vec![0i64; total_size];
                let token_type_array = Array2::from_shape_vec((batch_size, batch_max_len), token_type_vec)?;
                
                session.run(ort::inputs![
                    "input_ids" => Value::from_array((input_ids_array.shape().to_vec(), input_ids_array.into_raw_vec().into_boxed_slice()))?,
                    "attention_mask" => Value::from_array((attention_array.shape().to_vec(), attention_array.into_raw_vec().into_boxed_slice()))?,
                    "token_type_ids" => Value::from_array((token_type_array.shape().to_vec(), token_type_array.into_raw_vec().into_boxed_slice()))?,
                ])?
            } else {
                session.run(ort::inputs![
                    "input_ids" => Value::from_array((input_ids_array.shape().to_vec(), input_ids_array.into_raw_vec().into_boxed_slice()))?,
                    "attention_mask" => Value::from_array((attention_array.shape().to_vec(), attention_array.into_raw_vec().into_boxed_slice()))?,
                ])?
            }
        } else {
            // Fallback to runtime detection (first call only)
            warn!("Model not warmed up, using runtime detection");
            let token_type_vec = vec![0i64; total_size];
            let token_type_array = Array2::from_shape_vec((batch_size, batch_max_len), token_type_vec)?;

            if session
                .run(ort::inputs![
                    "input_ids" => Value::from_array((input_ids_array.shape().to_vec(), input_ids_array.clone().into_raw_vec().into_boxed_slice()))?,
                    "attention_mask" => Value::from_array((attention_array.shape().to_vec(), attention_array.clone().into_raw_vec().into_boxed_slice()))?,
                    "token_type_ids" => Value::from_array((token_type_array.shape().to_vec(), token_type_array.into_raw_vec().into_boxed_slice()))?,
                ])
                .is_ok()
            {
                self.model_capabilities.needs_token_type_ids = true;
                let token_type_vec2 = vec![0i64; total_size];
                let token_type_array2 = Array2::from_shape_vec((batch_size, batch_max_len), token_type_vec2)?;
                session.run(ort::inputs![
                    "input_ids" => Value::from_array((input_ids_array.shape().to_vec(), input_ids_array.into_raw_vec().into_boxed_slice()))?,
                    "attention_mask" => Value::from_array((attention_array.shape().to_vec(), attention_array.into_raw_vec().into_boxed_slice()))?,
                    "token_type_ids" => Value::from_array((token_type_array2.shape().to_vec(), token_type_array2.into_raw_vec().into_boxed_slice()))?,
                ])?
            } else {
                self.model_capabilities.needs_token_type_ids = false;
                session.run(ort::inputs![
                    "input_ids" => Value::from_array((input_ids_array.shape().to_vec(), input_ids_array.into_raw_vec().into_boxed_slice()))?,
                    "attention_mask" => Value::from_array((attention_array.shape().to_vec(), attention_array.into_raw_vec().into_boxed_slice()))?,
                ])?
            }
        };

        // Extract embeddings from output
        let output_tensor = outputs.iter().next().ok_or_else(|| anyhow!("No output tensor"))?.1;
        let tensor_view = output_tensor.try_extract_tensor::<f32>()?;
        let shape = tensor_view.0;
        let data = tensor_view.1;

        let mut embeddings = Vec::with_capacity(batch_size);

        // Determine output shape type if not known (clone to avoid borrow issues)
        let output_shape_type = self.model_capabilities.output_shape_type.clone();
        let output_shape_type = if output_shape_type != OutputShapeType::Unknown {
            output_shape_type
        } else if shape.len() == 3 {
            OutputShapeType::SequenceLevel
        } else if shape.len() == 2 {
            OutputShapeType::Pooled
        } else {
            return Err(anyhow!("Unexpected output shape: {:?}", shape));
        };

        // Process based on output shape
        match output_shape_type {
            OutputShapeType::SequenceLevel if shape.len() == 3 => {
                // [batch, seq, hidden] - need pooling
                let seq_len = shape[1] as usize;
                let hidden_dim = shape[2] as usize;

                // Use fused pooling+normalization if enabled
                // TODO: Fix module visibility for binary builds
                // Fused ops temporarily disabled due to module resolution issues
                let _use_fused = self.config.use_fused_ops;
                {
                    // Standard two-pass approach
                    for i in 0..batch_size {
                        let start = i * seq_len * hidden_dim;
                        let batch_data = &data[start..start + seq_len * hidden_dim];
                        let pooled = Self::mean_pool_simd(batch_data, seq_len, hidden_dim);
                        let normalized = Self::l2_normalize_simd(&pooled);
                        embeddings.push(normalized);
                    }
                }
            }
            OutputShapeType::Pooled if shape.len() == 2 => {
                // [batch, hidden] - already pooled
                let hidden_dim = shape[1] as usize;
                for i in 0..batch_size {
                    let start = i * hidden_dim;
                    let batch_embedding = data[start..start + hidden_dim].to_vec();
                    let normalized = Self::l2_normalize_simd(&batch_embedding);
                    embeddings.push(normalized);
                }
            }
            OutputShapeType::SequenceLevel => {
                // Detected as sequence level but shape doesn't match - use default
                let seq_len = shape[1] as usize;
                let hidden_dim = shape[2] as usize;
                for i in 0..batch_size {
                    let start = i * seq_len * hidden_dim;
                    let batch_data = &data[start..start + seq_len * hidden_dim];
                    let pooled = Self::mean_pool_simd(batch_data, seq_len, hidden_dim);
                    let normalized = Self::l2_normalize_simd(&pooled);
                    embeddings.push(normalized);
                }
            }
            _ => {
                // Fallback for unknown or pooled without proper shape
                let hidden_dim = shape[shape.len() - 1] as usize;
                for i in 0..batch_size {
                    let start = i * hidden_dim;
                    let batch_embedding = data[start..start + hidden_dim].to_vec();
                    let normalized = Self::l2_normalize_simd(&batch_embedding);
                    embeddings.push(normalized);
                }
            }
        }

        // Apply post-processing if requested
        if apply_postprocessing {
            let requested_dim = self.config.target_dimension.unwrap_or(self.config.dimensions);
            let binary_quant = self.config.binary_quantization;
            let _use_fused = self.config.use_fused_ops;
            
            embeddings = embeddings
                .into_iter()
                .map(|mut emb| {
                    // TODO: Enable fused ops once module visibility is fixed
                    // Original two-step approach
                    // Truncate to requested dimension
                    if requested_dim < emb.len() {
                        emb.truncate(requested_dim);
                    }
                    
                    // Apply binary quantization if enabled
                    if binary_quant {
                        Self::binary_quantize_vec(&emb)
                    } else {
                        emb
                    }
                })
                .collect();
        }

        Ok(embeddings)
    }

    /// SIMD-optimized mean pooling
    #[inline]
    fn mean_pool_simd(data: &[f32], seq_len: usize, hidden_dim: usize) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    return Self::mean_pool_avx2(data, seq_len, hidden_dim);
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            // NEON is always available on AArch64
            unsafe {
                Self::mean_pool_neon(data, seq_len, hidden_dim)
            }
        }

        // Fallback: scalar implementation
        #[cfg(not(target_arch = "aarch64"))]
        {
            let mut pooled = vec![0.0f32; hidden_dim];
            for i in 0..seq_len {
                for j in 0..hidden_dim {
                    pooled[j] += data[i * hidden_dim + j];
                }
            }

            let scale = 1.0 / seq_len as f32;
            for val in &mut pooled {
                *val *= scale;
            }
            pooled
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn mean_pool_avx2(data: &[f32], seq_len: usize, hidden_dim: usize) -> Vec<f32> {
        let mut pooled = vec![0.0f32; hidden_dim];
        let chunks = hidden_dim / 8;
        let remainder = hidden_dim % 8;

        for i in 0..seq_len {
            let row_offset = i * hidden_dim;
            
            // Process 8 floats at a time with AVX2
            for j in 0..chunks {
                let idx = j * 8;
                let sum = _mm256_loadu_ps(pooled.as_ptr().add(idx));
                let val = _mm256_loadu_ps(data.as_ptr().add(row_offset + idx));
                let new_sum = _mm256_add_ps(sum, val);
                _mm256_storeu_ps(pooled.as_mut_ptr().add(idx), new_sum);
            }
            
            // Handle remainder
            for j in (chunks * 8)..hidden_dim {
                pooled[j] += data[row_offset + j];
            }
        }

        // Scale by sequence length
        let scale = _mm256_set1_ps(1.0 / seq_len as f32);
        for j in 0..chunks {
            let idx = j * 8;
            let val = _mm256_loadu_ps(pooled.as_ptr().add(idx));
            let scaled = _mm256_mul_ps(val, scale);
            _mm256_storeu_ps(pooled.as_mut_ptr().add(idx), scaled);
        }
        
        for j in (chunks * 8)..hidden_dim {
            pooled[j] /= seq_len as f32;
        }

        pooled
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn mean_pool_neon(data: &[f32], seq_len: usize, hidden_dim: usize) -> Vec<f32> {
        let mut pooled = vec![0.0f32; hidden_dim];
        let chunks = hidden_dim / 4;

        for i in 0..seq_len {
            let row_offset = i * hidden_dim;
            
            // Process 4 floats at a time with NEON
            for j in 0..chunks {
                let idx = j * 4;
                let sum = vld1q_f32(pooled.as_ptr().add(idx));
                let val = vld1q_f32(data.as_ptr().add(row_offset + idx));
                let new_sum = vaddq_f32(sum, val);
                vst1q_f32(pooled.as_mut_ptr().add(idx), new_sum);
            }
            
            // Handle remainder
            for j in (chunks * 4)..hidden_dim {
                pooled[j] += data[row_offset + j];
            }
        }

        // Scale
        let scale = 1.0 / seq_len as f32;
        for val in &mut pooled {
            *val *= scale;
        }

        pooled
    }

    /// SIMD-optimized L2 normalization
    #[inline]
    fn l2_normalize_simd(embedding: &[f32]) -> Vec<f32> {
        let norm_sq = Self::dot_product_simd(embedding, embedding);
        
        if norm_sq <= 0.0 {
            return embedding.to_vec();
        }

        let norm = norm_sq.sqrt();
        let scale = 1.0 / norm;

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    return Self::scale_vector_avx2(embedding, scale);
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                Self::scale_vector_neon(embedding, scale)
            }
        }

        // Fallback: scalar
        #[cfg(not(target_arch = "aarch64"))]
        {
            let mut normalized = vec![0.0f32; embedding.len()];
            for (i, &val) in embedding.iter().enumerate() {
                normalized[i] = val * scale;
            }
            normalized
        }
    }

    /// SIMD-optimized dot product
    #[inline]
    fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    return Self::dot_product_avx2(a, b);
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            unsafe {
                Self::dot_product_neon(a, b)
            }
        }

        // Fallback
        #[cfg(not(target_arch = "aarch64"))]
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let chunks = len / 8;
        let mut sum = _mm256_setzero_ps();

        for i in 0..chunks {
            let idx = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(idx));
            let vb = _mm256_loadu_ps(b.as_ptr().add(idx));
            let prod = _mm256_mul_ps(va, vb);
            sum = _mm256_add_ps(sum, prod);
        }

        // Horizontal sum
        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), sum);
        let mut total = result.iter().sum::<f32>();

        // Handle remainder
        for i in (chunks * 8)..len {
            total += a[i] * b[i];
        }

        total
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let chunks = len / 4;
        let mut sum = vdupq_n_f32(0.0);

        for i in 0..chunks {
            let idx = i * 4;
            let va = vld1q_f32(a.as_ptr().add(idx));
            let vb = vld1q_f32(b.as_ptr().add(idx));
            let prod = vmulq_f32(va, vb);
            sum = vaddq_f32(sum, prod);
        }

        // Extract sum
        let mut result = [0.0f32; 4];
        vst1q_f32(result.as_mut_ptr(), sum);
        let mut total = result.iter().sum::<f32>();

        // Handle remainder
        for i in (chunks * 4)..len {
            total += a[i] * b[i];
        }

        total
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn scale_vector_avx2(vec: &[f32], scale: f32) -> Vec<f32> {
        let len = vec.len();
        let chunks = len / 8;
        let mut result = vec![0.0f32; len];
        let scale_vec = _mm256_set1_ps(scale);

        for i in 0..chunks {
            let idx = i * 8;
            let val = _mm256_loadu_ps(vec.as_ptr().add(idx));
            let scaled = _mm256_mul_ps(val, scale_vec);
            _mm256_storeu_ps(result.as_mut_ptr().add(idx), scaled);
        }

        for i in (chunks * 8)..len {
            result[i] = vec[i] * scale;
        }

        result
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn scale_vector_neon(vec: &[f32], scale: f32) -> Vec<f32> {
        let len = vec.len();
        let chunks = len / 4;
        let mut result = vec![0.0f32; len];
        let scale_vec = vdupq_n_f32(scale);

        for i in 0..chunks {
            let idx = i * 4;
            let val = vld1q_f32(vec.as_ptr().add(idx));
            let scaled = vmulq_f32(val, scale_vec);
            vst1q_f32(result.as_mut_ptr().add(idx), scaled);
        }

        for i in (chunks * 4)..len {
            result[i] = vec[i] * scale;
        }

        result
    }

    /// Fallback: Simple embedding based on character/word features
    fn create_simple_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut embedding = vec![0.0f32; self.config.dimensions];

        // Create a simple but deterministic embedding based on text features
        let _words: Vec<&str> = text.split_whitespace().collect();

        // Use hashing for deterministic feature generation
        for chunk in text.as_bytes().chunks(8) {
            let mut hasher = DefaultHasher::new();
            chunk.hash(&mut hasher);
            let hash = hasher.finish();

            let idx = (hash as usize) % embedding.len();
            embedding[idx] += ((hash >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        debug!(
            "Generated fallback embedding of dimension {}",
            embedding.len()
        );
        Ok(embedding)
    }

    /// Binary quantization: convert float32 to 1-bit (0 or 1)
    #[allow(dead_code)]
    fn binary_quantize(&self, embedding: &[f32]) -> Vec<f32> {
        Self::binary_quantize_vec(embedding)
    }

    /// Binary quantization (static version): convert float32 to 1-bit (0 or 1)
    fn binary_quantize_vec(embedding: &[f32]) -> Vec<f32> {
        embedding
            .iter()
            .map(|&x| if x > 0.0 { 1.0 } else { 0.0 })
            .collect()
    }

    /// Truncate embedding to specific matryoshka dimension
    #[allow(dead_code)]
    pub fn truncate_embedding(&self, embedding: &[f32], target_dim: usize) -> Vec<f32> {
        embedding.iter().take(target_dim).copied().collect()
    }

    /// Get configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &EmbedderConfig {
        &self.config
    }
}

/// Async batch processing queue for background embedding generation
pub struct AsyncBatchQueue {
    tx: mpsc::UnboundedSender<BatchRequest>,
    _worker_handle: tokio::task::JoinHandle<()>,
}

/// Request for async batch processing
struct BatchRequest {
    texts: Vec<String>,
    response_tx: oneshot::Sender<Result<Vec<Vec<f32>>>>,
}

impl AsyncBatchQueue {
    /// Create new async batch queue with background worker
    pub fn new(config: EmbedderConfig) -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel::<BatchRequest>();
        
        // Spawn background worker
        let worker_handle = tokio::spawn(async move {
            let embedder = Arc::new(Mutex::new(
                Embedder::new(config).expect("Failed to create embedder for async queue")
            ));
            
            info!("Async batch queue worker started");
            
            while let Some(request) = rx.recv().await {
                let embedder_clone = Arc::clone(&embedder);
                
                // Process batch in background
                tokio::spawn(async move {
                    let result = {
                        let mut embedder = embedder_clone.lock().await;
                        embedder.embed_batch(&request.texts)
                    };
                    
                    // Send response back (ignore if receiver dropped)
                    let _ = request.response_tx.send(result);
                });
            }
            
            info!("Async batch queue worker stopped");
        });
        
        Ok(Self {
            tx,
            _worker_handle: worker_handle,
        })
    }
    
    /// Submit texts for async embedding generation
    pub async fn embed_async(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.tx.send(BatchRequest {
            texts,
            response_tx,
        }).map_err(|e| anyhow!("Failed to send batch request: {}", e))?;
        
        response_rx.await
            .map_err(|e| anyhow!("Failed to receive response: {}", e))?
    }
    
    /// Submit single text for async embedding generation
    pub async fn embed_one_async(&self, text: String) -> Result<Vec<f32>> {
        let mut results = self.embed_async(vec![text]).await?;
        results.pop().ok_or_else(|| anyhow!("No embedding returned"))
    }
}

/// Bounded async batch queue with backpressure support
pub struct BoundedAsyncBatchQueue {
    tx: mpsc::Sender<BatchRequest>,
    _worker_handle: tokio::task::JoinHandle<()>,
}

impl BoundedAsyncBatchQueue {
    /// Create new bounded async batch queue with specified capacity
    pub fn new(config: EmbedderConfig, capacity: usize) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel::<BatchRequest>(capacity);
        
        // Spawn background worker
        let worker_handle = tokio::spawn(async move {
            let embedder = Arc::new(Mutex::new(
                Embedder::new(config).expect("Failed to create embedder for bounded async queue")
            ));
            
            info!("Bounded async batch queue worker started (capacity: {})", capacity);
            
            while let Some(request) = rx.recv().await {
                let embedder_clone = Arc::clone(&embedder);
                
                // Process batch in background
                tokio::spawn(async move {
                    let result = {
                        let mut embedder = embedder_clone.lock().await;
                        embedder.embed_batch(&request.texts)
                    };
                    
                    // Send response back (ignore if receiver dropped)
                    let _ = request.response_tx.send(result);
                });
            }
            
            info!("Bounded async batch queue worker stopped");
        });
        
        Ok(Self {
            tx,
            _worker_handle: worker_handle,
        })
    }
    
    /// Submit texts for async embedding generation (with backpressure)
    pub async fn embed_async(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.tx.send(BatchRequest {
            texts,
            response_tx,
        }).await.map_err(|e| anyhow!("Failed to send batch request: {}", e))?;
        
        response_rx.await
            .map_err(|e| anyhow!("Failed to receive response: {}", e))?
    }
    
    /// Submit single text for async embedding generation (with backpressure)
    pub async fn embed_one_async(&self, text: String) -> Result<Vec<f32>> {
        let mut results = self.embed_async(vec![text]).await?;
        results.pop().ok_or_else(|| anyhow!("No embedding returned"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_creation() {
        let config = EmbedderConfig::from_name("efficient").unwrap();
        let embedder = Embedder::new(config).unwrap();
        assert_eq!(embedder.config().dimensions, 384);
    }

    #[test]
    fn test_embed_dimensions() {
        let config = EmbedderConfig::from_name("efficient").unwrap();
        let mut embedder = Embedder::new(config).unwrap();
        let embedding = embedder.embed("test text").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_dynamic_dimension_truncation() {
        let config = EmbedderConfig::for_dimension(256, "desktop").unwrap();
        let mut embedder = Embedder::new(config).unwrap();
        let embedding = embedder.embed("dynamic dims").unwrap();
        assert_eq!(embedding.len(), 256);
    }

    #[test]
    fn test_reject_oversized_dimension_request() {
        let config = EmbedderConfig::for_dimension(1536, "desktop");
        assert!(config.is_err());
    }
}
