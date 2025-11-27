/// Demo Models - Pre-configured models for interactive testing
use sutra_core::Result;
use sutra_rwkv::{RwkvConfig, RwkvModel};
use sutra_mamba::{MambaConfig, MambaModel};
use sutra_tokenizer::{BpeConfig, BpeTokenizer, Tokenizer, VocabBuilder};
use std::time::Instant;

pub struct DemoModels {
    /// Small RWKV model for chat
    pub rwkv_chat: RwkvModel,
    
    /// Small Mamba model for comparison
    pub mamba_model: MambaModel,
    
    /// Simple tokenizer for demo
    pub tokenizer: Tokenizer,
    
    /// Model configurations for display
    pub rwkv_config: RwkvConfig,
    pub mamba_config: MambaConfig,
}

impl DemoModels {
    pub fn new() -> Self {
        println!("🔄 Creating demo models...");
        
        // Create small, fast models for interactive demo
        let rwkv_config = RwkvConfig::new(
            4,    // 4 layers - fast for demo
            384,  // 384 hidden size - good balance
            2000, // 2K vocab - sufficient for demo
        );
        
        let mamba_config = MambaConfig::new(
            4,    // 4 layers - same as RWKV for fair comparison  
            384,  // 384 hidden size - same as RWKV
            2000, // 2K vocab - same as RWKV
        );
        
        println!("📝 RWKV Config: {} layers, {} hidden, {} vocab", 
                 rwkv_config.num_layers, rwkv_config.hidden_size, rwkv_config.vocab_size);
        println!("📝 Mamba Config: {} layers, {} hidden, {} vocab",
                 mamba_config.num_layers, mamba_config.hidden_size, mamba_config.vocab_size);
        
        let rwkv_model = RwkvModel::new(rwkv_config.clone())
            .expect("Failed to create RWKV model");
            
        let mamba_model = MambaModel::new(mamba_config.clone())
            .expect("Failed to create Mamba model");
            
        let tokenizer = Self::create_demo_tokenizer()
            .expect("Failed to create tokenizer");
            
        println!("✅ Models ready!");
        
        Self {
            rwkv_chat: rwkv_model,
            mamba_model,
            tokenizer,
            rwkv_config,
            mamba_config,
        }
    }
    
    /// Run RWKV inference and measure time
    pub fn rwkv_inference(&self, prompt: &str) -> Result<(String, f64)> {
        let start = Instant::now();
        
        // Tokenize
        let encoding = self.tokenizer.encode(prompt).map_err(|e| sutra_core::SutraError::ComputeError(e.to_string()))?;
        let token_ids: Vec<usize> = encoding.ids.iter().map(|&id| (id as usize) % 2000).collect();
        
        // Generate (simplified for demo - just one token)
        let (logits, _state) = self.rwkv_chat.forward(&token_ids, None)?;
        
        // Sample and decode
        let next_token = self.sample_token(&logits);
        let response = self.tokenizer.decode(&[next_token as u32])
            .unwrap_or_else(|_| format!("token_{}", next_token));
            
        let elapsed = start.elapsed().as_secs_f64();
        
        Ok((response, elapsed))
    }
    
    /// Run Mamba inference and measure time
    pub fn mamba_inference(&self, prompt: &str) -> Result<(String, f64)> {
        let start = Instant::now();
        
        // Tokenize
        let encoding = self.tokenizer.encode(prompt).map_err(|e| sutra_core::SutraError::ComputeError(e.to_string()))?;
        let token_ids: Vec<usize> = encoding.ids.iter().map(|&id| (id as usize) % 2000).collect();
        
        // Generate
        let logits = self.mamba_model.forward(&token_ids)?;
        
        // Sample and decode
        let next_token = self.sample_token(&logits);
        let response = self.tokenizer.decode(&[next_token as u32])
            .unwrap_or_else(|_| format!("token_{}", next_token));
            
        let elapsed = start.elapsed().as_secs_f64();
        
        Ok((response, elapsed))
    }
    
    /// Generate longer text for demo purposes
    pub fn generate_text(&self, prompt: &str, length: usize, use_rwkv: bool) -> Result<(String, f64)> {
        let start = Instant::now();
        
        let encoding = self.tokenizer.encode(prompt).map_err(|e| sutra_core::SutraError::ComputeError(e.to_string()))?;
        let mut token_ids: Vec<usize> = encoding.ids.iter().map(|&id| (id as usize) % 2000).collect();
        
        for _ in 0..length {
            let logits = if use_rwkv {
                self.rwkv_chat.forward(&token_ids, None)?.0
            } else {
                self.mamba_model.forward(&token_ids)?
            };
            
            let next_token = self.sample_token(&logits);
            token_ids.push(next_token);
            
            // Stop on special tokens
            if next_token == 0 || next_token == 1 {
                break;
            }
        }
        
        // Decode generated tokens
        let generated_ids: Vec<u32> = token_ids[encoding.ids.len()..].iter()
            .map(|&id| id as u32).collect();
            
        let response = self.tokenizer.decode(&generated_ids)
            .unwrap_or_else(|_| "Generated text...".to_string());
            
        let elapsed = start.elapsed().as_secs_f64();
        
        Ok((response, elapsed))
    }
    
    fn sample_token(&self, logits: &[f32]) -> usize {
        // Simple sampling - find max logit
        logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    fn create_demo_tokenizer() -> Result<Tokenizer> {
        let mut vocab = VocabBuilder::new()
            .with_standard_special_tokens()
            .build();
        
        // Add common words for demo
        let demo_words = vec![
            // Common words
            "the", "and", "of", "to", "a", "in", "for", "is", "on", "that", "by", "this", "with", "i", "you", "it",
            "not", "or", "be", "are", "from", "at", "as", "your", "all", "any", "can", "had", "her", "was", "one",
            "our", "out", "day", "get", "has", "him", "his", "how", "man", "new", "now", "old", "see", "two", "way",
            "who", "boy", "did", "its", "let", "put", "say", "she", "too", "use",
            
            // AI/tech words for demo
            "ai", "model", "neural", "network", "data", "algorithm", "machine", "learning", "deep", "intelligence",
            "computer", "software", "code", "program", "system", "technology", "digital", "artificial", "robot",
            "automation", "prediction", "analysis", "pattern", "recognition", "classification", "optimization",
            
            // Demo phrases
            "hello", "world", "test", "example", "demo", "sample", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
            "rust", "programming", "language", "memory", "safety", "performance", "concurrent", "parallel", "async",
        ];
        
        for word in demo_words {
            vocab.add_token(word.to_string());
        }
        
        let config = BpeConfig {
            vocab,
            merges: Vec::new(),
            unk_token: "[UNK]".to_string(),
            byte_level: true,
        };
        
        Ok(Tokenizer::Bpe(BpeTokenizer::new(config)))
    }
}

impl Default for DemoModels {
    fn default() -> Self {
        Self::new()
    }
}