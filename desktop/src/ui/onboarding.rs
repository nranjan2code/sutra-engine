//! Interactive Onboarding Tour
//!
//! A guided tour that highlights key features on first launch.
//! Uses overlay-based highlighting with step-by-step navigation.

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};
use crate::theme::{PRIMARY, PRIMARY_LIGHT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_DARK, BG_ELEVATED, SUCCESS};
use crate::ui::SidebarView;

/// Onboarding tour state
pub struct OnboardingTour {
    /// Whether the tour is currently active
    pub is_active: bool,
    /// Current step index
    pub current_step: usize,
    /// Whether this is the first launch (show tour automatically)
    pub first_launch: bool,
    /// Steps of the tour
    steps: Vec<TourStep>,
}

/// A single step in the tour
#[derive(Clone)]
struct TourStep {
    /// Title of the step
    title: &'static str,
    /// Description text
    description: &'static str,
    /// Icon/emoji for the step
    icon: &'static str,
    /// Target area to highlight (optional)
    target: TourTarget,
    /// Position of the tooltip
    tooltip_position: TooltipPosition,
}

/// Target area to highlight during a tour step
#[derive(Clone, Copy, PartialEq)]
pub enum TourTarget {
    /// The entire sidebar
    Sidebar,
    /// The chat area
    ChatArea,
    /// A specific sidebar item
    SidebarItem(SidebarView),
    /// The input area
    InputArea,
    /// The menu bar
    MenuBar,
    /// Center of screen (for welcome/finish)
    Center,
    /// The status bar
    StatusBar,
}

/// Position of the tooltip relative to the highlight
#[derive(Clone, Copy)]
enum TooltipPosition {
    Right,
    Left,
    Above,
    Below,
    Center,
}

impl Default for OnboardingTour {
    fn default() -> Self {
        Self {
            is_active: false,
            current_step: 0,
            first_launch: false,
            steps: create_tour_steps(),
        }
    }
}

impl OnboardingTour {
    /// Create a new onboarding tour
    pub fn new(first_launch: bool) -> Self {
        Self {
            is_active: first_launch,
            current_step: 0,
            first_launch,
            steps: create_tour_steps(),
        }
    }
    
    /// Check if the tour should show
    pub fn should_show(&self) -> bool {
        self.is_active && self.current_step < self.steps.len()
    }
    
