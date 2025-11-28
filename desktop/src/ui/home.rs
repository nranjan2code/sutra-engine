//! Home Dashboard - Default Landing View
//!
//! A welcoming dashboard that provides an overview of the knowledge base,
//! quick actions, and recent activity.

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Sense, Stroke, Vec2};
use chrono::{DateTime, Local};
use crate::theme::{
    PRIMARY, PRIMARY_DIM, PRIMARY_LIGHT, SECONDARY, ACCENT, SUCCESS, WARNING, ERROR,
    TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_WIDGET, BG_ELEVATED, BG_DARK,
    elevated_card,
};

/// Home dashboard panel
pub struct HomeDashboard {
    /// Overview stats
    pub stats: DashboardStats,
    /// Recent activity log
    pub recent_activity: Vec<ActivityItem>,
    /// Quick tips shown on dashboard
    pub tips: Vec<QuickTip>,
    /// Currently shown tip index
    pub current_tip: usize,
    /// Last update time
    pub last_update: DateTime<Local>,
}

/// Dashboard statistics
#[derive(Default, Clone)]
pub struct DashboardStats {
    pub total_concepts: usize,
    pub total_connections: usize,
    pub concepts_today: usize,
    pub queries_today: usize,
    pub storage_mb: f32,
    pub health_score: f32, // 0.0 - 1.0
}

/// Recent activity item
#[derive(Clone)]
pub struct ActivityItem {
    pub timestamp: DateTime<Local>,
    pub activity_type: DashboardActivityType,
    pub description: String,
    pub icon: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DashboardActivityType {
    Learn,
    Query,
    Import,
    Export,
    Delete,
}

/// Quick tip for users
#[derive(Clone)]
pub struct QuickTip {
    pub title: &'static str,
    pub description: &'static str,
    pub command: Option<&'static str>,
    pub icon: &'static str,
}

impl Default for HomeDashboard {
    fn default() -> Self {
        Self {
            stats: DashboardStats::default(),
            recent_activity: Vec::new(),
            tips: create_quick_tips(),
            current_tip: 0,
            last_update: Local::now(),
        }
    }
}

impl HomeDashboard {
    /// Update dashboard stats from storage
    pub fn update_stats(&mut self, concepts: usize, connections: usize, concepts_today: usize, queries_today: usize) {
        self.stats.total_concepts = concepts;
        self.stats.total_connections = connections;
        self.stats.concepts_today = concepts_today;
        self.stats.queries_today = queries_today;
        self.stats.storage_mb = (concepts as f32 * 0.5) / 1024.0; // Rough estimate
        self.stats.health_score = calculate_health_score(&self.stats);
        self.last_update = Local::now();
    }
    
    /// Add activity item
    pub fn add_activity(&mut self, activity_type: DashboardActivityType, description: String) {
        let icon = match activity_type {
            DashboardActivityType::Learn => "📝",
            DashboardActivityType::Query => "🔍",
            DashboardActivityType::Import => "📥",
            DashboardActivityType::Export => "📤",
            DashboardActivityType::Delete => "🗑️",
        };
        
        self.recent_activity.insert(0, ActivityItem {
            timestamp: Local::now(),
            activity_type,
            description,
            icon,
        });
        
        // Keep only last 50 activities
        if self.recent_activity.len() > 50 {
            self.recent_activity.truncate(50);
        }
    }
    
    /// Cycle to next tip
    pub fn next_tip(&mut self) {
        self.current_tip = (self.current_tip + 1) % self.tips.len();
    }
    
    /// Render the dashboard
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<HomeAction> {
        let mut action = None;
        
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.set_width(ui.available_width().min(900.0));
                
                // Welcome header
                self.render_header(ui);
                ui.add_space(24.0);
                
                // Stats overview cards
                self.render_stats_cards(ui);
                ui.add_space(24.0);
                
                // Main content grid
                ui.horizontal(|ui| {
                    // Left column - Quick actions & Tips
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() * 0.45);
                        
                        // Quick actions
                        if let Some(a) = self.render_quick_actions(ui) {
                            action = Some(a);
                        }
                        ui.add_space(20.0);
                        
                        // Tips carousel
                        self.render_tips(ui);
                    });
                    
                    ui.add_space(20.0);
                    
