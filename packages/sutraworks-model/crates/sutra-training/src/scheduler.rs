//! Learning rate schedulers

pub trait LRScheduler {
    fn get_lr(&self, step: usize) -> f32;
}

/// Cosine annealing scheduler
pub struct CosineScheduler {
    initial_lr: f32,
    min_lr: f32,
    total_steps: usize,
}

impl CosineScheduler {
    pub fn new(initial_lr: f32, min_lr: f32, total_steps: usize) -> Self {
        Self {
            initial_lr,
            min_lr,
            total_steps,
        }
    }
}

impl LRScheduler for CosineScheduler {
    fn get_lr(&self, step: usize) -> f32 {
        let progress = (step as f32 / self.total_steps as f32).min(1.0);
        let cosine = 0.5 * (1.0 + (std::f32::consts::PI * progress).cos());
        self.min_lr + (self.initial_lr - self.min_lr) * cosine
    }
}

/// Linear warmup + decay scheduler
pub struct LinearScheduler {
    initial_lr: f32,
    warmup_steps: usize,
    total_steps: usize,
}

impl LinearScheduler {
    pub fn new(initial_lr: f32, warmup_steps: usize, total_steps: usize) -> Self {
        Self {
            initial_lr,
            warmup_steps,
            total_steps,
        }
    }
}

impl LRScheduler for LinearScheduler {
    fn get_lr(&self, step: usize) -> f32 {
        if step < self.warmup_steps {
            self.initial_lr * (step as f32 / self.warmup_steps as f32)
        } else {
            let progress = ((step - self.warmup_steps) as f32)
                / ((self.total_steps - self.warmup_steps) as f32);
            self.initial_lr * (1.0 - progress).max(0.0)
        }
    }
}
