/// Interactive Chat Interface with RWKV
use eframe::egui;
use std::sync::Arc;
use std::collections::VecDeque;

use crate::models::DemoModels;

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub inference_time: Option<f64>,
}

pub struct ChatInterface {
    models: Arc<DemoModels>,
    messages: VecDeque<ChatMessage>,
    input_text: String,
    is_generating: bool,
    selected_model: ModelChoice,
    generation_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum ModelChoice {
    Rwkv,
    Mamba,
}

impl ChatInterface {
    pub fn new(models: Arc<DemoModels>) -> Self {
        let mut messages = VecDeque::new();
        
        // Welcome message
        messages.push_back(ChatMessage {
            role: "System".to_string(),
            content: "🦀 Welcome to SutraWorks AI Chat!\n\nThis is a pure Rust implementation with RWKV and Mamba models.\nType anything to see the AI respond in real-time!".to_string(),
            timestamp: Self::current_time(),
            inference_time: None,
        });
        
        Self {
            models,
            messages,
            input_text: String::new(),
            is_generating: false,
            selected_model: ModelChoice::Rwkv,
            generation_length: 10,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("💬 Interactive AI Chat");
        
        ui.horizontal(|ui| {
            ui.label("Model:");
            if ui.add(egui::RadioButton::new(self.selected_model == ModelChoice::Rwkv, "RWKV (RNN)")).clicked() {
                self.selected_model = ModelChoice::Rwkv;
            }
            if ui.add(egui::RadioButton::new(self.selected_model == ModelChoice::Mamba, "Mamba (SSM)")).clicked() {
                self.selected_model = ModelChoice::Mamba;
            }
            
            ui.separator();
            ui.label("Length:");
            ui.add(egui::Slider::new(&mut self.generation_length, 1..=50).text("tokens"));
        });
        
        ui.separator();
        
        // Chat history
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.messages {
                    self.show_message(ui, message);
                }
            });
        
        ui.separator();
        
        // Input area
        ui.horizontal(|ui| {
            let input_response = ui.add(
                egui::TextEdit::singleline(&mut self.input_text)
                    .hint_text("Type your message here...")
                    .desired_width(ui.available_width() - 100.0)
            );
            
            let send_button = ui.add_enabled(
                !self.input_text.trim().is_empty() && !self.is_generating,
                egui::Button::new(if self.is_generating { "⏳ Generating..." } else { "🚀 Send" })
            );
            
            if (input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                || send_button.clicked() {
                self.send_message(ctx);
            }
        });
        
        // Model info
        ui.separator();
        self.show_model_info(ui);
    }
    
    fn show_message(&self, ui: &mut egui::Ui, message: &ChatMessage) {
        let is_user = message.role == "User";
        let bg_color = if is_user {
            egui::Color32::from_rgb(40, 80, 120) // Terminal blue for user
        } else if message.role == "System" {
            egui::Color32::from_rgb(80, 80, 100) // Neutral gray for system
        } else if message.role == "Error" {
            egui::Color32::from_rgb(150, 40, 40) // Terminal red for errors
        } else {
            egui::Color32::from_rgb(40, 120, 60) // Terminal green for AI
        };
        
        let text_color = egui::Color32::from_rgb(240, 240, 240); // High contrast white text
        
        ui.horizontal(|ui| {
            if !is_user {
                ui.add_space(20.0);
            }
            
            egui::Frame::none()
                .fill(bg_color)
                .inner_margin(12.0)
                .rounding(6.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 120)))
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width() - 40.0);
                    
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 255, 255), &message.role);
                        ui.colored_label(egui::Color32::from_rgb(180, 180, 180), format!("• {}", message.timestamp));
                        
                        if let Some(time) = message.inference_time {
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("• {:.1}ms", time * 1000.0));
                        }
                    });
                    
                    ui.colored_label(text_color, &message.content);
                });
            
            if is_user {
                ui.add_space(20.0);
            }
        });
        
        ui.add_space(8.0);
    }
    
    fn send_message(&mut self, ctx: &egui::Context) {
        if self.input_text.trim().is_empty() || self.is_generating {
            return;
        }
        
        // Add user message
        self.messages.push_back(ChatMessage {
            role: "User".to_string(),
            content: self.input_text.clone(),
            timestamp: Self::current_time(),
            inference_time: None,
        });
        
        let prompt = self.input_text.clone();
        self.input_text.clear();
        self.is_generating = true;
        
        // Generate AI response
        let use_rwkv = self.selected_model == ModelChoice::Rwkv;
        match self.models.generate_text(&prompt, self.generation_length, use_rwkv) {
            Ok((response, inference_time)) => {
                let model_name = if use_rwkv { "RWKV" } else { "Mamba" };
                
                self.messages.push_back(ChatMessage {
                    role: model_name.to_string(),
                    content: if response.trim().is_empty() {
                        format!("✨ [Generated {} tokens with {} model]", self.generation_length, model_name)
                    } else {
                        response
                    },
                    timestamp: Self::current_time(),
                    inference_time: Some(inference_time),
                });
            }
            Err(e) => {
                self.messages.push_back(ChatMessage {
                    role: "Error".to_string(),
                    content: format!("Failed to generate: {}", e),
                    timestamp: Self::current_time(),
                    inference_time: None,
                });
            }
        }
        
        self.is_generating = false;
        ctx.request_repaint();
        
        // Keep only last 50 messages
        while self.messages.len() > 50 {
            self.messages.pop_front();
        }
    }
    
    fn show_model_info(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "🔄 RWKV Model");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Layers: {}", self.models.rwkv_config.num_layers));
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Hidden: {}", self.models.rwkv_config.hidden_size));
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), "Complexity: O(n)");
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), "Memory: Constant");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), "🚀 Mamba Model");
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Layers: {}", self.models.mamba_config.num_layers));
                    ui.colored_label(egui::Color32::from_rgb(200, 200, 200), format!("Hidden: {}", self.models.mamba_config.hidden_size));
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), "Complexity: O(n)");
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "Memory: Linear");
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "🦀 Pure Rust Benefits");
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "• Memory Safe");
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "• Zero Dependencies");
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "• Single Binary");
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "• Edge Deployment");
                });
            });
        });
    }
    
    fn current_time() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;
        let seconds = now % 60;
        
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}