//! Loss functions

use crate::error::Result;
use ndarray::ArrayD;

pub trait Loss {
    fn compute(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<f32>;
    fn backward(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<ArrayD<f32>>;
}

/// Cross-entropy loss
pub struct CrossEntropyLoss;

impl Loss for CrossEntropyLoss {
    fn compute(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<f32> {
        let mut loss = 0.0;
        for (pred, target) in predictions.iter().zip(targets.iter()) {
            loss += -target * pred.ln();
        }
        Ok(loss / predictions.len() as f32)
    }

    fn backward(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<ArrayD<f32>> {
        let scale = predictions.len() as f32;
        Ok((predictions - targets).mapv(|x| x / scale))
    }
}

/// Mean squared error loss
pub struct MSELoss;

impl Loss for MSELoss {
    fn compute(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<f32> {
        let diff = predictions - targets;
        Ok(diff.mapv(|x| x * x).sum() / predictions.len() as f32)
    }

    fn backward(&self, predictions: &ArrayD<f32>, targets: &ArrayD<f32>) -> Result<ArrayD<f32>> {
        let scale = predictions.len() as f32;
        Ok((predictions - targets).mapv(|x| x * 2.0 / scale))
    }
}
