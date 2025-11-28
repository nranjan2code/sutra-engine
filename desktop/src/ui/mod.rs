//! UI components for Sutra Desktop
//!
//! This module contains all UI panels and components for the enhanced desktop application.
//! Each panel is designed to leverage the full capabilities of the sutra-storage engine
//! while maintaining a clean, production-grade user experience.

// Core panels (original)
pub mod sidebar;
pub mod chat;
pub mod knowledge;
pub mod quick_learn;
pub mod settings;
pub mod status_bar;

// Enhanced panels (new)
pub mod graph_view;
pub mod reasoning_paths;
pub mod temporal_view;
pub mod causal_view;
pub mod analytics;
pub mod query_builder;
pub mod export_import;

// New UX features (v3.4.0)
pub mod onboarding;
pub mod undo_redo;
pub mod home;

// Re-exports for core panels
pub use sidebar::{Sidebar, SidebarView};
pub use chat::{ChatPanel, ChatAction};
pub use knowledge::{KnowledgePanel, KnowledgeAction};
pub use quick_learn::{QuickLearnPanel, QuickLearnAction};
pub use settings::{SettingsPanel, SettingsAction, StorageStatsUI, StorageStatus};
pub use status_bar::{StatusBar, ConnectionStatus};

// Re-exports for enhanced panels (use actual struct/enum names from modules)
pub use graph_view::{GraphView, GraphAction};
pub use reasoning_paths::{ReasoningPathsPanel, ReasoningPathsAction};
pub use temporal_view::{TemporalView, TemporalViewAction};
pub use causal_view::{CausalView, CausalViewAction};
pub use analytics::{AnalyticsDashboard, AnalyticsAction};
pub use query_builder::{QueryBuilder, QueryBuilderAction};
pub use export_import::{ExportImportPanel, ExportImportAction};

// Re-exports for new UX features
pub use onboarding::{OnboardingTour, OnboardingAction, is_first_launch, mark_onboarding_complete};
pub use undo_redo::{CommandHistory, Command, CommandType, UndoRedoResult};
pub use home::{HomeDashboard, HomeAction, DashboardActivityType};

