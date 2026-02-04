//! Metrics collection and monitoring

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MetricsCollector {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn increment_counter(&self, name: &str, value: u64) {
        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(name.to_string()).or_insert(0) += value;
        }
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        if let Ok(mut gauges) = self.gauges.lock() {
            gauges.insert(name.to_string(), value);
        }
    }

    pub fn get_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();

        if let Ok(counters) = self.counters.lock() {
            for (key, value) in counters.iter() {
                metrics.insert(key.clone(), serde_json::json!(value));
            }
        }

        if let Ok(gauges) = self.gauges.lock() {
            for (key, value) in gauges.iter() {
                metrics.insert(key.clone(), serde_json::json!(value));
            }
        }

        metrics
    }
}
