//! Mamba: Linear-Time Sequence Modeling with Selective State Spaces
//!
//! Mamba achieves Transformer-quality performance with linear complexity:
//! - O(n) time complexity (vs O(n²) for Transformers)
//! - 5x higher throughput than Transformers
//! - Scales linearly with sequence length
//! - Selective state space mechanism

pub mod layer;
pub mod model;
pub mod selective;
pub mod ssm;

pub use layer::MambaLayer;
pub use model::{MambaConfig, MambaModel};
pub use ssm::StateSpaceModel;

pub mod prelude {
    pub use crate::{MambaConfig, MambaLayer, MambaModel, StateSpaceModel};
}
