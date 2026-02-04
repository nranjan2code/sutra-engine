/// Model templates for common use cases
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::{ModelArchitecture, ModelSize, LoraConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTemplate {
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub architecture: ModelArchitecture,
    pub recommended_size: ModelSize,
    pub data_requirements: DataRequirements,
    pub training_config: TemplateTrainingConfig,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRequirements {
    pub min_samples: usize,
    pub recommended_samples: usize,
    pub data_format: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTrainingConfig {
    pub epochs: usize,
    pub batch_size: usize,
    pub learning_rate: f32,
    pub lora: LoraConfig,
}

pub struct TemplateManager {
    templates: HashMap<String, ModelTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.load_builtin_templates();
        manager
    }

    pub fn get_templates(&self) -> Vec<&ModelTemplate> {
        self.templates.values().collect()
    }

    pub fn get_template(&self, name: &str) -> Option<&ModelTemplate> {
        self.templates.get(name)
    }

    fn load_builtin_templates(&mut self) {
        // Chat Assistant Template
        self.templates.insert("chat-assistant".to_string(), ModelTemplate {
            name: "Chat Assistant".to_string(),
            description: "A helpful AI assistant for general conversations and Q&A".to_string(),
            use_case: "Customer support, general assistance, educational tutoring".to_string(),
            architecture: ModelArchitecture::RWKV,
            recommended_size: ModelSize::Small,
            data_requirements: DataRequirements {
                min_samples: 1000,
                recommended_samples: 10000,
                data_format: "Conversation pairs in JSONL format".to_string(),
                example: r#"{"instruction": "What is the capital of France?", "response": "The capital of France is Paris."}"#.to_string(),
            },
            training_config: TemplateTrainingConfig {
                epochs: 5,
                batch_size: 4,
                learning_rate: 5e-5,
                lora: LoraConfig {
                    enabled: true,
                    rank: 8,
                    alpha: 16.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into(), "o_proj".into()],
                },
            },
            icon: "💬".to_string(),
        });

        // Code Assistant Template
        self.templates.insert("code-assistant".to_string(), ModelTemplate {
            name: "Code Assistant".to_string(),
            description: "Specialized for code generation, debugging, and programming help".to_string(),
            use_case: "Code completion, bug fixing, code review, programming education".to_string(),
            architecture: ModelArchitecture::Mamba,
            recommended_size: ModelSize::Medium,
            data_requirements: DataRequirements {
                min_samples: 2000,
                recommended_samples: 50000,
                data_format: "Code examples with comments/explanations".to_string(),
                example: r#"{"code": "def fibonacci(n):", "completion": "    if n <= 1: return n\n    return fibonacci(n-1) + fibonacci(n-2)"}"#.to_string(),
            },
            training_config: TemplateTrainingConfig {
                epochs: 8,
                batch_size: 2,
                learning_rate: 3e-5,
                lora: LoraConfig {
                    enabled: true,
                    rank: 16,
                    alpha: 32.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into(), "gate_proj".into()],
                },
            },
            icon: "👨‍💻".to_string(),
        });

        // Document Analyzer Template
        self.templates.insert("document-analyzer".to_string(), ModelTemplate {
            name: "Document Analyzer".to_string(),
            description: "Extract insights and answer questions about documents".to_string(),
            use_case: "Document Q&A, summarization, information extraction".to_string(),
            architecture: ModelArchitecture::RWKV,
            recommended_size: ModelSize::Large,
            data_requirements: DataRequirements {
                min_samples: 500,
                recommended_samples: 5000,
                data_format: "Document-question-answer triplets".to_string(),
                example: r#"{"document": "Annual Report 2023...", "question": "What was the revenue growth?", "answer": "Revenue grew 15% year-over-year."}"#.to_string(),
            },
            training_config: TemplateTrainingConfig {
                epochs: 6,
                batch_size: 2,
                learning_rate: 2e-5,
                lora: LoraConfig {
                    enabled: true,
                    rank: 12,
                    alpha: 24.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into(), "k_proj".into()],
                },
            },
            icon: "📄".to_string(),
        });

        // Creative Writer Template
        self.templates.insert("creative-writer".to_string(), ModelTemplate {
            name: "Creative Writer".to_string(),
            description: "Generate creative content like stories, poems, and articles".to_string(),
            use_case: "Content creation, storytelling, marketing copy, creative writing".to_string(),
            architecture: ModelArchitecture::Mamba,
            recommended_size: ModelSize::Medium,
            data_requirements: DataRequirements {
                min_samples: 1000,
                recommended_samples: 20000,
                data_format: "Creative text samples with style/genre labels".to_string(),
                example: r#"{"style": "fantasy", "prompt": "Once upon a time", "text": "Once upon a time, in a land where magic flowed..."}"#.to_string(),
            },
            training_config: TemplateTrainingConfig {
                epochs: 10,
                batch_size: 4,
                learning_rate: 4e-5,
                lora: LoraConfig {
                    enabled: true,
                    rank: 8,
                    alpha: 16.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into()],
                },
            },
            icon: "✍️".to_string(),
        });

        // Data Scientist Template
        self.templates.insert("data-scientist".to_string(), ModelTemplate {
            name: "Data Scientist".to_string(),
            description: "Analyze data, create visualizations, and provide insights".to_string(),
            use_case: "Data analysis, statistical modeling, report generation".to_string(),
            architecture: ModelArchitecture::RWKV,
            recommended_size: ModelSize::Medium,
            data_requirements: DataRequirements {
                min_samples: 1500,
                recommended_samples: 15000,
                data_format: "Data analysis examples with code and explanations".to_string(),
                example: r#"{"dataset": "sales_data.csv", "question": "What's the trend?", "analysis": "import pandas as pd\ndf = pd.read_csv('sales_data.csv')..."}"#.to_string(),
            },
            training_config: TemplateTrainingConfig {
                epochs: 7,
                batch_size: 3,
                learning_rate: 3e-5,
                lora: LoraConfig {
                    enabled: true,
                    rank: 10,
                    alpha: 20.0,
                    target_modules: vec!["q_proj".into(), "v_proj".into(), "o_proj".into()],
                },
            },
            icon: "📊".to_string(),
        });
    }
}