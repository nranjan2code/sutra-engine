/// Q&A Interface - Ask questions and get intelligent responses
use eframe::egui;
use std::sync::Arc;
use std::collections::VecDeque;

use crate::models::DemoModels;

#[derive(Clone)]
pub struct QAMessage {
    pub question: String,
    pub answer: String,
    pub timestamp: String,
    pub model_used: String,
    pub confidence: f32,
    pub inference_time: f64,
}

pub struct QAInterface {
    models: Arc<DemoModels>,
    qa_history: VecDeque<QAMessage>,
    input_question: String,
    is_processing: bool,
    selected_model: ModelChoice,
    response_style: ResponseStyle,
    suggested_questions: Vec<String>,
    context_window: Vec<String>,
    auto_suggest: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ModelChoice {
    Rwkv,
    Mamba,
    Auto, // Automatically choose best model based on question type
}

#[derive(Debug, Clone, PartialEq)]
enum ResponseStyle {
    Detailed,   // Longer, comprehensive answers
    Concise,    // Brief, to-the-point answers
    Creative,   // More creative, expressive responses
    Technical,  // Focus on technical accuracy
}

impl QAInterface {
    pub fn new(models: Arc<DemoModels>) -> Self {
        let mut qa_history = VecDeque::new();
        
        // Welcome message
        qa_history.push_back(QAMessage {
            question: "Welcome to SutraWorks Q&A Demo!".to_string(),
            answer: "🤖 This is a Q&A demo showing live RWKV & Mamba inference in pure Rust!\n\n⚠️ Important: These are small demo models for testing the architecture.\n\n🧪 What you'll see:\n• Real AI model inference (not pre-written answers)\n• Actual performance metrics and timing\n• The raw output from neural networks\n• How well small models can respond\n\n🚀 Try asking questions and see what the AI models actually generate!".to_string(),
            timestamp: Self::current_time(),
            model_used: "System".to_string(),
            confidence: 1.0,
            inference_time: 0.0,
        });
        
        let suggested_questions = vec![
            "What is the difference between RWKV and Mamba architectures?".to_string(),
            "How does quantization improve AI model efficiency?".to_string(),
            "Explain the advantages of pure Rust for AI development".to_string(),
            "What are the benefits of linear complexity in neural networks?".to_string(),
            "How do state space models work?".to_string(),
            "What is the WKV mechanism in RWKV?".to_string(),
            "How does selective scanning work in Mamba?".to_string(),
            "What are the memory advantages of RNN-style models?".to_string(),
        ];
        
        Self {
            models,
            qa_history,
            input_question: String::new(),
            is_processing: false,
            selected_model: ModelChoice::Auto,
            response_style: ResponseStyle::Detailed,
            suggested_questions,
            context_window: Vec::new(),
            auto_suggest: true,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("🤔 Q&A Assistant");
        
        // Configuration panel
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.strong("🎛️ Settings");
                    
                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        if ui.add(egui::RadioButton::new(self.selected_model == ModelChoice::Auto, "Auto")).clicked() {
                            self.selected_model = ModelChoice::Auto;
                        }
                        if ui.add(egui::RadioButton::new(self.selected_model == ModelChoice::Rwkv, "RWKV")).clicked() {
                            self.selected_model = ModelChoice::Rwkv;
                        }
                        if ui.add(egui::RadioButton::new(self.selected_model == ModelChoice::Mamba, "Mamba")).clicked() {
                            self.selected_model = ModelChoice::Mamba;
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Style:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.response_style))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.response_style, ResponseStyle::Detailed, "Detailed");
                                ui.selectable_value(&mut self.response_style, ResponseStyle::Concise, "Concise");
                                ui.selectable_value(&mut self.response_style, ResponseStyle::Creative, "Creative");
                                ui.selectable_value(&mut self.response_style, ResponseStyle::Technical, "Technical");
                            });
                    });
                    
                    ui.checkbox(&mut self.auto_suggest, "Auto-suggest questions");
                });
            });
        });
        
        ui.separator();
        
        // Q&A history
        egui::ScrollArea::vertical()
            .max_height(350.0)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for qa in &self.qa_history {
                    self.show_qa_pair(ui, qa);
                }
            });
        
        ui.separator();
        
        // Question input
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Your question:");
                let input_response = ui.add_sized(
                    [ui.available_width() - 100.0, 60.0],
                    egui::TextEdit::multiline(&mut self.input_question)
                        .hint_text("Ask me anything about AI, programming, or general topics...")
                );
                
                ui.vertical(|ui| {
                    let ask_button = ui.add_enabled(
                        !self.input_question.trim().is_empty() && !self.is_processing,
                        egui::Button::new(if self.is_processing { "🤔 Thinking..." } else { "🤖 Ask" })
                    );
                    
                    if ui.button("🗑️ Clear").clicked() {
                        self.input_question.clear();
                    }
                    
                    if (input_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                        || ask_button.clicked() {
                        self.process_question(ctx);
                    }
                });
            });
            
            // Suggested questions
            if self.auto_suggest && !self.suggested_questions.is_empty() {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("💡 Suggested questions:");
                    for suggestion in self.suggested_questions.clone() {
                        if ui.small_button(&suggestion).clicked() {
                            self.input_question = suggestion;
                        }
                    }
                });
            }
        });
        
        ui.separator();
        
        // Performance stats
        self.show_performance_stats(ui);
    }
    
    fn show_qa_pair(&self, ui: &mut egui::Ui, qa: &QAMessage) {
        ui.group(|ui| {
            // Question header with terminal blue
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), "❓ Question:");
                ui.colored_label(egui::Color32::from_rgb(180, 180, 180), &qa.timestamp);
            });
            
            // Question text with high contrast
            ui.colored_label(egui::Color32::from_rgb(240, 240, 240), &qa.question);
            
            ui.add_space(8.0);
            
            // Answer header with terminal green  
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 150), "🤖 Answer:");
                ui.colored_label(egui::Color32::from_rgb(180, 180, 180), format!("{} • {:.1}ms", qa.model_used, qa.inference_time * 1000.0));
                
                // Confidence with color coding
                let confidence_color = if qa.confidence > 0.8 {
                    egui::Color32::from_rgb(100, 255, 100) // Green for high confidence
                } else if qa.confidence > 0.6 {
                    egui::Color32::from_rgb(255, 200, 100) // Yellow for medium
                } else {
                    egui::Color32::from_rgb(255, 100, 100) // Red for low confidence
                };
                ui.colored_label(confidence_color, format!("{:.0}% confidence", qa.confidence * 100.0));
            });
            
            // Answer box with terminal-style background
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(35, 45, 55))
                .inner_margin(12.0)
                .rounding(4.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 100, 120)))
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(230, 230, 230), &qa.answer);
                });
        });
        
        ui.add_space(12.0);
    }
    
    fn process_question(&mut self, ctx: &egui::Context) {
        if self.input_question.trim().is_empty() || self.is_processing {
            return;
        }
        
        let question = self.input_question.clone();
        self.input_question.clear();
        self.is_processing = true;
        
        // Choose model based on question type or user preference
        let use_rwkv = match self.selected_model {
            ModelChoice::Rwkv => true,
            ModelChoice::Mamba => false,
            ModelChoice::Auto => self.choose_best_model(&question),
        };
        
        // Generate contextual prompt
        let prompt = self.build_qa_prompt(&question);
        
        // Determine response length based on style
        let response_length = match self.response_style {
            ResponseStyle::Concise => 20,
            ResponseStyle::Detailed => 50,
            ResponseStyle::Creative => 40,
            ResponseStyle::Technical => 60,
        };
        
        // Generate answer
        match self.models.generate_text(&prompt, response_length, use_rwkv) {
            Ok((raw_response, inference_time)) => {
                let model_name = if use_rwkv { "RWKV" } else { "Mamba" };
                
                // Post-process response based on style
                let processed_answer = self.post_process_answer(&raw_response, &question);
                let confidence = self.estimate_confidence(&question, &processed_answer);
                
                self.qa_history.push_back(QAMessage {
                    question: question.clone(),
                    answer: processed_answer,
                    timestamp: Self::current_time(),
                    model_used: model_name.to_string(),
                    confidence,
                    inference_time,
                });
                
                // Update context window
                self.update_context(&question);
                
                // Generate new suggestions based on current topic
                self.update_suggestions(&question);
            }
            Err(e) => {
                self.qa_history.push_back(QAMessage {
                    question: question.clone(),
                    answer: format!("❌ Sorry, I encountered an error: {}\n\nPlease try rephrasing your question.", e),
                    timestamp: Self::current_time(),
                    model_used: "Error".to_string(),
                    confidence: 0.0,
                    inference_time: 0.0,
                });
            }
        }
        
        self.is_processing = false;
        ctx.request_repaint();
        
        // Keep only last 20 Q&A pairs
        while self.qa_history.len() > 20 {
            self.qa_history.pop_front();
        }
    }
    
    fn choose_best_model(&self, question: &str) -> bool {
        // Simple heuristics to choose model based on question type
        let question_lower = question.to_lowercase();
        
        // RWKV for sequential/reasoning tasks
        if question_lower.contains("step") || question_lower.contains("process") 
            || question_lower.contains("how") || question_lower.contains("explain") {
            return true; // Use RWKV
        }
        
        // Mamba for factual/classification tasks
        if question_lower.contains("what") || question_lower.contains("define") 
            || question_lower.contains("difference") || question_lower.contains("compare") {
            return false; // Use Mamba
        }
        
        // Default to RWKV for general questions
        true
    }
    
    fn build_qa_prompt(&self, question: &str) -> String {
        let style_prompt = match self.response_style {
            ResponseStyle::Detailed => "Provide a comprehensive and detailed answer.",
            ResponseStyle::Concise => "Give a brief and direct answer.",
            ResponseStyle::Creative => "Answer creatively with examples and analogies.",
            ResponseStyle::Technical => "Focus on technical accuracy and precision.",
        };
        
        let context = if !self.context_window.is_empty() {
            format!("Previous topics discussed: {}\n\n", self.context_window.join(", "))
        } else {
            String::new()
        };
        
        format!("{}{}\n\nQuestion: {}\nAnswer:", context, style_prompt, question)
    }
    
    fn post_process_answer(&self, raw_response: &str, _question: &str) -> String {
        // If the raw response is empty or just tokens, provide a meaningful answer based on the question
        if raw_response.trim().is_empty() || raw_response.chars().all(|c| c == '|' || c.is_whitespace()) {
            return "The AI model didn't generate any meaningful output. This is a demo with small models that are still learning.".to_string();
        }
        
        // Clean up and format the response
        let mut answer = raw_response.trim().to_string();
        
        // Add style-specific formatting
        match self.response_style {
            ResponseStyle::Technical => {
                if !answer.contains("•") && !answer.contains("-") {
                    answer = format!("Technical Answer: {}", answer);
                }
            }
            ResponseStyle::Creative => {
                if !answer.starts_with('�') {
                    answer = format!("✨ {}", answer);
                }
            }
            _ => {}
        }
        
        answer
    }

    
    fn estimate_confidence(&self, question: &str, answer: &str) -> f32 {
        // Simple confidence estimation based on response quality
        let mut confidence: f32 = 0.7; // Base confidence
        
        // Higher confidence for longer, more detailed answers
        if answer.len() > 100 { confidence += 0.1; }
        if answer.len() > 200 { confidence += 0.1; }
        
        // Lower confidence for very short answers
        if answer.len() < 50 { confidence -= 0.2; }
        
        // Adjust based on question complexity
        if question.len() > 100 { confidence -= 0.1; } // Complex questions are harder
        if question.contains('?') { confidence += 0.05; } // Proper questions
        
        confidence.clamp(0.1_f32, 0.95_f32)
    }
    
    fn update_context(&mut self, question: &str) {
        // Extract key topics from the question
        let keywords = self.extract_keywords(question);
        for keyword in keywords {
            self.context_window.push(keyword);
        }
        
        // Keep only recent context
        while self.context_window.len() > 5 {
            self.context_window.remove(0);
        }
    }
    
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction
        let important_words = vec![
            "AI", "RWKV", "Mamba", "neural", "network", "model", "quantization", 
            "rust", "programming", "algorithm", "machine learning", "deep learning"
        ];
        
        let mut keywords = Vec::new();
        let text_lower = text.to_lowercase();
        
        for word in important_words {
            if text_lower.contains(&word.to_lowercase()) {
                keywords.push(word.to_string());
            }
        }
        
        keywords
    }
    
    fn update_suggestions(&mut self, question: &str) {
        // Generate follow-up questions based on current question
        let new_suggestions = if question.to_lowercase().contains("rwkv") {
            vec![
                "How does RWKV compare to traditional transformers?".to_string(),
                "What is the WKV mechanism in RWKV?".to_string(),
                "Why is RWKV memory efficient?".to_string(),
            ]
        } else if question.to_lowercase().contains("mamba") {
            vec![
                "How does selective scanning work in Mamba?".to_string(),
                "What are state space models?".to_string(),
                "Why is Mamba faster than transformers?".to_string(),
            ]
        } else if question.to_lowercase().contains("quantization") {
            vec![
                "What is AWQ quantization?".to_string(),
                "How much memory does quantization save?".to_string(),
                "Does quantization affect model accuracy?".to_string(),
            ]
        } else {
            vec![
                "Tell me about pure Rust AI frameworks".to_string(),
                "How do I get started with SutraWorks?".to_string(),
                "What are the benefits of edge AI?".to_string(),
            ]
        };
        
        self.suggested_questions = new_suggestions;
    }
    
    fn show_performance_stats(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "📊 Q&A Performance Stats");
                ui.separator();
                
                let total_questions = self.qa_history.len().saturating_sub(1); // Exclude welcome message
                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), format!("Questions: {}", total_questions));
                
                if total_questions > 0 {
                    let avg_time: f64 = self.qa_history.iter()
                        .skip(1) // Skip welcome message
                        .map(|qa| qa.inference_time)
                        .sum::<f64>() / total_questions as f64;
                    
                    let avg_confidence: f32 = self.qa_history.iter()
                        .skip(1)
                        .map(|qa| qa.confidence)
                        .sum::<f32>() / total_questions as f32;
                    
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 150), format!("Avg Response Time: {:.1}ms", avg_time * 1000.0));
                    ui.separator();
                    
                    // Color-code confidence
                    let confidence_color = if avg_confidence > 0.8 {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else if avg_confidence > 0.6 {
                        egui::Color32::from_rgb(255, 200, 100)
                    } else {
                        egui::Color32::from_rgb(255, 100, 100)
                    };
                    ui.colored_label(confidence_color, format!("Avg Confidence: {:.0}%", avg_confidence * 100.0));
                }
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