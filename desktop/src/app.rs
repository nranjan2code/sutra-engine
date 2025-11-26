//! Main application - THIN wrapper around sutra-storage
//!
//! This module provides the GUI lifecycle management.
//! ALL storage logic comes from sutra_storage crate - NO duplication.

use std::path::PathBuf;
use std::sync::Arc;
use eframe::egui;
use tracing::{info, error, warn};
use directories::ProjectDirs;

// ============================================================================
// REUSE EXISTING STORAGE ENGINE - NO DUPLICATION
// ============================================================================
use sutra_storage::{
    ConcurrentMemory,
    ConcurrentConfig,
    ConcurrentStats,
    AdaptiveReconcilerConfig,
    ConceptId,
    ConceptNode,
};

use crate::ui::{
    Sidebar, SidebarView, ChatPanel, KnowledgePanel, SettingsPanel, StatusBar,
    ChatAction, KnowledgeAction, SettingsAction, ConnectionStatus,
    StorageStatsUI, StorageStatus,
};
use crate::theme::{BG_DARK, BG_PANEL};

/// Main Sutra Desktop application
pub struct SutraApp {
    // ========================================================================
    // STORAGE: Direct use of sutra_storage::ConcurrentMemory
    // This is THE SAME storage engine used by Docker deployments
    // ========================================================================
    storage: Arc<ConcurrentMemory>,
    data_dir: PathBuf,
    
    // UI Components (thin layer)
    sidebar: Sidebar,
    chat: ChatPanel,
    knowledge: KnowledgePanel,
    settings: SettingsPanel,
    status_bar: StatusBar,
    
    // State
    initialized: bool,
}

