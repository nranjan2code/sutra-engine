/// Live Quantization Demo
use eframe::egui;
use std::time::Instant;

use sutra_core::{Tensor, DType};
use sutra_quantize::{AwqQuantizer, AwqConfig};

pub struct QuantizationDemo {
    /// Configuration
    bits: u8,
    group_size: usize,
    
    /// Demo matrices
    matrix_size: usize,
    original_matrix: Option<Tensor>,
    quantized_result: Option<QuantizationResult>,
    
    /// Performance
    last_quantization_time: Option<f64>,
    is_quantizing: bool,
    
    /// Visualization
    show_weights: bool,
    weight_sample_size: usize,
}

struct QuantizationResult {
    compression_ratio: f32,
    memory_saved: usize,
    original_size: usize,
    quantized_size: usize,
    time_taken: f64,
}

impl QuantizationDemo {
    pub fn new() -> Self {
        Self {
            bits: 4,
            group_size: 128,
            matrix_size: 512,
            original_matrix: None,
            quantized_result: None,
            last_quantization_time: None,
            is_quantizing: false,
            show_weights: false,
            weight_sample_size: 20,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("⚡ Live Quantization Demo");
        
        // Controls
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.strong("⚙️ Configuration");
                    
                    ui.horizontal(|ui| {
                        ui.label("Bits:");
                        ui.add(egui::Slider::new(&mut self.bits, 2..=8));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Group Size:");
                        ui.add(egui::Slider::new(&mut self.group_size, 32..=512).step_by(32.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Matrix Size:");
                        ui.add(egui::Slider::new(&mut self.matrix_size, 256..=2048).step_by(256.0));
                    });
                });
            });
            
            ui.separator();
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.strong("🎯 Actions");
                    
                    if ui.add_enabled(!self.is_quantizing, egui::Button::new("🎲 Generate Random Matrix")).clicked() {
                        self.generate_matrix();
                    }
                    
                    if ui.add_enabled(
                        self.original_matrix.is_some() && !self.is_quantizing, 
                        egui::Button::new("⚡ Quantize Now!")
                    ).clicked() {
                        self.quantize_matrix();
                    }
                    
