//! Model compression through quantization
//!
//! Implements AWQ (Activation-aware Weight Quantization) for 4-bit model compression.
//! AWQ preserves model accuracy by protecting salient weights during quantization.

pub mod awq;
pub mod dequantizer;
pub mod quantizer;
pub mod quantized_ops;

pub use awq::{AwqConfig, AwqQuantizer, QuantizedWeights};
pub use dequantizer::Dequantizer;
pub use quantizer::{QuantizedTensor, Quantizer};
pub use quantized_ops::{quantized_matmul, quantized_linear, calibrate_quantization};

pub mod prelude {
    pub use crate::{
        AwqConfig, AwqQuantizer, Dequantizer, QuantizedTensor, Quantizer, QuantizedWeights,
        quantized_matmul, quantized_linear, calibrate_quantization,
    };
}