    /// Start the tour
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_step = 0;
    }
    
    /// Skip/dismiss the tour
    pub fn dismiss(&mut self) {
        self.is_active = false;
        self.first_launch = false;
    }
    
    /// Move to the next step
    pub fn next_step(&mut self) {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
        } else {
            self.dismiss();
        }
    }
    
    /// Move to the previous step
    pub fn prev_step(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }
    
    /// Get the current target for highlighting
    pub fn current_target(&self) -> Option<TourTarget> {
        self.steps.get(self.current_step).map(|s| s.target)
    }
    
    /// Render the tour overlay
    pub fn render(&mut self, ctx: &egui::Context, screen_rect: Rect) -> Option<OnboardingAction> {
        if !self.should_show() {
            return None;
        }
        
        let mut action = None;
        
        // Get current step
        let step = match self.steps.get(self.current_step) {
            Some(s) => s.clone(),
            None => return None,
        };
        
        // Calculate highlight area based on target
        let highlight_rect = calculate_highlight_rect(step.target, screen_rect);
        
        // Draw overlay
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("onboarding_overlay"),
        ));
        
        // Semi-transparent background with cutout for highlight
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 200);
        
        if step.target == TourTarget::Center {
            // Full overlay for center messages
            painter.rect_filled(screen_rect, 0.0, overlay_color);
        } else {
            // Draw overlay with cutout
            draw_overlay_with_cutout(&painter, screen_rect, highlight_rect, overlay_color);
            
            // Highlight border with glow effect
            painter.rect_filled(
                highlight_rect.expand(4.0),
                Rounding::same(14.0),
                PRIMARY.gamma_multiply(0.3),
            );
            painter.rect_stroke(
                highlight_rect.expand(2.0),
                Rounding::same(12.0),
                Stroke::new(3.0, PRIMARY),
            );
        }
        
        // Calculate tooltip position
        let tooltip_rect = calculate_tooltip_rect(highlight_rect, screen_rect, step.tooltip_position);
        
        // Draw tooltip card
        painter.rect_filled(tooltip_rect, Rounding::same(16.0), BG_ELEVATED);
        painter.rect_stroke(tooltip_rect, Rounding::same(16.0), Stroke::new(1.0, PRIMARY.gamma_multiply(0.3)));
        
        // Draw arrow pointing to highlight
        if step.target != TourTarget::Center {
            draw_tooltip_arrow(&painter, tooltip_rect, highlight_rect, step.tooltip_position);
        }
        
        // Tooltip content using egui Area
        egui::Area::new(egui::Id::new("onboarding_tooltip"))
            .fixed_pos(tooltip_rect.min + Vec2::new(20.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_max_width(tooltip_rect.width() - 40.0);
                
                // Icon and title row
                ui.horizontal(|ui| {
                    ui.label(RichText::new(step.icon).size(28.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new(step.title).size(18.0).color(TEXT_PRIMARY).strong());
                });
                
                ui.add_space(12.0);
                
                // Description
                ui.label(RichText::new(step.description).size(14.0).color(TEXT_SECONDARY));
                
                ui.add_space(20.0);
                
                // Navigation buttons
                ui.horizontal(|ui| {
                    // Skip button
                    if ui.add(
                        egui::Button::new(RichText::new("Skip Tour").size(13.0).color(TEXT_MUTED))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                    ).clicked() {
                        action = Some(OnboardingAction::Dismiss);
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Next/Finish button
                        let is_last = self.current_step >= self.steps.len() - 1;
                        let next_text = if is_last { "Get Started!" } else { "Next →" };
                        let next_btn = egui::Button::new(RichText::new(next_text).size(14.0).color(Color32::WHITE))
                            .fill(PRIMARY)
                            .rounding(Rounding::same(8.0))
                            .min_size(Vec2::new(100.0, 32.0));
                        
                        if ui.add(next_btn).clicked() {
                            action = Some(OnboardingAction::Next);
                        }
                        
                        ui.add_space(8.0);
                        
                        // Back button (not on first step)
                        if self.current_step > 0 {
                            let back_btn = egui::Button::new(RichText::new("← Back").size(13.0).color(TEXT_SECONDARY))
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, TEXT_MUTED))
                                .rounding(Rounding::same(8.0));
                            
                            if ui.add(back_btn).clicked() {
                                action = Some(OnboardingAction::Prev);
                            }
                        }
                    });
                });
                
                ui.add_space(12.0);
                
                // Progress dots
                ui.horizontal(|ui| {
                    ui.add_space((tooltip_rect.width() - 40.0 - (self.steps.len() as f32 * 12.0)) / 2.0);
                    for i in 0..self.steps.len() {
                        let color = if i == self.current_step { PRIMARY } else { TEXT_MUTED.gamma_multiply(0.3) };
                        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                        ui.add_space(4.0);
                    }
                });
            });
        
        action
    }
}

/// Actions from the onboarding tour
#[derive(Debug, Clone, Copy)]
pub enum OnboardingAction {
    Next,
    Prev,
    Dismiss,
    NavigateTo(SidebarView),
}

