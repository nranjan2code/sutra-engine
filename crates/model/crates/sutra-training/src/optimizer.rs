//! Optimizers for training neural networks

use crate::error::Result;
use ndarray::{ArrayD, Zip};
use serde::{Deserialize, Serialize};

/// Optimizer trait
pub trait Optimizer {
    fn step(&mut self, params: &mut [ArrayD<f32>], grads: &[ArrayD<f32>]) -> Result<()>;
    fn zero_grad(&mut self);
    fn learning_rate(&self) -> f32;
    fn set_learning_rate(&mut self, lr: f32);
}

/// Adam optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdamConfig {
    pub learning_rate: f32,
    pub beta1: f32,
    pub beta2: f32,
    pub epsilon: f32,
    pub weight_decay: f32,
}

impl Default for AdamConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-3,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            weight_decay: 0.0,
        }
    }
}

/// Adam optimizer
pub struct Adam {
    config: AdamConfig,
    step_count: usize,
    m: Vec<ArrayD<f32>>, // First moment
    v: Vec<ArrayD<f32>>, // Second moment
}

impl Adam {
    pub fn new(config: AdamConfig, num_params: usize) -> Self {
        Self {
            config,
            step_count: 0,
            m: Vec::with_capacity(num_params),
            v: Vec::with_capacity(num_params),
        }
    }

    fn init_moments(&mut self, params: &[ArrayD<f32>]) {
        if self.m.is_empty() {
            for param in params {
                self.m.push(ArrayD::zeros(param.raw_dim()));
                self.v.push(ArrayD::zeros(param.raw_dim()));
            }
        }
    }
}

impl Optimizer for Adam {
    fn step(&mut self, params: &mut [ArrayD<f32>], grads: &[ArrayD<f32>]) -> Result<()> {
        self.init_moments(params);
        self.step_count += 1;

        let lr = self.config.learning_rate;
        let beta1 = self.config.beta1;
        let beta2 = self.config.beta2;
        let eps = self.config.epsilon;

        // Bias correction
        let bias_correction1 = 1.0 - beta1.powi(self.step_count as i32);
        let bias_correction2 = 1.0 - beta2.powi(self.step_count as i32);
        let step_size = lr * (bias_correction2.sqrt()) / bias_correction1;

        for (i, (param, grad)) in params.iter_mut().zip(grads.iter()).enumerate() {
            // Update biased first moment estimate
            Zip::from(&mut self.m[i])
                .and(grad)
                .for_each(|m, &g| *m = beta1 * *m + (1.0 - beta1) * g);

            // Update biased second moment estimate
            Zip::from(&mut self.v[i])
                .and(grad)
                .for_each(|v, &g| *v = beta2 * *v + (1.0 - beta2) * g * g);

            // Update parameters
            Zip::from(param)
                .and(&self.m[i])
                .and(&self.v[i])
                .for_each(|p, &m, &v| {
                    *p -= step_size * m / (v.sqrt() + eps);
                    if self.config.weight_decay > 0.0 {
                        *p -= lr * self.config.weight_decay * *p;
                    }
                });
        }

        Ok(())
    }

    fn zero_grad(&mut self) {
        // Gradients are managed externally
    }

    fn learning_rate(&self) -> f32 {
        self.config.learning_rate
    }

    fn set_learning_rate(&mut self, lr: f32) {
        self.config.learning_rate = lr;
    }
}

/// SGD optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgdConfig {
    pub learning_rate: f32,
    pub momentum: f32,
    pub weight_decay: f32,
    pub nesterov: bool,
}

impl Default for SgdConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-2,
            momentum: 0.0,
            weight_decay: 0.0,
            nesterov: false,
        }
    }
}

/// SGD optimizer
pub struct Sgd {
    config: SgdConfig,
    velocity: Vec<ArrayD<f32>>,
}

impl Sgd {
    pub fn new(config: SgdConfig, num_params: usize) -> Self {
        Self {
            config,
            velocity: Vec::with_capacity(num_params),
        }
    }

    fn init_velocity(&mut self, params: &[ArrayD<f32>]) {
        if self.velocity.is_empty() {
            for param in params {
                self.velocity.push(ArrayD::zeros(param.raw_dim()));
            }
        }
    }
}

impl Optimizer for Sgd {
    fn step(&mut self, params: &mut [ArrayD<f32>], grads: &[ArrayD<f32>]) -> Result<()> {
        self.init_velocity(params);

        let lr = self.config.learning_rate;
        let momentum = self.config.momentum;
        let weight_decay = self.config.weight_decay;

        for (i, (param, grad)) in params.iter_mut().zip(grads.iter()).enumerate() {
            let mut d_p = grad.clone();

            // Add weight decay
            if weight_decay > 0.0 {
                d_p = &d_p + &(param.mapv(|x| x * weight_decay));
            }

            // Apply momentum
            if momentum > 0.0 {
                Zip::from(&mut self.velocity[i])
                    .and(&d_p)
                    .for_each(|v, &g| *v = momentum * *v + g);

                if self.config.nesterov {
                    d_p = &d_p + &self.velocity[i].mapv(|x| x * momentum);
                } else {
                    d_p = self.velocity[i].clone();
                }
            }

            // Update parameters
            Zip::from(param).and(&d_p).for_each(|p, &g| *p -= lr * g);
        }

        Ok(())
    }

    fn zero_grad(&mut self) {
        // Gradients are managed externally
    }

    fn learning_rate(&self) -> f32 {
        self.config.learning_rate
    }

    fn set_learning_rate(&mut self, lr: f32) {
        self.config.learning_rate = lr;
    }
}

/// AdamW optimizer (Adam with decoupled weight decay)
pub type AdamW = Adam;

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr1;

    #[test]
    fn test_adam_step() {
        let config = AdamConfig::default();
        let mut optimizer = Adam::new(config, 1);

        let mut params = vec![arr1(&[1.0, 2.0, 3.0]).into_dyn()];
        let grads = vec![arr1(&[0.1, 0.2, 0.3]).into_dyn()];

        optimizer.step(&mut params, &grads).unwrap();

        // Parameters should have been updated
        assert_ne!(params[0][[0]], 1.0);
    }

    #[test]
    fn test_sgd_step() {
        let config = SgdConfig::default();
        let mut optimizer = Sgd::new(config, 1);

        let mut params = vec![arr1(&[1.0, 2.0, 3.0]).into_dyn()];
        let grads = vec![arr1(&[0.1, 0.2, 0.3]).into_dyn()];

        optimizer.step(&mut params, &grads).unwrap();

        // Parameters should have been updated
        assert_ne!(params[0][[0]], 1.0);
    }
}