                    ui.checkbox(&mut self.show_weights, "👁️ Show Weight Values");
                });
            });
        });
        
        ui.separator();
        
        // Matrix info
        if let Some(ref matrix) = self.original_matrix {
            self.show_matrix_info(ui, matrix);
        }
        
        // Quantization results
        if let Some(ref result) = self.quantized_result {
            ui.separator();
            self.show_quantization_results(ui, result);
        }
        
        // Weight visualization
        if self.show_weights {
            if let Some(ref matrix) = self.original_matrix.clone() {
                ui.separator();
                self.show_weight_visualization(ui, matrix);
            }
        }
        
        // Compression explanation
        ui.separator();
        self.show_compression_explanation(ui);
    }
    
    fn generate_matrix(&mut self) {
        self.is_quantizing = true;
        
        // Generate random matrix
        let total_elements = self.matrix_size * self.matrix_size;
        let data: Vec<f32> = (0..total_elements)
            .map(|_| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                rng.gen_range(-1.0..1.0) * 0.1 // Small weights typical of neural networks
            })
            .collect();
            
        match Tensor::from_slice(&data, &[self.matrix_size, self.matrix_size], DType::F32) {
            Ok(matrix) => {
                self.original_matrix = Some(matrix);
                self.quantized_result = None; // Clear previous results
            }
            Err(e) => {
                eprintln!("Failed to create matrix: {}", e);
            }
        }
        
        self.is_quantizing = false;
    }
    
    fn quantize_matrix(&mut self) {
        if let Some(ref matrix) = self.original_matrix {
            self.is_quantizing = true;
            
            let start = Instant::now();
            
            let config = AwqConfig {
                bits: self.bits,
                group_size: self.group_size,
                n_samples: 512,
                zero_point: true,
            };
            
            let quantizer = AwqQuantizer::new(config);
            
            match quantizer.quantize(matrix, None) {
                Ok(quantized) => {
                    let time_taken = start.elapsed().as_secs_f64();
                    let compression_ratio = quantized.compression_ratio();
                    let original_size = matrix.memory_usage();
                    let quantized_size = quantized.memory_usage();
                    let memory_saved = original_size.saturating_sub(quantized_size);
                    
                    self.quantized_result = Some(QuantizationResult {
                        compression_ratio,
                        memory_saved,
                        original_size,
                        quantized_size,
                        time_taken,
                    });
                    
                    self.last_quantization_time = Some(time_taken);
                }
                Err(e) => {
                    eprintln!("Quantization failed: {}", e);
                }
            }
            
            self.is_quantizing = false;
        }
    }
    
    fn show_matrix_info(&self, ui: &mut egui::Ui, matrix: &Tensor) {
        ui.group(|ui| {
            ui.strong("📊 Original Matrix Info");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("Shape: {:?}", matrix.shape()));
                    ui.label(format!("Elements: {}", matrix.data().len()));
                    ui.label(format!("Memory: {:.2} MB", matrix.memory_usage() as f64 / 1_048_576.0));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    // Calculate statistics
                    let data = matrix.data().as_slice().unwrap();
                    let min_val = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                    let max_val = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                    let mean = data.iter().sum::<f32>() / data.len() as f32;
                    
                    ui.label(format!("Min: {:.6}", min_val));
                    ui.label(format!("Max: {:.6}", max_val));
                    ui.label(format!("Mean: {:.6}", mean));
                });
            });
        });
    }
    
    fn show_quantization_results(&self, ui: &mut egui::Ui, result: &QuantizationResult) {
        ui.group(|ui| {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "⚡ Quantization Results");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 100),
                        format!("🗜️ Compression: {:.2}x", result.compression_ratio)
                    );
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), format!("⏱️ Time: {:.2}ms", result.time_taken * 1000.0));
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("💾 Saved: {:.2} MB", result.memory_saved as f64 / 1_048_576.0));
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Original: {:.2} MB", result.original_size as f64 / 1_048_576.0));
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Quantized: {:.2} MB", result.quantized_size as f64 / 1_048_576.0));
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), format!("Reduction: {:.1}%", 
                        (result.memory_saved as f64 / result.original_size as f64) * 100.0));
                });
            });
            
            // Progress bar showing compression
            let compression_progress = 1.0 - (result.quantized_size as f32 / result.original_size as f32);
            ui.add(egui::ProgressBar::new(compression_progress)
                .text(format!("{:.1}% compressed", compression_progress * 100.0)));
        });
    }
    
    fn show_weight_visualization(&mut self, ui: &mut egui::Ui, matrix: &Tensor) {
        ui.group(|ui| {
            ui.strong("👁️ Weight Visualization (Sample)");
            
            ui.horizontal(|ui| {
                ui.label("Sample size:");
                ui.add(egui::Slider::new(&mut self.weight_sample_size, 10..=100));
            });
            
            if let Some(data) = matrix.data().as_slice() {
                let step = (data.len() / self.weight_sample_size).max(1);
                let sample: Vec<f32> = data.iter().step_by(step).take(self.weight_sample_size).cloned().collect();
                
                ui.horizontal_wrapped(|ui| {
                    for (i, &weight) in sample.iter().enumerate() {
                        let color = if weight > 0.0 {
                            egui::Color32::from_rgb(100, 200, 255) // Blue for positive
                        } else {
                            egui::Color32::from_rgb(255, 120, 120) // Red for negative
                        };
                        
                        ui.colored_label(color, format!("{:.3}", weight));
                        
                        if (i + 1) % 10 == 0 {
                            ui.end_row();
                        }
                    }
                });
            }
        });
    }
    
    fn show_compression_explanation(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.strong("🧠 How AWQ Quantization Works");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("📉 Bit Reduction:");
                    ui.label(format!("• FP32: 32 bits → INT{}: {} bits", self.bits, self.bits));
                    ui.label(format!("• Theoretical: {:.1}x compression", 32.0 / self.bits as f32));
                    ui.label("• Bit-packing: 2 values per byte (4-bit)");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("🎯 AWQ Features:");
                    ui.label("• Activation-aware scaling");
                    ui.label("• Salient weight protection");
                    ui.label("• Group-wise quantization");
                    ui.label("• Zero-point optimization");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label("⚡ Benefits:");
                    ui.label("• Reduced memory usage");
                    ui.label("• Faster inference");
                    ui.label("• Edge device deployment");
                    ui.label("• Maintained accuracy");
                });
            });
        });
    }
}