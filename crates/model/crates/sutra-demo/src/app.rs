/// Main Demo Application - Interactive AI Playground
use eframe::egui;
use std::sync::Arc;
use std::time::Instant;

use crate::chat::ChatInterface;
use crate::comparison::ArchitectureComparison;
use crate::models::DemoModels;
use crate::qa::QAInterface;
use crate::quantization_demo::QuantizationDemo;

#[derive(Debug, Clone, PartialEq)]
pub enum DemoTab {
    Chat,
    QA,
    Comparison,
    Quantization,
    NeuralSymbolic,
}

pub struct DemoApp {
    /// Current active tab
    active_tab: DemoTab,
    
    /// Pre-loaded demo models
    models: Arc<DemoModels>,
    
    /// Chat interface
    chat: ChatInterface,
    
    /// Q&A interface
    qa: QAInterface,
    
    /// Architecture comparison
    comparison: ArchitectureComparison,
    
    /// Quantization demo
    quantization: QuantizationDemo,
    
    /// App startup time for performance metrics
    startup_time: Instant,
    
    /// Demo statistics
    inference_count: usize,
    total_inference_time: f64,
}

impl DemoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let startup_time = Instant::now();
        
        println!("🚀 Initializing SutraWorks AI Demo...");
        
        // Initialize models
        let models = Arc::new(DemoModels::new());
        
        Self {
            active_tab: DemoTab::Chat,
            models: models.clone(),
            chat: ChatInterface::new(models.clone()),
            qa: QAInterface::new(models.clone()),
            comparison: ArchitectureComparison::new(models.clone()),
            quantization: QuantizationDemo::new(),
            startup_time,
            inference_count: 0,
            total_inference_time: 0.0,
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top header with branding and stats
        self.show_header(ctx);
        
        // Tab selection
        self.show_tab_bar(ctx);
        
        // Main content based on active tab
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                DemoTab::Chat => self.chat.show(ui, ctx),
                DemoTab::QA => self.qa.show(ui, ctx),
                DemoTab::Comparison => self.comparison.show(ui, ctx),
                DemoTab::Quantization => self.quantization.show(ui, ctx),
                DemoTab::NeuralSymbolic => self.show_neural_symbolic(ui),
            }
        });
        
        // Request repaint for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

impl DemoApp {
    fn show_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo/Title with bright colors
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "🦀 SutraWorks AI Demo");
                ui.separator();
                
                // Performance stats with color coding
                let uptime = self.startup_time.elapsed().as_secs();
                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("⏱️ Uptime: {}s", uptime));
                ui.separator();
                
                ui.colored_label(egui::Color32::from_rgb(100, 255, 150), format!("🔄 Inferences: {}", self.inference_count));
                ui.separator();
                
                if self.inference_count > 0 {
                    let avg_time = self.total_inference_time / self.inference_count as f64;
                    ui.colored_label(egui::Color32::from_rgb(255, 255, 100), format!("⚡ Avg: {:.1}ms", avg_time * 1000.0));
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), "Pure Rust • No PyTorch • Edge Optimized");
                });
            });
        });
    }
    
    fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 4.0;
                
                if ui.add(egui::Button::new("💬 Chat with RWKV")
                    .selected(self.active_tab == DemoTab::Chat)).clicked() {
                    self.active_tab = DemoTab::Chat;
                }
                
                if ui.add(egui::Button::new("🤔 Q&A Assistant")
                    .selected(self.active_tab == DemoTab::QA)).clicked() {
                    self.active_tab = DemoTab::QA;
                }
                
                if ui.add(egui::Button::new("🏎️ Architecture Race")
                    .selected(self.active_tab == DemoTab::Comparison)).clicked() {
                    self.active_tab = DemoTab::Comparison;
                }
                
                if ui.add(egui::Button::new("⚡ Live Quantization")
                    .selected(self.active_tab == DemoTab::Quantization)).clicked() {
                    self.active_tab = DemoTab::Quantization;
                }
                
                if ui.add(egui::Button::new("🧠 Neuro-Symbolic")
                    .selected(self.active_tab == DemoTab::NeuralSymbolic)).clicked() {
                    self.active_tab = DemoTab::NeuralSymbolic;
                }
            });
        });
    }
    
    fn show_neural_symbolic(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧠 Neuro-Symbolic Reasoning");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.label("🤖 Neural Component (Pattern Recognition)");
                    ui.text_edit_multiline(&mut "What's the weather like?".to_string());
                    if ui.button("🔍 Analyze Intent").clicked() {
                        // Demo intent recognition
                    }
                });
                
                ui.group(|ui| {
                    ui.label("⚙️ Symbolic Component (Logical Reasoning)");
                    ui.label("Detected Intent: weather_query");
                    ui.label("Required Tools: [weather_api, location_service]");
                    ui.label("Confidence: 95%");
                });
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.label("🛠️ Available Tools");
                    ui.label("• Calculator");
                    ui.label("• Weather API"); 
                    ui.label("• Web Search");
                    ui.label("• Code Executor");
                    
                    if ui.button("🚀 Execute Reasoning Chain").clicked() {
                        // Demo tool execution
                    }
                });
                
                ui.group(|ui| {
                    ui.label("✅ Verification");
                    ui.label("Logic check: PASSED");
                    ui.label("Fact verification: PENDING");
                    ui.label("Safety check: PASSED");
                });
            });
        });
        
        ui.separator();
        ui.colored_label(egui::Color32::LIGHT_GREEN, 
            "💡 Neuro-symbolic combines pattern recognition with guaranteed logical correctness");
    }
    
    pub fn record_inference(&mut self, time_taken: f64) {
        self.inference_count += 1;
        self.total_inference_time += time_taken;
    }
}