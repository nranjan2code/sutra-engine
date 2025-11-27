//! RWKV: Reinventing RNNs for Parallel Training
//!
//! RWKV combines the parallelizable training of Transformers with the
//! efficient inference of RNNs. Key advantages:
//! - Linear complexity in sequence length
//! - Constant memory during inference
//! - Runs efficiently on CPUs and edge devices
//! - No GPU overhead

pub mod attention;
pub mod ffn;
pub mod layer;
pub mod model;
pub mod state;

pub use layer::RwkvLayer;
pub use model::{RwkvConfig, RwkvModel};
pub use state::RwkvState;

pub mod prelude {
    pub use crate::{RwkvConfig, RwkvLayer, RwkvModel, RwkvState};
}