                    // Right column - Recent activity
                    ui.vertical(|ui| {
                        self.render_recent_activity(ui);
                    });
                });
                
                ui.add_space(40.0);
            });
        
        action
    }
    
    fn render_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                // Greeting based on time of day
                let greeting = get_greeting();
                ui.label(RichText::new(greeting).size(14.0).color(TEXT_SECONDARY));
                ui.add_space(4.0);
                ui.label(RichText::new("Welcome to Sutra AI").size(32.0).color(TEXT_PRIMARY).strong());
                ui.add_space(4.0);
                ui.label(RichText::new("Your personal knowledge reasoning engine").size(14.0).color(TEXT_MUTED));
            });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Health indicator
                let (health_color, health_label) = if self.stats.health_score >= 0.8 {
                    (SUCCESS, "Healthy")
                } else if self.stats.health_score >= 0.5 {
                    (WARNING, "Good")
                } else {
                    (ERROR, "Needs Attention")
                };
                
                egui::Frame::none()
                    .fill(health_color.gamma_multiply(0.15))
                    .rounding(Rounding::same(20.0))
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 4.0, health_color);
                            ui.add_space(8.0);
                            ui.label(RichText::new(health_label).size(13.0).color(health_color));
                        });
                    });
            });
        });
    }
    
    fn render_stats_cards(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Concepts card
            self.stat_card(
                ui,
                "📚",
                "Concepts",
                &self.stats.total_concepts.to_string(),
                &format!("+{} today", self.stats.concepts_today),
                PRIMARY,
            );
            
            ui.add_space(12.0);
            
            // Connections card
            self.stat_card(
                ui,
                "🔗",
                "Connections",
                &self.stats.total_connections.to_string(),
                "Knowledge graph edges",
                SECONDARY,
            );
            
            ui.add_space(12.0);
            
            // Queries card
            self.stat_card(
                ui,
                "🔍",
                "Queries",
                &self.stats.queries_today.to_string(),
                "Today",
                ACCENT,
            );
            
            ui.add_space(12.0);
            
            // Storage card
            self.stat_card(
                ui,
                "💾",
                "Storage",
                &format!("{:.1} MB", self.stats.storage_mb),
                "Local data",
                SUCCESS,
            );
        });
    }
    
    fn stat_card(&self, ui: &mut egui::Ui, icon: &str, label: &str, value: &str, subtitle: &str, color: Color32) {
        let card_width = (ui.available_width() - 36.0) / 4.0;
        
        egui::Frame::none()
            .fill(BG_ELEVATED)
            .stroke(Stroke::new(1.0, color.gamma_multiply(0.2)))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(card_width);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Icon with background
                        egui::Frame::none()
                            .fill(color.gamma_multiply(0.15))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(icon).size(18.0));
                            });
                        ui.add_space(8.0);
                        ui.label(RichText::new(label).size(13.0).color(TEXT_SECONDARY));
                    });
                    
                    ui.add_space(12.0);
                    
                    ui.label(RichText::new(value).size(28.0).color(TEXT_PRIMARY).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(subtitle).size(11.0).color(TEXT_MUTED));
                });
            });
    }
    
    fn render_quick_actions(&self, ui: &mut egui::Ui) -> Option<HomeAction> {
        let mut action = None;
        
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Quick Actions").size(15.0).color(TEXT_PRIMARY).strong());
            });
            
            ui.add_space(16.0);
            
            // Action buttons
            if self.action_button(ui, "💬", "Start Chatting", "Ask questions or teach knowledge", PRIMARY) {
                action = Some(HomeAction::GoToChat);
            }
            
            ui.add_space(8.0);
            
            if self.action_button(ui, "⚡", "Quick Learn", "Rapidly add new concepts", SECONDARY) {
                action = Some(HomeAction::GoToQuickLearn);
            }
            
            ui.add_space(8.0);
            
            if self.action_button(ui, "🕸️", "View Graph", "Visualize knowledge connections", ACCENT) {
                action = Some(HomeAction::GoToGraph);
            }
            
            ui.add_space(8.0);
            
            if self.action_button(ui, "📥", "Import Data", "Load from JSON or CSV", SUCCESS) {
                action = Some(HomeAction::GoToImport);
            }
        });
        
        action
    }
    
    fn action_button(&self, ui: &mut egui::Ui, icon: &str, label: &str, hint: &str, color: Color32) -> bool {
        let response = egui::Frame::none()
            .fill(BG_WIDGET)
            .stroke(Stroke::new(1.0, Color32::from_rgb(55, 55, 80)))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(14.0, 12.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    // Icon
                    egui::Frame::none()
                        .fill(color.gamma_multiply(0.15))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(icon).size(16.0));
                        });
                    
                    ui.add_space(12.0);
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new(label).size(14.0).color(TEXT_PRIMARY));
                        ui.label(RichText::new(hint).size(11.0).color(TEXT_MUTED));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("→").size(16.0).color(TEXT_MUTED));
                    });
                });
            });
        
        // Make clickable
        let rect = response.response.rect;
        let sense_response = ui.interact(rect, ui.id().with(label), Sense::click());
        
        if sense_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        
        sense_response.clicked()
    }
    
    fn render_tips(&mut self, ui: &mut egui::Ui) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("💡").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Pro Tip").size(15.0).color(TEXT_PRIMARY).strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("Next →").size(12.0).color(TEXT_SECONDARY))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                    ).clicked() {
                        self.next_tip();
                    }
                });
            });
            
            ui.add_space(12.0);
            
            if let Some(tip) = self.tips.get(self.current_tip) {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tip.icon).size(24.0));
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tip.title).size(14.0).color(TEXT_PRIMARY).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new(tip.description).size(13.0).color(TEXT_SECONDARY));
                        
                        if let Some(cmd) = tip.command {
                            ui.add_space(8.0);
                            egui::Frame::none()
                                .fill(BG_DARK)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(cmd).size(12.0).color(PRIMARY).monospace());
                                });
                        }
                    });
                });
            }
            
            ui.add_space(8.0);
            
            // Progress dots
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (self.tips.len() as f32 * 12.0)) / 2.0);
                for i in 0..self.tips.len() {
                    let color = if i == self.current_tip { PRIMARY } else { TEXT_MUTED.gamma_multiply(0.3) };
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                    ui.add_space(4.0);
                }
            });
        });
    }
    
    fn render_recent_activity(&self, ui: &mut egui::Ui) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("📋").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Recent Activity").size(15.0).color(TEXT_PRIMARY).strong());
            });
            
            ui.add_space(12.0);
            
            if self.recent_activity.is_empty() {
                // Empty state
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("📭").size(32.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new("No recent activity").size(14.0).color(TEXT_MUTED));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Start by teaching Sutra something new!").size(12.0).color(TEXT_MUTED));
                    ui.add_space(20.0);
                });
            } else {
                // Activity list
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for activity in self.recent_activity.iter().take(10) {
                            self.activity_item(ui, activity);
                            ui.add_space(8.0);
                        }
                    });
            }
        });
    }
    
    fn activity_item(&self, ui: &mut egui::Ui, item: &ActivityItem) {
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.5))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(item.icon).size(14.0));
                    ui.add_space(8.0);
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&item.description).size(13.0).color(TEXT_PRIMARY));
                        ui.label(RichText::new(format_time_ago(item.timestamp)).size(11.0).color(TEXT_MUTED));
                    });
                });
            });
    }
}

