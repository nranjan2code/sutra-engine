//! Neuro-Symbolic AI: Combining Neural and Symbolic Reasoning
//!
//! NeSy systems integrate:
//! - Neural networks (pattern recognition, language understanding)
//! - Symbolic AI (logic, math, guaranteed correctness)
//!
//! This antithesis to pure scaling enables:
//! - Reduced hallucinations via symbolic verification
//! - Exact computation for math/logic tasks
//! - Smaller models with external tool augmentation

pub mod agent;
pub mod executor;
pub mod tools;
pub mod verifier;

pub use agent::{AgentConfig, NesyAgent};
pub use executor::ToolExecutor;
pub use tools::{Tool, ToolRegistry, ToolResult};
pub use verifier::SymbolicVerifier;

pub mod prelude {
    pub use crate::{
        AgentConfig, NesyAgent, SymbolicVerifier, Tool, ToolExecutor, ToolRegistry, ToolResult,
    };
}
