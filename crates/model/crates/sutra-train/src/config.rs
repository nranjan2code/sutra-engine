/// Training configuration management
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    // Model configuration
    pub model: ModelConfig,
    
    // Training parameters  
    pub training: TrainingParams,
    
    // Data configuration
    pub data: DataConfig,
    
    // Output settings
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub template: String,
    pub architecture: ModelArchitecture,
    pub size: ModelSize,
    pub quantization: QuantizationConfig,
    pub lora: LoraConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelArchitecture {
    RWKV,
    Mamba,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelSize {
    Tiny,    // ~100M params
    Small,   // ~500M params
    Medium,  // ~1B params
    Large,   // ~3B params
    XL,      // ~7B params
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub enabled: bool,
    pub bits: u8,
    pub method: QuantMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantMethod {
    AWQ,
    GPTQ,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    pub enabled: bool,
    pub rank: usize,
    pub alpha: f32,
    pub target_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingParams {
    pub epochs: usize,
    pub batch_size: usize,
    pub learning_rate: f32,
    pub warmup_steps: usize,
    pub weight_decay: f32,
    pub gradient_clipping: f32,
    pub save_every: usize,
    pub eval_every: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub train_path: Option<String>,
    pub validation_path: Option<String>,
    pub test_path: Option<String>,
    pub format: DataFormat,
    pub max_length: usize,
    pub validation_split: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    Text,
    Jsonl,
    Csv,
    Parquet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub output_dir: String,
    pub model_name: String,
    pub save_checkpoints: bool,
    pub export_format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Safetensors,
    ONNX,
    TorchScript,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                template: "chat-assistant".to_string(),
                architecture: ModelArchitecture::RWKV,
                size: ModelSize::Small,
                quantization: QuantizationConfig {
                    enabled: true,
                    bits: 4,
                    method: QuantMethod::AWQ,
                },
                lora: LoraConfig {
                    enabled: true,
                    rank: 8,
                    alpha: 16.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into()],
                },
            },
            training: TrainingParams {
                epochs: 10,
                batch_size: 4,
                learning_rate: 5e-5,
                warmup_steps: 100,
                weight_decay: 0.01,
                gradient_clipping: 1.0,
                save_every: 1000,
                eval_every: 500,
            },
            data: DataConfig {
                train_path: None,
                validation_path: None,
                test_path: None,
                format: DataFormat::Text,
                max_length: 2048,
                validation_split: 0.1,
            },
            output: OutputConfig {
                output_dir: "./training_output".to_string(),
                model_name: "my_model".to_string(),
                save_checkpoints: true,
                export_format: ExportFormat::Safetensors,
            },
        }
    }
}

impl ModelSize {
    pub fn params(&self) -> u64 {
        match self {
            ModelSize::Tiny => 100_000_000,
            ModelSize::Small => 500_000_000,
            ModelSize::Medium => 1_000_000_000,
            ModelSize::Large => 3_000_000_000,
            ModelSize::XL => 7_000_000_000,
        }
    }

    pub fn memory_gb(&self) -> f32 {
        match self {
            ModelSize::Tiny => 0.4,
            ModelSize::Small => 2.0,
            ModelSize::Medium => 4.0,
            ModelSize::Large => 12.0,
            ModelSize::XL => 28.0,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ModelSize::Tiny => "Best for testing and experimentation",
            ModelSize::Small => "Good balance of performance and resource usage",
            ModelSize::Medium => "Higher quality outputs, moderate resource needs",
            ModelSize::Large => "Professional quality, requires 16GB+ RAM",
            ModelSize::XL => "Highest quality, requires 32GB+ RAM",
        }
    }
}