impl SutraApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Get platform-specific data directory
        let data_dir = get_data_directory();
        info!("Data directory: {:?}", data_dir);
        
        // Create storage directory
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        
        // ====================================================================
        // Initialize storage using EXISTING sutra_storage crate
        // Same config structure as Docker deployment
        // ====================================================================
        let config = ConcurrentConfig {
            storage_path: data_dir.clone(),
            memory_threshold: 10_000,  // Desktop-appropriate threshold
            vector_dimension: 256,      // Fast mode for desktop (Matryoshka)
            adaptive_reconciler_config: AdaptiveReconcilerConfig {
                base_interval_ms: 100,  // More relaxed for desktop
                ..Default::default()
            },
            ..Default::default()
        };
        
        let storage = ConcurrentMemory::new(config);
        let stats = storage.stats();
        
        info!(
            "Storage initialized: {} concepts, {} edges",
            stats.snapshot.concept_count,
            stats.snapshot.edge_count,
        );
        
        // Initialize settings with data path
        let mut settings = SettingsPanel::default();
        settings.data_path = data_dir.display().to_string();
        settings.stats = convert_to_ui_stats(&stats);
        
        // Initialize status bar
        let mut status_bar = StatusBar::default();
        status_bar.set_status(ConnectionStatus::Connected);
        status_bar.set_concept_count(stats.snapshot.concept_count);
        
        Self {
            storage: Arc::new(storage),
            data_dir,
            sidebar: Sidebar::default(),
            chat: ChatPanel::default(),
            knowledge: KnowledgePanel::default(),
            settings,
            status_bar,
            initialized: false,
        }
    }
    
    /// Handle chat actions - uses storage directly
    fn handle_chat_action(&mut self, action: ChatAction) {
        match action {
            ChatAction::Learn(content) => {
                self.chat.is_processing = true;
                
                // Generate concept ID using MD5 (same as storage server does)
                let hash = md5::compute(content.as_bytes());
                let concept_id = ConceptId::from_bytes(hash.0);
                
                // ============================================================
                // DIRECT USE of sutra_storage::ConcurrentMemory::learn_concept
                // Same API as used by TCP server
                // ============================================================
                match self.storage.learn_concept(
                    concept_id,
                    content.as_bytes().to_vec(),
                    None,    // No embedding in desktop basic mode
                    1.0,     // strength
                    1.0,     // confidence
                ) {
                    Ok(_) => {
                        self.chat.add_response(format!(
                            "✅ Learned! Stored as concept `{}`.\n\nYou can now ask me questions about this.",
                            &concept_id.to_hex()[..8]
                        ));
                        let preview = if content.len() > 30 { &content[..30] } else { &content };
                        self.status_bar.set_activity(format!("Learned: {}...", preview));
                        self.refresh_stats();
                    }
                    Err(e) => {
                        self.chat.add_response(format!("❌ Failed to learn: {:?}", e));
                        error!("Learn failed: {:?}", e);
                    }
                }
                
                self.chat.is_processing = false;
            }
            
            ChatAction::Query(query) => {
                self.chat.is_processing = true;
                
                // Use ConcurrentMemory's text_search (shared with enterprise)
                let results = self.storage.text_search(&query, 5);
                
                if results.is_empty() {
                    self.chat.add_response(
                        "🤔 I don't have knowledge about that yet.\n\nTeach me with: `/learn <your knowledge>`".to_string()
                    );
                } else {
                    let mut response = String::from("Based on my knowledge:\n\n");
                    for (i, (_id, content, confidence)) in results.iter().enumerate() {
                        response.push_str(&format!("{}. {} (relevance: {:.0}%)\n", i + 1, content, confidence * 100.0));
                    }
                    self.chat.add_response(response);
                }
                
                let preview = if query.len() > 30 { &query[..30] } else { &query };
                self.status_bar.set_activity(format!("Searched: {}", preview));
                self.chat.is_processing = false;
            }
            
            ChatAction::Help => {
                // Help is handled directly in ChatPanel::parse_command
            }
            
            ChatAction::Clear => {
                self.chat.messages.clear();
                self.chat.add_response("🧹 Chat cleared. Ready for new conversations!".to_string());
                self.status_bar.set_activity("Chat cleared");
            }
            
            ChatAction::Stats => {
                let stats = self.storage.stats();
                let hnsw_stats = self.storage.hnsw_stats();
                self.chat.add_response(format!(
                    "📊 **Knowledge Statistics**\n\n\
                    • **Concepts:** {}\n\
                    • **Connections:** {}\n\
                    • **Vectors indexed:** {}\n\
                    • **Vector dimension:** {}\n\
                    • **Data path:** {}",
                    stats.snapshot.concept_count,
                    stats.snapshot.edge_count,
                    hnsw_stats.indexed_vectors,
                    hnsw_stats.dimension,
                    self.settings.data_path
                ));
                self.status_bar.set_activity("Stats displayed");
            }
        }
    }
    
    /// Handle knowledge panel actions
    fn handle_knowledge_action(&mut self, action: KnowledgeAction) {
        match action {
            KnowledgeAction::Search(query) => {
                self.knowledge.is_loading = true;
                
                let concepts = if query.is_empty() {
                    self.load_all_concepts(100)
                } else {
                    // Use ConcurrentMemory's text_search (shared with enterprise)
                    let results = self.storage.text_search(&query, 50);
                    
                    // Convert to ConceptInfo
                    results
                        .into_iter()
                        .filter_map(|(concept_id, _, _)| {
                            let snapshot = self.storage.get_snapshot();
                            snapshot.get_concept(&concept_id)
                                .map(|node| node_to_concept_info(&node, &self.storage))
                        })
                        .collect()
                };
                
                self.knowledge.set_concepts(concepts);
            }
            
            KnowledgeAction::Refresh => {
                self.knowledge.is_loading = true;
                let concepts = self.load_all_concepts(100);
                self.knowledge.set_concepts(concepts);
                self.status_bar.set_activity("Knowledge refreshed");
                self.refresh_stats();
            }
            
            KnowledgeAction::SelectConcept(id) => {
                self.knowledge.selected_concept = Some(id);
            }
        }
    }
    
    /// Load all concepts from storage
    fn load_all_concepts(&self, limit: usize) -> Vec<ConceptInfo> {
        let snapshot = self.storage.get_snapshot();
        
        snapshot.all_concepts()
            .into_iter()
            .take(limit)
            .map(|node| node_to_concept_info(&node, &self.storage))
            .collect()
    }
    
    /// Handle settings actions
    fn handle_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::Save => {
                match self.storage.flush() {
                    Ok(_) => {
                        self.status_bar.set_activity("Settings saved");
                        info!("Storage flushed");
                    }
                    Err(e) => {
                        error!("Flush failed: {}", e);
                    }
                }
            }
            
            SettingsAction::ExportData => {
                // TODO: Implement export
                warn!("Export not yet implemented");
                self.status_bar.set_activity("Export not yet implemented");
            }
            
            SettingsAction::ImportData => {
                // TODO: Implement import
                warn!("Import not yet implemented");
                self.status_bar.set_activity("Import not yet implemented");
            }
            
            SettingsAction::ClearData => {
                // TODO: Implement clear with confirmation
                warn!("Clear data not yet implemented");
                self.status_bar.set_activity("Clear data not yet implemented");
            }
        }
    }
    
    /// Refresh statistics from storage
    fn refresh_stats(&mut self) {
        let stats = self.storage.stats();
        self.status_bar.set_concept_count(stats.snapshot.concept_count);
        self.settings.update_stats(convert_to_ui_stats(&stats));
    }
}