/// Create the tour steps
fn create_tour_steps() -> Vec<TourStep> {
    vec![
        TourStep {
            title: "Welcome to Sutra AI!",
            description: "Your personal knowledge reasoning engine with temporal, causal, and semantic understanding. Let's take a quick tour of the key features.",
            icon: "🧠",
            target: TourTarget::Center,
            tooltip_position: TooltipPosition::Center,
        },
        TourStep {
            title: "The Chat Interface",
            description: "This is where you interact with Sutra. Type /learn to teach new knowledge, or just ask questions to retrieve what you've learned.",
            icon: "💬",
            target: TourTarget::ChatArea,
            tooltip_position: TooltipPosition::Left,
        },
        TourStep {
            title: "Slash Commands",
            description: "Type '/' to see available commands:\n• /learn - Teach new knowledge\n• /search - Find concepts\n• /help - Show all commands\n• /stats - View statistics",
            icon: "⚡",
            target: TourTarget::InputArea,
            tooltip_position: TooltipPosition::Above,
        },
        TourStep {
            title: "Knowledge Browser",
            description: "Browse all concepts you've taught Sutra. View details, explore connections, and manage your knowledge base.",
            icon: "📚",
            target: TourTarget::SidebarItem(SidebarView::Knowledge),
            tooltip_position: TooltipPosition::Right,
        },
        TourStep {
            title: "Graph Visualization",
            description: "See your knowledge as an interactive graph! Pan, zoom, and explore connections between concepts visually.",
            icon: "🕸️",
            target: TourTarget::SidebarItem(SidebarView::Graph),
            tooltip_position: TooltipPosition::Right,
        },
        TourStep {
            title: "Analysis Tools",
            description: "Explore temporal relationships (Timeline), trace causality (Causality), and find reasoning paths between concepts.",
            icon: "🔍",
            target: TourTarget::SidebarItem(SidebarView::Timeline),
            tooltip_position: TooltipPosition::Right,
        },
        TourStep {
            title: "Analytics Dashboard",
            description: "Monitor your knowledge base growth, query performance, and recent activity in real-time.",
            icon: "📊",
            target: TourTarget::SidebarItem(SidebarView::Analytics),
            tooltip_position: TooltipPosition::Right,
        },
        TourStep {
            title: "You're All Set!",
            description: "Start by teaching Sutra something:\n\n/learn Rust is a systems programming language\n\nThen ask questions to see the magic! Your data stays completely local and private.",
            icon: "🚀",
            target: TourTarget::Center,
            tooltip_position: TooltipPosition::Center,
        },
    ]
}

/// Calculate the highlight rectangle for a given target
fn calculate_highlight_rect(target: TourTarget, screen_rect: Rect) -> Rect {
    match target {
        TourTarget::Sidebar => Rect::from_min_size(
            screen_rect.min + Vec2::new(0.0, 36.0), // Below menu bar
            Vec2::new(200.0, screen_rect.height() - 68.0), // Minus status bar
        ),
        TourTarget::ChatArea => Rect::from_min_size(
            screen_rect.min + Vec2::new(216.0, 52.0),
            Vec2::new(screen_rect.width() - 232.0, screen_rect.height() - 100.0),
        ),
        TourTarget::SidebarItem(view) => {
            let y_offset = match view {
                SidebarView::Chat => 130.0,
                SidebarView::Knowledge => 190.0,
                SidebarView::Search => 250.0,
                SidebarView::Graph => 330.0,
                SidebarView::Paths => 390.0,
                SidebarView::Timeline => 450.0,
                SidebarView::Causal => 510.0,
                SidebarView::Analytics => 590.0,
                SidebarView::Query => 650.0,
                SidebarView::Export => 710.0,
                SidebarView::Settings => 780.0,
                _ => 130.0,
            };
            Rect::from_min_size(
                screen_rect.min + Vec2::new(12.0, y_offset),
                Vec2::new(176.0, 56.0),
            )
        }
        TourTarget::InputArea => Rect::from_min_size(
            screen_rect.min + Vec2::new(216.0, screen_rect.height() - 120.0),
            Vec2::new(screen_rect.width() - 232.0, 80.0),
        ),
        TourTarget::MenuBar => Rect::from_min_size(
            screen_rect.min,
            Vec2::new(screen_rect.width(), 36.0),
        ),
        TourTarget::StatusBar => Rect::from_min_size(
            screen_rect.min + Vec2::new(0.0, screen_rect.height() - 32.0),
            Vec2::new(screen_rect.width(), 32.0),
        ),
        TourTarget::Center => Rect::from_center_size(
            screen_rect.center(),
            Vec2::new(0.0, 0.0),
        ),
    }
}

