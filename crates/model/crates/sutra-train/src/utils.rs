/// Utility functions for the training GUI
use std::path::Path;

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let unit_index = (bytes as f64).log(1024.0).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    
    let size = bytes as f64 / 1024_f64.powi(unit_index as i32);
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn validate_file_path(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn get_file_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

pub fn estimate_memory_usage(model_params: u64, quantized: bool, lora_enabled: bool, lora_rank: usize) -> f32 {
    // Base model memory (in GB)
    let base_memory = if quantized {
        // 4-bit quantization
        model_params as f32 * 0.5 / 1e9
    } else {
        // Full precision
        model_params as f32 * 4.0 / 1e9
    };

    // LoRA adapter memory (if enabled)
    let lora_memory = if lora_enabled {
        // Estimate LoRA parameters based on rank
        let hidden_size = (model_params as f32).sqrt() as u64; // Rough estimate
        let lora_params = hidden_size * lora_rank as u64 * 2; // A and B matrices
        lora_params as f32 * 4.0 / 1e9 // Full precision for adapters
    } else {
        0.0
    };

    // Optimizer states (Adam requires 2x parameter memory)
    let optimizer_memory = if lora_enabled {
        lora_memory * 2.0
    } else {
        base_memory * 2.0
    };

    // Activation memory (rough estimate)
    let activation_memory = 1.0; // ~1GB for intermediate activations

    base_memory + lora_memory + optimizer_memory + activation_memory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_format_duration() {
        use std::time::Duration;
        
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_memory_estimation() {
        let memory = estimate_memory_usage(500_000_000, true, true, 8);
        assert!(memory > 0.0);
        assert!(memory < 10.0); // Should be reasonable for a 500M model
    }
}