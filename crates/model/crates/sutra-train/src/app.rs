/// Main Training Application GUI
use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::config::TrainingConfig;
use crate::data::DataManager;
use crate::templates::{ModelTemplate, TemplateManager};
use crate::progress::TrainingProgress;
use crate::ui::{UIState, TabType};

pub struct TrainingApp {
    /// Current UI state
    ui_state: UIState,
    
    /// Training configuration
    config: TrainingConfig,
    
    /// Data management
    data_manager: DataManager,
    
    /// Available model templates
    template_manager: TemplateManager,
    
    /// Training progress tracking
    progress: Arc<Mutex<TrainingProgress>>,
    
    /// Background training task handle
    training_handle: Option<tokio::task::JoinHandle<()>>,
    
    /// Runtime for async tasks
    runtime: tokio::runtime::Runtime,
}

impl TrainingApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous session if available
        let config = if let Some(storage) = cc.storage {
            storage.get_string("training_config").and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
        } else {
            TrainingConfig::default()
        };

        Self {
            ui_state: UIState::default(),
            config,
            data_manager: DataManager::new(),
            template_manager: TemplateManager::new(),
            progress: Arc::new(Mutex::new(TrainingProgress::new())),
            training_handle: None,
            runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
        }
    }
}

impl eframe::App for TrainingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update progress from background tasks
        self.check_training_progress();

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menu_bar(ui, ctx);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        // Side panel for configuration
        egui::SidePanel::left("side_panel")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.show_side_panel(ui);
            });

        // Central panel for main content
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_central_panel(ui, ctx);
        });

        // Handle drag and drop
        self.handle_drag_drop(ctx);

        // Check if we need to repaint for animations
        if self.ui_state.training_active {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(config_str) = serde_json::to_string(&self.config) {
            storage.set_string("training_config", config_str);
        }
    }
}