/// Actions from the home dashboard
#[derive(Debug, Clone, Copy)]
pub enum HomeAction {
    GoToChat,
    GoToQuickLearn,
    GoToGraph,
    GoToImport,
    StartTour,
}

/// Create quick tips
fn create_quick_tips() -> Vec<QuickTip> {
    vec![
        QuickTip {
            title: "Teach with Relationships",
            description: "Use words like 'causes', 'before', 'after' to create rich knowledge connections.",
            command: Some("/learn Coffee causes alertness"),
            icon: "🔗",
        },
        QuickTip {
            title: "Ask Follow-up Questions",
            description: "Sutra understands context. Ask 'Why?' or 'What else?' after a response.",
            command: None,
            icon: "💬",
        },
        QuickTip {
            title: "Batch Import Knowledge",
            description: "Drop a CSV or JSON file to import multiple concepts at once.",
            command: None,
            icon: "📥",
        },
        QuickTip {
            title: "Explore Causality",
            description: "Use the Causality view to trace root causes of any concept.",
            command: None,
            icon: "🔍",
        },
        QuickTip {
            title: "Use Keyboard Shortcuts",
            description: "Type '/' to quickly access all commands from anywhere.",
            command: Some("/help"),
            icon: "⌨️",
        },
        QuickTip {
            title: "Export Your Knowledge",
            description: "Back up your knowledge base to JSON for portability.",
            command: None,
            icon: "💾",
        },
    ]
}

/// Get greeting based on time of day
fn get_greeting() -> &'static str {
    let hour = Local::now().hour();
    if hour < 12 {
        "Good morning ☀️"
    } else if hour < 17 {
        "Good afternoon 🌤️"
    } else if hour < 21 {
        "Good evening 🌅"
    } else {
        "Good night 🌙"
    }
}

/// Calculate health score
fn calculate_health_score(stats: &DashboardStats) -> f32 {
    let mut score: f32 = 0.5; // Base score
    
    // Bonus for having concepts
    if stats.total_concepts > 0 { score += 0.2; }
    if stats.total_concepts > 10 { score += 0.1; }
    if stats.total_concepts > 100 { score += 0.1; }
    
    // Bonus for connections
    if stats.total_connections > stats.total_concepts / 2 { score += 0.1; }
    
    score.min(1.0)
}

/// Format time as "X ago"
fn format_time_ago(dt: DateTime<Local>) -> String {
    let now = Local::now();
    let duration = now.signed_duration_since(dt);
    
    if duration.num_seconds() < 60 {
        "Just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hr ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}

use chrono::Timelike;
