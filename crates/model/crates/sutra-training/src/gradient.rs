//! Gradient accumulation utilities

use crate::error::Result;
use ndarray::ArrayD;

pub struct GradientAccumulator {
    gradients: Vec<ArrayD<f32>>,
    accumulation_steps: usize,
    current_step: usize,
}

impl GradientAccumulator {
    pub fn new(num_params: usize, accumulation_steps: usize) -> Self {
        Self {
            gradients: Vec::with_capacity(num_params),
            accumulation_steps,
            current_step: 0,
        }
    }

    pub fn accumulate(&mut self, grads: &[ArrayD<f32>]) -> Result<()> {
        if self.gradients.is_empty() {
            self.gradients = grads.to_vec();
        } else {
            for (acc, grad) in self.gradients.iter_mut().zip(grads.iter()) {
                *acc = &*acc + grad;
            }
        }
        self.current_step += 1;
        Ok(())
    }

    pub fn should_step(&self) -> bool {
        self.current_step >= self.accumulation_steps
    }

    pub fn get_accumulated(&mut self) -> Vec<ArrayD<f32>> {
        let scale = 1.0 / self.accumulation_steps as f32;
        let result: Vec<ArrayD<f32>> = self
            .gradients
            .iter()
            .map(|g| g.mapv(|x| x * scale))
            .collect();

        self.gradients.clear();
        self.current_step = 0;
        result
    }
}
