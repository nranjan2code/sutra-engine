/// Training progress tracking
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgress {
    pub current_epoch: usize,
    pub total_epochs: usize,
    pub current_step: usize,
    pub total_steps: usize,
    pub train_loss: f32,
    pub validation_loss: Option<f32>,
    pub learning_rate: f32,
    #[serde(skip)]
    pub start_time: Option<Instant>,
    pub estimated_time_remaining: Option<Duration>,
    pub metrics: TrainingMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub loss_history: Vec<f32>,
    pub validation_loss_history: Vec<f32>,
    pub learning_rate_history: Vec<f32>,
    pub perplexity: Option<f32>,
    pub throughput: Option<f32>, // tokens/second
    pub memory_usage: f32, // GB
}

impl TrainingProgress {
    pub fn new() -> Self {
        Self {
            current_epoch: 0,
            total_epochs: 0,
            current_step: 0,
            total_steps: 0,
            train_loss: 0.0,
            validation_loss: None,
            learning_rate: 0.0,
            start_time: None,
            estimated_time_remaining: None,
            metrics: TrainingMetrics::new(),
        }
    }

    pub fn start_training(&mut self, total_epochs: usize, total_steps: usize) {
        self.current_epoch = 0;
        self.total_epochs = total_epochs;
        self.current_step = 0;
        self.total_steps = total_steps;
        self.start_time = Some(Instant::now());
        self.metrics.clear();
    }

    pub fn update_step(&mut self, step: usize, loss: f32, learning_rate: f32) {
        self.current_step = step;
        self.train_loss = loss;
        self.learning_rate = learning_rate;
        self.metrics.loss_history.push(loss);
        self.metrics.learning_rate_history.push(learning_rate);
        
        self.update_eta();
    }

    pub fn update_epoch(&mut self, epoch: usize) {
        self.current_epoch = epoch;
    }

    pub fn update_validation(&mut self, val_loss: f32) {
        self.validation_loss = Some(val_loss);
        self.metrics.validation_loss_history.push(val_loss);
    }

    pub fn progress(&self) -> f32 {
        if self.total_steps == 0 {
            0.0
        } else {
            self.current_step as f32 / self.total_steps as f32
        }
    }

    pub fn epoch_progress(&self) -> f32 {
        if self.total_epochs == 0 {
            0.0
        } else {
            self.current_epoch as f32 / self.total_epochs as f32
        }
    }

    fn update_eta(&mut self) {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            let progress = self.progress();
            
            if progress > 0.0 && progress < 1.0 {
                let total_estimated = elapsed.as_secs_f64() / progress as f64;
                let remaining = total_estimated - elapsed.as_secs_f64();
                self.estimated_time_remaining = Some(Duration::from_secs_f64(remaining.max(0.0)));
            }
        }
    }

    pub fn format_eta(&self) -> String {
        match self.estimated_time_remaining {
            Some(eta) => {
                let total_secs = eta.as_secs();
                let hours = total_secs / 3600;
                let minutes = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;
                
                if hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            None => "Unknown".to_string(),
        }
    }

    pub fn is_training(&self) -> bool {
        self.start_time.is_some() && self.progress() < 1.0
    }
}

impl TrainingMetrics {
    pub fn new() -> Self {
        Self {
            loss_history: Vec::new(),
            validation_loss_history: Vec::new(),
            learning_rate_history: Vec::new(),
            perplexity: None,
            throughput: None,
            memory_usage: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.loss_history.clear();
        self.validation_loss_history.clear();
        self.learning_rate_history.clear();
        self.perplexity = None;
        self.throughput = None;
        self.memory_usage = 0.0;
    }

    pub fn latest_loss(&self) -> Option<f32> {
        self.loss_history.last().copied()
    }

    pub fn latest_val_loss(&self) -> Option<f32> {
        self.validation_loss_history.last().copied()
    }

    pub fn loss_trend(&self) -> Option<f32> {
        if self.loss_history.len() < 10 {
            return None;
        }

        let recent: Vec<f32> = self.loss_history.iter().rev().take(10).copied().collect();
        let early_avg = recent[5..].iter().sum::<f32>() / 5.0;
        let recent_avg = recent[..5].iter().sum::<f32>() / 5.0;

        Some((recent_avg - early_avg) / early_avg)
    }
}