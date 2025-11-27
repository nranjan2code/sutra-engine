/// Architecture Performance Comparison
use eframe::egui;
use std::sync::Arc;

use crate::models::DemoModels;

pub struct ArchitectureComparison {
    models: Arc<DemoModels>,
    test_input: String,
    last_rwkv_time: Option<f64>,
    last_mamba_time: Option<f64>,
    benchmark_results: Vec<BenchmarkResult>,
    is_running: bool,
}

#[derive(Clone)]
struct BenchmarkResult {
    sequence_length: usize,
    rwkv_time: f64,
    mamba_time: f64,
    rwkv_throughput: f64,
    mamba_throughput: f64,
}

impl ArchitectureComparison {
    pub fn new(models: Arc<DemoModels>) -> Self {
        Self {
            models,
            test_input: "The quick brown fox jumps over the lazy dog".to_string(),
            last_rwkv_time: None,
            last_mamba_time: None,
            benchmark_results: Vec::new(),
            is_running: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("🏎️ Architecture Performance Race");
        
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "🔄 RWKV (Recurrent)");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Linear complexity: O(n)");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Constant memory usage");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• RNN-style processing");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Sequential by nature");
                    
                    if let Some(time) = self.last_rwkv_time {
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 255, 150),
                            format!("⚡ Last: {:.2}ms", time * 1000.0)
                        );
                    }
                });
            });
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), "🚀 Mamba (State Space)");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Linear complexity: O(n)");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Selective attention");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Hardware-aware design");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "• Optimized scan operations");
                    
                    if let Some(time) = self.last_mamba_time {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 150, 100),
                            format!("⚡ Last: {:.2}ms", time * 1000.0)
                        );
                    }
                });
            });
        });
        
        ui.separator();
        
        // Input and controls
        ui.horizontal(|ui| {
            ui.label("Test Input:");
            ui.text_edit_singleline(&mut self.test_input);
        });
        
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.is_running, egui::Button::new("🎯 Single Shot Comparison")).clicked() {
                self.run_single_comparison();
            }
            
            if ui.add_enabled(!self.is_running, egui::Button::new("📊 Run Benchmark Suite")).clicked() {
                self.run_benchmark_suite();
            }
            
            if ui.button("📊 Clear Results").clicked() {
                self.benchmark_results.clear();
                self.last_rwkv_time = None;
                self.last_mamba_time = None;
            }
        });
        
        ui.separator();
        
        // Results display
        if let (Some(rwkv_time), Some(mamba_time)) = (self.last_rwkv_time, self.last_mamba_time) {
            self.show_comparison_results(ui, rwkv_time, mamba_time);
        }
        
        if !self.benchmark_results.is_empty() {
            ui.separator();
            self.show_benchmark_table(ui);
        }
        
        // Theoretical comparison
        ui.separator();
        self.show_theoretical_comparison(ui);
    }
    
    fn run_single_comparison(&mut self) {
        self.is_running = true;
        
        // Test RWKV
        if let Ok((_response, rwkv_time)) = self.models.rwkv_inference(&self.test_input) {
            self.last_rwkv_time = Some(rwkv_time);
        }
        
        // Test Mamba
        if let Ok((_response, mamba_time)) = self.models.mamba_inference(&self.test_input) {
            self.last_mamba_time = Some(mamba_time);
        }
        
        self.is_running = false;
    }
    
    fn run_benchmark_suite(&mut self) {
        self.is_running = true;
        self.benchmark_results.clear();
        
        // Test different sequence lengths
        let test_lengths = vec![10, 25, 50, 100];
        
        for &length in &test_lengths {
            // Create input of specific length
            let words: Vec<&str> = self.test_input.split_whitespace().collect();
            let mut test_input = String::new();
            
            for i in 0..length {
                if i > 0 { test_input.push(' '); }
                test_input.push_str(words[i % words.len()]);
            }
            
            // Benchmark both models
            let rwkv_time = self.models.rwkv_inference(&test_input)
                .map(|(_, time)| time)
                .unwrap_or(0.0);
                
            let mamba_time = self.models.mamba_inference(&test_input)
                .map(|(_, time)| time)
                .unwrap_or(0.0);
            
            // Calculate throughput (tokens per second)
            let rwkv_throughput = if rwkv_time > 0.0 { length as f64 / rwkv_time } else { 0.0 };
            let mamba_throughput = if mamba_time > 0.0 { length as f64 / mamba_time } else { 0.0 };
            
            self.benchmark_results.push(BenchmarkResult {
                sequence_length: length,
                rwkv_time,
                mamba_time,
                rwkv_throughput,
                mamba_throughput,
            });
        }
        
        self.is_running = false;
    }
    
    fn show_comparison_results(&self, ui: &mut egui::Ui, rwkv_time: f64, mamba_time: f64) {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "⚡ Performance Results");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("RWKV: {:.2}ms", rwkv_time * 1000.0));
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), format!("Mamba: {:.2}ms", mamba_time * 1000.0));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    let speedup = if mamba_time > 0.0 { rwkv_time / mamba_time } else { 1.0 };
                    let (winner, winner_color) = if rwkv_time < mamba_time { 
                        ("RWKV", egui::Color32::from_rgb(100, 200, 255))
                    } else { 
                        ("Mamba", egui::Color32::from_rgb(255, 150, 100))
                    };
                    
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), format!("Winner: {}", winner));
                    ui.colored_label(winner_color, format!("Speedup: {:.2}x", speedup.max(1.0/speedup)));
                });
            });
        });
    }
    
    fn show_benchmark_table(&self, ui: &mut egui::Ui) {
        ui.strong("📊 Benchmark Results");
        
        egui::Grid::new("benchmark_grid")
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.strong("Seq Length");
                ui.strong("RWKV (ms)");
                ui.strong("Mamba (ms)");
                ui.strong("RWKV (tok/s)");
                ui.strong("Mamba (tok/s)");
                ui.strong("Winner");
                ui.end_row();
                
                // Data rows
                for result in &self.benchmark_results {
                    ui.label(result.sequence_length.to_string());
                    ui.label(format!("{:.2}", result.rwkv_time * 1000.0));
                    ui.label(format!("{:.2}", result.mamba_time * 1000.0));
                    ui.label(format!("{:.0}", result.rwkv_throughput));
                    ui.label(format!("{:.0}", result.mamba_throughput));
                    
                    let (winner, winner_color) = if result.rwkv_time < result.mamba_time { 
                        ("RWKV", egui::Color32::from_rgb(100, 200, 255))
                    } else { 
                        ("Mamba", egui::Color32::from_rgb(255, 150, 100))
                    };
                    
                    ui.colored_label(winner_color, winner);
                    ui.end_row();
                }
            });
    }
    
    fn show_theoretical_comparison(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.strong("🧠 Theoretical Analysis");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("📈 Complexity Comparison:");
                    ui.label("• Both: O(n) time complexity");
                    ui.label("• RWKV: O(1) memory per step");
                    ui.label("• Mamba: O(n) memory total");
                    ui.label("• Both: Linear scaling advantage");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("⚡ vs Transformer:");
                    
                    let seq_len = 2048;
                    let transformer_ops = seq_len * seq_len;
                    let linear_ops = seq_len;
                    let speedup = transformer_ops as f64 / linear_ops as f64;
                    
                    ui.label(format!("Transformer: O(n²) = {}", transformer_ops));
                    ui.label(format!("RWKV/Mamba: O(n) = {}", linear_ops));
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("Theoretical speedup: {:.0}x", speedup)
                    );
                });
            });
        });
    }
}