/// Calculate tooltip rectangle position
fn calculate_tooltip_rect(highlight: Rect, screen: Rect, position: TooltipPosition) -> Rect {
    let tooltip_size = Vec2::new(380.0, 260.0);
    
    let pos = match position {
        TooltipPosition::Right => Pos2::new(
            (highlight.right() + 20.0).min(screen.right() - tooltip_size.x - 20.0),
            highlight.center().y - tooltip_size.y / 2.0,
        ),
        TooltipPosition::Left => Pos2::new(
            (highlight.left() - tooltip_size.x - 20.0).max(20.0),
            highlight.center().y - tooltip_size.y / 2.0,
        ),
        TooltipPosition::Above => Pos2::new(
            highlight.center().x - tooltip_size.x / 2.0,
            (highlight.top() - tooltip_size.y - 20.0).max(50.0),
        ),
        TooltipPosition::Below => Pos2::new(
            highlight.center().x - tooltip_size.x / 2.0,
            highlight.bottom() + 20.0,
        ),
        TooltipPosition::Center => Pos2::new(
            screen.center().x - tooltip_size.x / 2.0,
            screen.center().y - tooltip_size.y / 2.0,
        ),
    };
    
    // Clamp to screen bounds
    let clamped_pos = Pos2::new(
        pos.x.clamp(20.0, screen.right() - tooltip_size.x - 20.0),
        pos.y.clamp(50.0, screen.bottom() - tooltip_size.y - 50.0),
    );
    
    Rect::from_min_size(clamped_pos, tooltip_size)
}

/// Draw overlay with cutout for highlight area
fn draw_overlay_with_cutout(painter: &egui::Painter, screen: Rect, cutout: Rect, color: Color32) {
    // Top rectangle
    if cutout.top() > screen.top() {
        painter.rect_filled(
            Rect::from_min_max(screen.min, Pos2::new(screen.right(), cutout.top())),
            0.0,
            color,
        );
    }
    // Bottom rectangle
    if cutout.bottom() < screen.bottom() {
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(screen.left(), cutout.bottom()), screen.max),
            0.0,
            color,
        );
    }
    // Left rectangle
    if cutout.left() > screen.left() {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(screen.left(), cutout.top()),
                Pos2::new(cutout.left(), cutout.bottom()),
            ),
            0.0,
            color,
        );
    }
    // Right rectangle
    if cutout.right() < screen.right() {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(cutout.right(), cutout.top()),
                Pos2::new(screen.right(), cutout.bottom()),
            ),
            0.0,
            color,
        );
    }
}

/// Draw arrow from tooltip to highlight
fn draw_tooltip_arrow(painter: &egui::Painter, tooltip: Rect, highlight: Rect, position: TooltipPosition) {
    let arrow_size = 12.0;
    
    let (start, end) = match position {
        TooltipPosition::Right => (
            Pos2::new(tooltip.left(), tooltip.center().y),
            Pos2::new(highlight.right() + 8.0, highlight.center().y),
        ),
        TooltipPosition::Left => (
            Pos2::new(tooltip.right(), tooltip.center().y),
            Pos2::new(highlight.left() - 8.0, highlight.center().y),
        ),
        TooltipPosition::Above => (
            Pos2::new(tooltip.center().x, tooltip.bottom()),
            Pos2::new(highlight.center().x, highlight.top() - 8.0),
        ),
        TooltipPosition::Below => (
            Pos2::new(tooltip.center().x, tooltip.top()),
            Pos2::new(highlight.center().x, highlight.bottom() + 8.0),
        ),
        TooltipPosition::Center => return,
    };
    
    // Draw line
    painter.line_segment([start, end], Stroke::new(2.0, PRIMARY));
    
    // Draw arrowhead
    let dir = (end - start).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_base = end - dir * arrow_size;
    
    painter.add(egui::Shape::convex_polygon(
        vec![
            end,
            arrow_base + perp * (arrow_size / 2.0),
            arrow_base - perp * (arrow_size / 2.0),
        ],
        PRIMARY,
        Stroke::NONE,
    ));
}

/// Check if this is the first launch by looking for a marker file
pub fn is_first_launch() -> bool {
    if let Some(proj_dirs) = directories::ProjectDirs::from("ai", "sutra", "SutraDesktop") {
        let marker_path = proj_dirs.data_dir().join(".onboarding_complete");
        !marker_path.exists()
    } else {
        true
    }
}

/// Mark onboarding as complete
pub fn mark_onboarding_complete() {
    if let Some(proj_dirs) = directories::ProjectDirs::from("ai", "sutra", "SutraDesktop") {
        let marker_path = proj_dirs.data_dir().join(".onboarding_complete");
        let _ = std::fs::write(marker_path, "1");
    }
}
