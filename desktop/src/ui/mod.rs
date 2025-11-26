//! UI components for Sutra Desktop

pub mod sidebar;
pub mod chat;
pub mod knowledge;
pub mod settings;
pub mod status_bar;

pub use sidebar::{Sidebar, SidebarView};
pub use chat::{ChatPanel, ChatAction};
pub use knowledge::{KnowledgePanel, KnowledgeAction};
pub use settings::{SettingsPanel, SettingsAction, StorageStatsUI, StorageStatus};
pub use status_bar::{StatusBar, ConnectionStatus};
