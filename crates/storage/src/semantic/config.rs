use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticConfig {
    pub temporal: TemporalConfig,
    pub rules: RuleConfig,
    pub negation: NegationConfig,
    pub causal: CausalConfig,
    pub conditions: ConditionConfig,
    pub quantitative: QuantitativeConfig,
    pub definitional: DefinitionalConfig,
    pub events: EventConfig,
    pub domains: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalConfig {
    pub after: Vec<String>,
    pub before: Vec<String>,
    pub during: Vec<String>,
    pub between: Vec<String>,
    pub at: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleConfig {
    pub modal: Vec<String>,
    pub conditional: Vec<String>,
    pub imperative: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NegationConfig {
    pub explicit: Vec<String>,
    pub exception: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CausalConfig {
    pub direct: Vec<String>,
    pub enabling: Vec<String>,
    pub preventing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConditionConfig {
    pub if_clause: Vec<String>,
    pub when_clause: Vec<String>,
    pub unless_clause: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuantitativeConfig {
    pub number: Vec<String>,
    pub percentage: Vec<String>,
    pub measurement: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefinitionalConfig {
    pub is_a: Vec<String>,
    pub defined_as: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventConfig {
    pub past: Vec<String>,
    pub future: Vec<String>,
    pub ongoing: Vec<String>,
}