impl eframe::App for SutraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initial data load
        if !self.initialized {
            self.handle_knowledge_action(KnowledgeAction::Refresh);
            self.initialized = true;
        }
        
        // Request repaint for animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        
        // Sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(200.0)
            .frame(egui::Frame::none().fill(BG_DARK))
            .show(ctx, |ui| {
                self.sidebar.ui(ui);
            });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(28.0)
            .show(ctx, |ui| {
                self.status_bar.ui(ui);
            });
        
        // Main content
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(16.0))
            .show(ctx, |ui| {
                match self.sidebar.current_view {
                    SidebarView::Chat => {
                        if let Some(action) = self.chat.ui(ui) {
                            self.handle_chat_action(action);
                        }
                    }
                    SidebarView::Knowledge | SidebarView::Search => {
                        if let Some(action) = self.knowledge.ui(ui) {
                            self.handle_knowledge_action(action);
                        }
                    }
                    SidebarView::Settings => {
                        if let Some(action) = self.settings.ui(ui) {
                            self.handle_settings_action(action);
                        }
                    }
                }
            });
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("Shutting down - flushing storage");
        let _ = self.storage.flush();
    }
}

// ============================================================================
// Helper types and functions
// ============================================================================

/// Concept info for UI display
#[derive(Debug, Clone)]
pub struct ConceptInfo {
    pub id: String,
    pub content: String,
    pub strength: f32,
    pub confidence: f32,
    pub neighbors: Vec<String>,
}

/// Convert ConceptNode to ConceptInfo
fn node_to_concept_info(node: &ConceptNode, storage: &ConcurrentMemory) -> ConceptInfo {
    let content = std::str::from_utf8(&node.content)
        .unwrap_or("<binary>")
        .to_string();
    
    let neighbors: Vec<String> = storage
        .query_neighbors(&node.id)
        .into_iter()
        .map(|id| id.to_hex())
        .collect();
    
    ConceptInfo {
        id: node.id.to_hex(),
        content,
        strength: node.strength,
        confidence: node.confidence,
        neighbors,
    }
}

/// Convert sutra_storage stats to UI stats
fn convert_to_ui_stats(stats: &ConcurrentStats) -> StorageStatsUI {
    StorageStatsUI {
        total_concepts: stats.snapshot.concept_count,
        vector_dimensions: 256,  // Desktop uses 256-dim
        data_path: String::new(),  // Set by caller
        status: StorageStatus::Running,
    }
}

/// Get platform-specific data directory
fn get_data_directory() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("ai", "sutra", "SutraDesktop") {
        proj_dirs.data_dir().to_path_buf()
    } else {
        PathBuf::from("./sutra_data")
    }
}