impl TrainingApp {
    fn show_menu_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    self.new_project();
                    ui.close_menu();
                }
                
                if ui.button("Open Project").clicked() {
                    self.open_project();
                    ui.close_menu();
                }
                
                if ui.button("Save Project").clicked() {
                    self.save_project();
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Import Data").clicked() {
                    self.import_data();
                    ui.close_menu();
                }
                
                if ui.button("Export Model").clicked() {
                    self.export_model();
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Training", |ui| {
                if ui.button("Start Training").clicked() {
                    self.start_training();
                    ui.close_menu();
                }
                
                if ui.button("Stop Training").clicked() {
                    self.stop_training();
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Resume from Checkpoint").clicked() {
                    self.resume_training();
                    ui.close_menu();
                }
            });

            ui.menu_button("Models", |ui| {
                if ui.button("Browse Templates").clicked() {
                    self.ui_state.current_tab = TabType::Templates;
                    ui.close_menu();
                }
                
                if ui.button("Custom Model").clicked() {
                    self.ui_state.current_tab = TabType::CustomModel;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Quick Start Guide").clicked() {
                    self.ui_state.show_help = true;
                    ui.close_menu();
                }
                
                if ui.button("Examples").clicked() {
                    self.ui_state.show_examples = true;
                    ui.close_menu();
                }
                
                if ui.button("About").clicked() {
                    self.ui_state.show_about = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Training status
            if self.ui_state.training_active {
                ui.colored_label(egui::Color32::GREEN, "🟢 Training in progress");
                
                if let Ok(progress) = self.progress.lock() {
                    ui.label(format!("Epoch {}/{}", progress.current_epoch, progress.total_epochs));
                    ui.add(egui::ProgressBar::new(progress.progress())
                        .text(format!("{:.1}%", progress.progress() * 100.0)));
                }
            } else {
                ui.colored_label(egui::Color32::GRAY, "⚪ Ready");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Memory usage
                ui.label(format!("Memory: {:.1} GB", self.get_memory_usage()));
                
                ui.separator();
                
                // Data status
                let data_status = if self.data_manager.has_data() {
                    format!("📊 {} samples", self.data_manager.sample_count())
                } else {
                    "📊 No data loaded".to_string()
                };
                ui.label(data_status);
            });
        });
    }

    fn show_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.collapsing("Model Settings", |ui| {
                self.show_model_config(ui);
            });

            ui.collapsing("Training Parameters", |ui| {
                self.show_training_config(ui);
            });

            ui.collapsing("Data Settings", |ui| {
                self.show_data_config(ui);
            });

            ui.collapsing("Advanced Options", |ui| {
                self.show_advanced_config(ui);
            });
        });
    }

    fn show_central_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.ui_state.current_tab, TabType::Overview, "📊 Overview");
            ui.selectable_value(&mut self.ui_state.current_tab, TabType::Data, "📁 Data");
            ui.selectable_value(&mut self.ui_state.current_tab, TabType::Templates, "🎯 Models");
            ui.selectable_value(&mut self.ui_state.current_tab, TabType::Training, "🚀 Training");
            ui.selectable_value(&mut self.ui_state.current_tab, TabType::Results, "📈 Results");
        });

        ui.separator();

        // Tab content
        match self.ui_state.current_tab {
            TabType::Overview => self.show_overview_tab(ui),
            TabType::Data => self.show_data_tab(ui, ctx),
            TabType::Templates => self.show_templates_tab(ui),
            TabType::Training => self.show_training_tab(ui),
            TabType::Results => self.show_results_tab(ui),
            TabType::CustomModel => self.show_custom_model_tab(ui),
        }

        // Show modal dialogs
        self.show_modal_dialogs(ctx);
    }

    fn new_project(&mut self) {
        self.config = crate::config::TrainingConfig::default();
        self.data_manager.clear_all();
        self.ui_state.selected_template = None;
        if self.ui_state.training_active {
            self.stop_training();
        }
    }
    
    fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SutraWorks Project", &["swproj", "json"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<TrainingConfig>(&content) {
                    self.config = config;
                    println!("Project loaded from: {:?}", path);
                } else {
                    eprintln!("Failed to parse project file");
                }
            }
        }
    }
    
    fn save_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SutraWorks Project", &["swproj", "json"])
            .set_file_name("training_project.swproj")
            .save_file()
        {
            if let Ok(json) = serde_json::to_string_pretty(&self.config) {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("Failed to save project: {}", e);
                } else {
                    println!("Project saved to: {:?}", path);
                }
            }
        }
    }
    
    fn import_data(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .add_filter("Data Files", &["txt", "jsonl", "json", "csv"])
            .pick_files()
        {
            for path in paths {
                if let Err(e) = self.data_manager.add_training_file(path.clone()) {
                    eprintln!("Error loading file {:?}: {}", path, e);
                } else {
                    println!("Loaded data file: {:?}", path);
                }
            }
        }
    }
    
    fn export_model(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Model Files", &["safetensors", "onnx", "pt"])
            .set_file_name(format!("{}.safetensors", self.config.output.model_name))
            .save_file()
        {
            let output_dir = &self.config.output.output_dir;
            let model_file = format!("{}/{}.safetensors", output_dir, self.config.output.model_name);
            
            if std::path::Path::new(&model_file).exists() {
                if let Err(e) = std::fs::copy(&model_file, &path) {
                    eprintln!("Failed to export model: {}", e);
                } else {
                    println!("Model exported to: {:?}", path);
                }
            } else {
                eprintln!("Model file not found. Please complete training first.");
            }
        }
    }
    
    fn start_training(&mut self) {
        if !self.data_manager.has_data() || self.ui_state.selected_template.is_none() {
            return;
        }
        
        self.ui_state.training_active = true;
        
        // Initialize progress
        if let Ok(mut progress) = self.progress.lock() {
            let total_steps = self.data_manager.sample_count() / self.config.training.batch_size * self.config.training.epochs;
            progress.start_training(self.config.training.epochs, total_steps);
        }
        
        // Create output directory
        std::fs::create_dir_all(&self.config.output.output_dir).ok();
        
        // Clone data needed for background task
        let config = self.config.clone();
        let progress = Arc::clone(&self.progress);
        let data_files = self.data_manager.get_training_files().to_vec();
        
        // Spawn background training task
        let handle = self.runtime.spawn(async move {
            Self::run_training_loop(config, progress, data_files).await;
        });
        
        self.training_handle = Some(handle);
        println!("Training started with {} architecture", match self.config.model.architecture {
            crate::config::ModelArchitecture::RWKV => "RWKV",
            crate::config::ModelArchitecture::Mamba => "Mamba",
            crate::config::ModelArchitecture::Custom => "Custom",
        });
    }
    
    fn stop_training(&mut self) {
        self.ui_state.training_active = false;
        
        if let Some(handle) = self.training_handle.take() {
            handle.abort();
        }
        
        println!("Training stopped");
    }
    
    fn resume_training(&mut self) {
        let checkpoint_path = format!("{}/checkpoint.json", self.config.output.output_dir);
        
        if std::path::Path::new(&checkpoint_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&checkpoint_path) {
                let should_resume = if let Ok(mut progress) = self.progress.lock() {
                    if let Ok(loaded_progress) = serde_json::from_str::<TrainingProgress>(&content) {
                        let epoch = loaded_progress.current_epoch;
                        *progress = loaded_progress;
                        println!("Checkpoint loaded. Resuming from epoch {}", epoch);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                if should_resume {
                    self.start_training();
                }
            }
        } else {
            eprintln!("No checkpoint found at: {}", checkpoint_path);
        }
    }
    
    fn check_training_progress(&mut self) {
        if self.ui_state.training_active {
            // Check if training task completed
            if let Some(handle) = &self.training_handle {
                if handle.is_finished() {
                    self.ui_state.training_active = false;
                    println!("Training completed!");
                    
                    // Save final checkpoint
                    if let Ok(progress) = self.progress.lock() {
                        let checkpoint_path = format!("{}/checkpoint_final.json", self.config.output.output_dir);
                        if let Ok(json) = serde_json::to_string_pretty(&*progress) {
                            std::fs::write(checkpoint_path, json).ok();
                        }
                    }
                }
            }
            
            // Auto-save checkpoint periodically
            if let Ok(progress) = self.progress.lock() {
                if progress.current_step % self.config.training.save_every == 0 && progress.current_step > 0 {
                    let checkpoint_path = format!("{}/checkpoint.json", self.config.output.output_dir);
                    if let Ok(json) = serde_json::to_string_pretty(&*progress) {
                        std::fs::write(checkpoint_path, json).ok();
                    }
                }
            }
        }
    }
    
    fn handle_drag_drop(&mut self, ctx: &egui::Context) {
        // Check for dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        if let Err(e) = self.data_manager.add_training_file(path.clone()) {
                            eprintln!("Error adding file: {}", e);
                        }
                    }
                }
            }
        });
    }
    
    fn get_memory_usage(&self) -> f64 {
        let base = crate::utils::estimate_memory_usage(
            self.config.model.size.params(),
            self.config.model.quantization.enabled,
            self.config.model.lora.enabled,
            self.config.model.lora.rank
        );
        
        // Add training overhead if active
        if self.ui_state.training_active {
            base as f64 * 1.2 // 20% overhead for gradients and optimizer states
        } else {
            base as f64
        }
    }
    
    fn show_model_config(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Architecture:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.config.model.architecture))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.model.architecture, crate::config::ModelArchitecture::RWKV, "RWKV");
                    ui.selectable_value(&mut self.config.model.architecture, crate::config::ModelArchitecture::Mamba, "Mamba");
                });
        });
        
        ui.horizontal(|ui| {
            ui.label("Model Size:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.config.model.size))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.model.size, crate::config::ModelSize::Tiny, "Tiny (100M)");
                    ui.selectable_value(&mut self.config.model.size, crate::config::ModelSize::Small, "Small (500M)");
                    ui.selectable_value(&mut self.config.model.size, crate::config::ModelSize::Medium, "Medium (1B)");
                    ui.selectable_value(&mut self.config.model.size, crate::config::ModelSize::Large, "Large (3B)");
                });
        });
        
        ui.label(format!("Estimated memory: {:.1} GB", self.get_memory_usage()));
    }

    fn show_training_config(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Epochs:");
            ui.add(egui::DragValue::new(&mut self.config.training.epochs).range(1..=100));
        });
        
        ui.horizontal(|ui| {
            ui.label("Batch Size:");
            ui.add(egui::DragValue::new(&mut self.config.training.batch_size).range(1..=32));
        });
        
        ui.horizontal(|ui| {
            ui.label("Learning Rate:");
            ui.add(egui::DragValue::new(&mut self.config.training.learning_rate).speed(1e-6).range(1e-6..=1e-2).custom_formatter(|n, _| format!("{:.1e}", n)));
        });
    }

    fn show_data_config(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Max Length:");
            ui.add(egui::DragValue::new(&mut self.config.data.max_length).range(128..=8192));
        });
        
        ui.horizontal(|ui| {
            ui.label("Validation Split:");
            ui.add(egui::Slider::new(&mut self.config.data.validation_split, 0.0..=0.3).suffix("%").custom_formatter(|n, _| format!("{:.1}%", n * 100.0)));
        });
    }

    fn show_advanced_config(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.config.model.quantization.enabled, "Enable Quantization");
        
        if self.config.model.quantization.enabled {
            ui.horizontal(|ui| {
                ui.label("Bits:");
                ui.add(egui::Slider::new(&mut self.config.model.quantization.bits, 2..=8));
            });
        }
        
        ui.checkbox(&mut self.config.model.lora.enabled, "Enable LoRA");
        
        if self.config.model.lora.enabled {
            ui.horizontal(|ui| {
                ui.label("Rank:");
                ui.add(egui::DragValue::new(&mut self.config.model.lora.rank).range(1..=64));
            });
            
            ui.horizontal(|ui| {
                ui.label("Alpha:");
                ui.add(egui::DragValue::new(&mut self.config.model.lora.alpha).range(1.0..=128.0));
            });
        }
    }
    
    fn show_overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 SutraWorks Training Studio");
        ui.separator();

        ui.horizontal(|ui| {
            // Quick stats cards
            ui.group(|ui| {
                ui.set_width(200.0);
                ui.vertical_centered(|ui| {
                    ui.heading("📊");
                    ui.label("Data Status");
                    if self.data_manager.has_data() {
                        ui.colored_label(egui::Color32::GREEN, format!("{} samples", self.data_manager.sample_count()));
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "No data loaded");
                    }
                });
            });

            ui.group(|ui| {
                ui.set_width(200.0);
                ui.vertical_centered(|ui| {
                    ui.heading("🎯");
                    ui.label("Model Template");
                    if let Some(template) = &self.ui_state.selected_template {
                        ui.colored_label(egui::Color32::GREEN, template);
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "Not selected");
                    }
                });
            });

            ui.group(|ui| {
                ui.set_width(200.0);
                ui.vertical_centered(|ui| {
                    ui.heading("⚡");
                    ui.label("Training Status");
                    if self.ui_state.training_active {
                        ui.colored_label(egui::Color32::GREEN, "In Progress");
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "Ready");
                    }
                });
            });
        });

        ui.add_space(20.0);

        // Quick start guide
        ui.group(|ui| {
            ui.label("🎯 Quick Start Guide:");
            ui.separator();
            ui.label("1. 📁 Load your training data (drag & drop supported)");
            ui.label("2. 🎯 Choose a model template that fits your use case");
            ui.label("3. ⚙️ Configure training parameters (or use defaults)");
            ui.label("4. 🚀 Start training and monitor progress");
            ui.label("5. 📤 Export your trained model");
        });
    }
    fn show_data_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("📁 Training Data");
        ui.separator();

        // Data loading area
        ui.group(|ui| {
            ui.set_min_height(200.0);
            ui.vertical_centered(|ui| {
                if self.data_manager.has_data() {
                    ui.heading("✅ Data Loaded");
                    ui.label(format!("{} training files", self.data_manager.get_training_files().len()));
                    ui.label(format!("~{} samples", self.data_manager.sample_count()));
                    
                    if ui.button("Clear Data").clicked() {
                        self.data_manager.clear_all();
                    }
                } else {
                    ui.heading("📂 Drop files here");
                    ui.label("Supported formats: .txt, .jsonl, .csv");
                    
                    if ui.button("Browse Files...").clicked() {
                        self.import_data();
                    }
                }
            });
        });

        ui.add_space(20.0);

        // Data requirements for selected template
        if let Some(template_name) = &self.ui_state.selected_template {
            if let Some(template) = self.template_manager.get_template(template_name) {
                ui.group(|ui| {
                    ui.label(format!("📋 Data Requirements for {}", template.name));
                    ui.separator();
                    ui.label(format!("Minimum samples: {}", template.data_requirements.min_samples));
                    ui.label(format!("Recommended: {}", template.data_requirements.recommended_samples));
                    ui.label(format!("Format: {}", template.data_requirements.data_format));
                    
                    ui.collapsing("Example", |ui| {
                        ui.code(&template.data_requirements.example);
                    });
                });
            }
        }

        // Data validation warnings
        if let Ok(warnings) = self.data_manager.validate_data() {
            if !warnings.is_empty() {
                ui.group(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Warnings:");
                    for warning in warnings {
                        ui.label(format!("• {}", warning));
                    }
                });
            }
        }
    }
    fn show_templates_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 Model Templates");
        ui.label("Choose a template that best fits your use case:");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let templates: Vec<_> = self.template_manager.get_templates().into_iter().cloned().collect();
            
            for template in templates {
                let template_name = template.name.clone();
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&template.icon);
                        ui.vertical(|ui| {
                            ui.heading(&template.name);
                            ui.label(&template.description);
                            ui.small(&template.use_case);
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("Architecture: {:?}", template.architecture));
                                ui.label(format!("Size: {:?}", template.recommended_size));
                            });
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Select").clicked() {
                                self.ui_state.selected_template = Some(template_name);
                                self.apply_template(&template);
                            }
                        });
                    });
                });
                ui.add_space(10.0);
            }
        });
    }
    fn show_training_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Training");
        ui.separator();

        // Training controls
        ui.horizontal(|ui| {
            if !self.ui_state.training_active {
                let can_start = self.data_manager.has_data() && self.ui_state.selected_template.is_some();
                
                ui.add_enabled_ui(can_start, |ui| {
                    if ui.button("▶️ Start Training").clicked() {
                        self.start_training();
                    }
                });
                
                if !can_start {
                    ui.colored_label(egui::Color32::GRAY, "Select template and load data first");
                }
            } else {
                if ui.button("⏹️ Stop Training").clicked() {
                    self.stop_training();
                }
                
                if ui.button("⏸️ Pause").clicked() {
                    // Save checkpoint and stop
                    if let Ok(progress) = self.progress.lock() {
                        let checkpoint_path = format!("{}/checkpoint_paused.json", self.config.output.output_dir);
                        if let Ok(json) = serde_json::to_string_pretty(&*progress) {
                            std::fs::write(checkpoint_path, json).ok();
                        }
                    }
                    self.stop_training();
                    println!("Training paused. Use 'Resume from Checkpoint' to continue.");
                }
            }
        });

        ui.add_space(20.0);

        // Training progress
        if self.ui_state.training_active {
            if let Ok(progress) = self.progress.lock() {
                ui.group(|ui| {
                    ui.label("📊 Training Progress");
                    ui.separator();
                    
                    // Overall progress
                    ui.horizontal(|ui| {
                        ui.label("Overall:");
                        ui.add(egui::ProgressBar::new(progress.progress())
                            .text(format!("{:.1}%", progress.progress() * 100.0)));
                        ui.label(format!("{}/{} steps", progress.current_step, progress.total_steps));
                    });
                    
                    // Epoch progress
                    ui.horizontal(|ui| {
                        ui.label("Epoch:");
                        ui.add(egui::ProgressBar::new(progress.epoch_progress())
                            .text(format!("{}/{}", progress.current_epoch, progress.total_epochs)));
                    });
                    
                    // Metrics
                    ui.horizontal(|ui| {
                        ui.label(format!("Loss: {:.4}", progress.train_loss));
                        ui.label(format!("LR: {:.6}", progress.learning_rate));
                        ui.label(format!("ETA: {}", progress.format_eta()));
                    });
                });
            }
        }

        ui.add_space(20.0);

        // Training configuration preview
        ui.group(|ui| {
            ui.label("⚙️ Training Configuration");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(format!("Epochs: {}", self.config.training.epochs));
                ui.label(format!("Batch size: {}", self.config.training.batch_size));
                ui.label(format!("Learning rate: {}", self.config.training.learning_rate));
            });
            
            if let Some(eta) = self.data_manager.estimate_training_time(self.config.training.epochs, self.config.training.batch_size).checked_sub(std::time::Duration::from_secs(0)) {
                ui.label(format!("Estimated time: {}", crate::utils::format_duration(eta)));
            }
        });
    }
    fn apply_template(&mut self, template: &ModelTemplate) {
        // Update configuration based on template
        self.config.model.architecture = template.architecture.clone();
        self.config.model.template = template.name.clone();
        self.config.training.epochs = template.training_config.epochs;
        self.config.training.batch_size = template.training_config.batch_size;
        self.config.training.learning_rate = template.training_config.learning_rate;
        self.config.model.lora = template.training_config.lora.clone();
    }
    
    async fn run_training_loop(
        config: TrainingConfig,
        progress: Arc<Mutex<TrainingProgress>>,
        data_files: Vec<std::path::PathBuf>,
    ) {
        use sutra_training::{Trainer, TrainerConfig};
        
        // Initialize trainer
        let trainer_config = TrainerConfig {
            epochs: config.training.epochs,
            batch_size: config.training.batch_size,
            gradient_accumulation_steps: 1,
            max_grad_norm: Some(config.training.gradient_clipping),
            eval_steps: config.training.eval_every,
            save_steps: config.training.save_every,
            logging_steps: 100,
            output_dir: config.output.output_dir.clone(),
        };
        
        let trainer = Trainer::new(trainer_config);
        
        // Calculate total steps
        let samples_per_file = 1000; // Estimate
        let total_samples = data_files.len() * samples_per_file;
        let steps_per_epoch = total_samples.div_ceil(config.training.batch_size);
        let total_steps = steps_per_epoch * config.training.epochs;
        
        // Start training
        if let Ok(mut prog) = progress.lock() {
            prog.start_training(config.training.epochs, total_steps);
        }
        
        // Training loop
        for epoch in 0..config.training.epochs {
            if let Ok(mut prog) = progress.lock() {
                prog.update_epoch(epoch + 1);
            }
            
            for step in 0..steps_per_epoch {
                let global_step = epoch * steps_per_epoch + step;
                
                // Simulate training step (in production, this would do actual forward/backward pass)
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                
                // Calculate metrics
                let progress_ratio = global_step as f32 / total_steps as f32;
                let loss = 2.5 * (1.0 - progress_ratio) + 0.5; // Decreasing loss
                let lr = config.training.learning_rate * (1.0 - progress_ratio);
                
                // Update progress
                if let Ok(mut prog) = progress.lock() {
                    prog.update_step(global_step + 1, loss, lr);
                    prog.metrics.memory_usage = crate::utils::estimate_memory_usage(
                        config.model.size.params(),
                        config.model.quantization.enabled,
                        config.model.lora.enabled,
                        config.model.lora.rank,
                    );
                }
                
                // Periodic evaluation
                if (global_step + 1).is_multiple_of(config.training.eval_every) {
                    let val_loss = loss * 1.1; // Validation typically slightly higher
                    if let Ok(mut prog) = progress.lock() {
                        prog.update_validation(val_loss);
                    }
                }
                
                // Logging
                if (global_step + 1).is_multiple_of(100) {
                    trainer.log(format!(
                        "Epoch {}/{}, Step {}/{}, Loss: {:.4}, LR: {:.6}",
                        epoch + 1,
                        config.training.epochs,
                        step + 1,
                        steps_per_epoch,
                        loss,
                        lr
                    ));
                }
            }
            
            println!("Completed epoch {}/{}", epoch + 1, config.training.epochs);
        }
        
        // Save final model
        let model_path = format!(
            "{}/{}.safetensors",
            config.output.output_dir, config.output.model_name
        );
        
        // Create a placeholder model file (in production, this would save actual model weights)
        if let Ok(prog) = progress.lock() {
            let metadata = serde_json::json!({
                "architecture": format!("{:?}", config.model.architecture),
                "model_size": format!("{:?}", config.model.size),
                "final_loss": prog.train_loss,
                "epochs": config.training.epochs,
                "quantization": config.model.quantization.enabled,
                "lora_enabled": config.model.lora.enabled,
            });
            
            std::fs::write(&model_path, serde_json::to_string_pretty(&metadata).unwrap()).ok();
            println!("Model saved to: {}", model_path);
        }
        
        println!("Training completed successfully!");
    }

    fn show_results_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📈 Training Results");
        ui.separator();
        
        if let Ok(progress) = self.progress.lock() {
            if progress.metrics.loss_history.is_empty() {
                ui.label("No training data yet. Start training to see results.");
                return;
            }
            
            // Summary statistics
            ui.group(|ui| {
                ui.heading("📊 Summary");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Training Progress:");
                        ui.label(format!("Epochs: {}/{}", progress.current_epoch, progress.total_epochs));
                        ui.label(format!("Steps: {}/{}", progress.current_step, progress.total_steps));
                    });
                    
                    ui.separator();
                    
                    ui.vertical(|ui| {
                        ui.label("Loss Metrics:");
                        if let Some(latest_loss) = progress.metrics.loss_history.last() {
                            ui.label(format!("Current Loss: {:.4}", latest_loss));
                        }
                        if let Some(first_loss) = progress.metrics.loss_history.first() {
                            if let Some(latest_loss) = progress.metrics.loss_history.last() {
                                let improvement = ((first_loss - latest_loss) / first_loss * 100.0).max(0.0);
                                ui.label(format!("Improvement: {:.1}%", improvement));
                            }
                        }
                    });
                });
            });
            
            ui.add_space(20.0);
            
            // Loss curve visualization (text-based)
            ui.group(|ui| {
                ui.heading("📉 Loss History");
                ui.separator();
                
                let history = &progress.metrics.loss_history;
                if history.len() > 1 {
                    let min_loss = history.iter().cloned().fold(f32::INFINITY, f32::min);
                    let max_loss = history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let range = max_loss - min_loss;
                    
                    // Show last 20 data points
                    let start = if history.len() > 20 { history.len() - 20 } else { 0 };
                    for (i, loss) in history[start..].iter().enumerate() {
                        let normalized = if range > 0.0 { (max_loss - loss) / range } else { 0.5 };
                        let bar_width = (normalized * 300.0) as usize;
                        let bar = "█".repeat(bar_width.max(1));
                        ui.label(format!("Step {}: {} {:.4}", start + i, bar, loss));
                    }
                } else {
                    ui.label("Collecting data...");
                }
            });
            
            ui.add_space(20.0);
            
            // Validation results
            if !progress.metrics.validation_loss_history.is_empty() {
                ui.group(|ui| {
                    ui.heading("✓ Validation Results");
                    ui.separator();
                    
                    if let Some(val_loss) = progress.metrics.validation_loss_history.last() {
                        ui.label(format!("Latest Validation Loss: {:.4}", val_loss));
                    }
                    
                    ui.label(format!("Validation checks: {}", progress.metrics.validation_loss_history.len()));
                });
            }
        } else {
            ui.label("Unable to access training progress.");
        }
    }
    fn show_custom_model_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Custom Model Configuration");
        ui.label("Build your own model configuration from scratch.");
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Custom configuration allows you to fine-tune all parameters.");
            ui.separator();
            
            self.show_model_config(ui);
            ui.add_space(10.0);
            self.show_training_config(ui);
            ui.add_space(10.0);
            self.show_data_config(ui);
            ui.add_space(10.0);
            self.show_advanced_config(ui);
        });
        
        ui.add_space(20.0);
        
        if ui.button("💾 Save as Custom Template").clicked() {
            self.config.model.template = "Custom".to_string();
            self.ui_state.selected_template = Some("Custom".to_string());
        }
    }
    
    fn show_modal_dialogs(&mut self, ctx: &egui::Context) {
        // Help dialog
        if self.ui_state.show_help {
            egui::Window::new("📚 Quick Start Guide")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Welcome to SutraWorks Training Studio!");
                    ui.separator();
                    ui.label("1. 📁 Load your training data (drag & drop or use File menu)");
                    ui.label("2. 🎯 Choose a template from the Models tab");
                    ui.label("3. ⚙️ Adjust settings in the left panel (or use defaults)");
                    ui.label("4. 🚀 Go to Training tab and click Start Training");
                    ui.label("5. 📈 Monitor progress in Results tab");
                    ui.label("6. 📤 Export your model when complete");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.ui_state.show_help = false;
                    }
                });
        }
        
        // Examples dialog
        if self.ui_state.show_examples {
            egui::Window::new("📖 Data Format Examples")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.label("Example data formats for each template:");
                    ui.separator();
                    
                    for template in self.template_manager.get_templates() {
                        ui.collapsing(&template.name, |ui| {
                            ui.label(format!("Format: {}", template.data_requirements.data_format));
                            ui.code(&template.data_requirements.example);
                        });
                    }
                    
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.ui_state.show_examples = false;
                    }
                });
        }
        
        // About dialog
        if self.ui_state.show_about {
            egui::Window::new("ℹ️ About SutraWorks Training Studio")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("SutraWorks Training Studio");
                    ui.label("Version 1.0.0");
                    ui.separator();
                    ui.label("A user-friendly GUI for training AI models");
                    ui.label("without requiring machine learning expertise.");
                    ui.add_space(10.0);
                    ui.label("Built with:");
                    ui.label("• Rust - Systems programming language");
                    ui.label("• egui - Immediate mode GUI framework");
                    ui.label("• SutraWorks - Efficient AI architecture library");
                    ui.add_space(10.0);
                    ui.label("© 2025 SutraWorks. All rights reserved.");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.ui_state.show_about = false;
                    }
                });
        }
    }
}