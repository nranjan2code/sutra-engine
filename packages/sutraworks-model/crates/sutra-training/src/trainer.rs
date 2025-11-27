//! Training loop implementation

use crate::error::{Result, TrainingError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerConfig {
    pub epochs: usize,
    pub batch_size: usize,
    pub gradient_accumulation_steps: usize,
    pub max_grad_norm: Option<f32>,
    pub eval_steps: usize,
    pub save_steps: usize,
    pub logging_steps: usize,
    pub output_dir: String,
}

impl Default for TrainerConfig {
    fn default() -> Self {
        Self {
            epochs: 10,
            batch_size: 32,
            gradient_accumulation_steps: 1,
            max_grad_norm: Some(1.0),
            eval_steps: 500,
            save_steps: 1000,
            logging_steps: 100,
            output_dir: "./output".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingState {
    pub epoch: usize,
    pub global_step: usize,
    pub train_loss: f32,
    pub eval_loss: Option<f32>,
    pub learning_rate: f32,
}

impl TrainingState {
    pub fn new() -> Self {
        Self {
            epoch: 0,
            global_step: 0,
            train_loss: 0.0,
            eval_loss: None,
            learning_rate: 0.0,
        }
    }
}

impl Default for TrainingState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Trainer {
    #[allow(dead_code)]
    config: TrainerConfig,
    state: TrainingState,
}

impl Trainer {
    pub fn new(config: TrainerConfig) -> Self {
        std::fs::create_dir_all(&config.output_dir).ok();

        Self {
            config,
            state: TrainingState::new(),
        }
    }

    pub fn state(&self) -> &TrainingState {
        &self.state
    }

    pub fn save_checkpoint(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state)
            .map_err(|e| TrainingError::Checkpoint(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_checkpoint(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        self.state =
            serde_json::from_str(&json).map_err(|e| TrainingError::Checkpoint(e.to_string()))?;
        Ok(())
    }

    pub fn log(&self, message: impl AsRef<str>) {
        println!("[Step {}] {}", self.state.global_step, message.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trainer_creation() {
        let config = TrainerConfig::default();
        let trainer = Trainer::new(config);
        assert_eq!(trainer.state().epoch, 0);
    }
}
