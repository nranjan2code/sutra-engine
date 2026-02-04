/// Data management for training
use std::path::PathBuf;
use anyhow::Result;

pub struct DataManager {
    train_files: Vec<PathBuf>,
    validation_files: Vec<PathBuf>,
    test_files: Vec<PathBuf>,
    sample_count: usize,
}

impl DataManager {
    pub fn new() -> Self {
        Self {
            train_files: Vec::new(),
            validation_files: Vec::new(),
            test_files: Vec::new(),
            sample_count: 0,
        }
    }

    pub fn has_data(&self) -> bool {
        !self.train_files.is_empty()
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn add_training_file(&mut self, path: PathBuf) -> Result<()> {
        self.train_files.push(path);
        self.update_sample_count()?;
        Ok(())
    }

    pub fn add_validation_file(&mut self, path: PathBuf) -> Result<()> {
        self.validation_files.push(path);
        Ok(())
    }

    pub fn clear_all(&mut self) {
        self.train_files.clear();
        self.validation_files.clear();
        self.test_files.clear();
        self.sample_count = 0;
    }

    pub fn get_training_files(&self) -> &[PathBuf] {
        &self.train_files
    }

    pub fn get_validation_files(&self) -> &[PathBuf] {
        &self.validation_files
    }

    fn update_sample_count(&mut self) -> Result<()> {
        // Count samples by reading files
        let mut total_samples = 0;
        
        for file_path in &self.train_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // Count based on file format
                let extension = file_path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                
                match extension {
                    "jsonl" => {
                        // Count newlines for JSONL
                        total_samples += content.lines().filter(|line| !line.trim().is_empty()).count();
                    }
                    "csv" => {
                        // Count lines minus header for CSV
                        let lines = content.lines().count();
                        total_samples += lines.saturating_sub(1);
                    }
                    "txt" => {
                        // For text files, estimate by splitting on common delimiters
                        let content = std::fs::read_to_string(file_path).unwrap_or_default();
                        let lines = content.lines().count();
                        total_samples += lines;
                    }
                    "json" => {
                        // For JSON files, count as single sample
                        total_samples += 1;
                    }
                    _ => {
                        // For unknown formats, estimate by splitting on common delimiters
                        // or count as single sample for JSON
                        if extension == "json" {
                            total_samples += 1;
                        } else {
                            // Estimate paragraphs or entries
                            total_samples += content.split("\n\n").filter(|s| !s.trim().is_empty()).count();
                        }
                    }
                }
            }
        }
        
        self.sample_count = total_samples.max(1); // At least 1 sample
        Ok(())
    }

    pub fn validate_data(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        if self.train_files.is_empty() {
            warnings.push("No training data loaded".to_string());
        }

        if self.validation_files.is_empty() && self.train_files.len() == 1 {
            warnings.push("No validation data - will use 10% of training data".to_string());
        }

        Ok(warnings)
    }

    pub fn estimate_training_time(&self, epochs: usize, batch_size: usize) -> std::time::Duration {
        // Rough estimation based on sample count and hardware
        let samples_per_epoch = self.sample_count;
        let batches_per_epoch = samples_per_epoch.div_ceil(batch_size);
        let total_batches = batches_per_epoch * epochs;
        
        // Estimate ~100ms per batch on M1/M2 Mac
        let seconds = (total_batches as f64 * 0.1).max(60.0); // Minimum 1 minute
        std::time::Duration::from_secs_f64(seconds)
    }
}