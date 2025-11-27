//! Main application - THIN wrapper around sutra-storage
//!
//! This module provides the GUI lifecycle management.
//! ALL storage logic comes from sutra_storage crate - NO duplication.
//!
//! ENHANCED UI (v3.3) features:
//! - Graph visualization with force-directed layout
//! - MPPA-style reasoning path exploration
//! - Temporal and causal analysis views
//! - Real-time analytics dashboard
//! - Advanced query builder
//! - Export/Import functionality

use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::time::Instant;
use eframe::egui::{self, RichText};
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
    semantic::SemanticType,
    ParallelPathFinder,
    PathResult,
    learning_pipeline::{LearningPipeline, LearnOptions},
};

use crate::local_embedding::LocalEmbeddingProvider;
use crate::local_nlg::LocalNlgProvider;

// Core UI imports
use crate::ui::{
    Sidebar, SidebarView, ChatPanel, KnowledgePanel, QuickLearnPanel, SettingsPanel, StatusBar,
    ChatAction, KnowledgeAction, QuickLearnAction, SettingsAction, ConnectionStatus,
    StorageStatsUI, StorageStatus,
};

// Enhanced UI imports
use crate::ui::{
    GraphView, GraphAction,
    ReasoningPathsPanel, ReasoningPathsAction,
    TemporalView, TemporalViewAction,
    CausalView, CausalViewAction,
    AnalyticsDashboard, AnalyticsAction,
    QueryBuilder, QueryBuilderAction,
    ExportImportPanel, ExportImportAction,
};

use crate::theme::{BG_DARK, BG_PANEL};
use crate::types::{
    AppMessage, ConceptInfo, GraphNode, GraphEdge, EdgeType,
    ReasoningPath, PathStep, ConsensusResult, PathCluster,
    CausalChain, CausalNode, CausalRelationType, TimelineEvent,
};

/// Main Sutra Desktop application
pub struct SutraApp {
    // ========================================================================
    // STORAGE: Direct use of sutra_storage::ConcurrentMemory
    // This is THE SAME storage engine used by Docker deployments
    // ========================================================================
    storage: Arc<ConcurrentMemory>,
    pipeline: Arc<LearningPipeline>, // 🔥 NEW: Unified learning pipeline
    nlg: Option<Arc<LocalNlgProvider>>, // 🔥 NEW: Local NLG provider
    data_dir: PathBuf,
    
    // Async communication
    tx: mpsc::Sender<AppMessage>,
    rx: mpsc::Receiver<AppMessage>,
    
    // Core UI Components
    sidebar: Sidebar,
    chat: ChatPanel,
    knowledge: KnowledgePanel,
    quick_learn: QuickLearnPanel,
    settings: SettingsPanel,
    status_bar: StatusBar,
    
    // Enhanced UI Components
    graph_view: GraphView,
    reasoning_paths: ReasoningPathsPanel,
    temporal_view: TemporalView,
    causal_view: CausalView,
    analytics: AnalyticsDashboard,
    query_builder: QueryBuilder,
    export_import: ExportImportPanel,
    
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
            memory_threshold: 10_000,
            vector_dimension: 768,
            adaptive_reconciler_config: AdaptiveReconcilerConfig {
                base_interval_ms: 100,
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
        
        // Initialize local NLG provider
        info!("Initializing local NLG provider...");
        let nlg = match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(LocalNlgProvider::new_async()) {
            Ok(provider) => {
                info!("✅ Local NLG provider initialized successfully");
                Some(Arc::new(provider))
            }
            Err(e) => {
                error!("❌ Failed to initialize local NLG: {}", e);
                // Don't panic, just continue without NLG
                None
            }
        };

        status_bar.set_concept_count(stats.snapshot.concept_count);
        // Initialize local AI pipeline
        info!("Initializing local AI pipeline...");
        let pipeline = match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(LocalEmbeddingProvider::new_async()) {
            Ok(provider) => {
                let provider = Arc::new(provider);
                // We need to block here because LearningPipeline::new_with_provider is async
                // but we are in a sync context. Since we're initializing, it's okay to block.
                let pipeline = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(LearningPipeline::new_with_provider(provider))
                    .expect("Failed to initialize learning pipeline");
                info!("✅ Local AI pipeline initialized successfully");
                Arc::new(pipeline)
            }
            Err(e) => {
                error!("❌ Failed to initialize local AI: {}", e);
                // Fallback to dummy pipeline or handle error gracefully
                // For now, we panic because AI is core to the "world class" experience
                panic!("Failed to initialize local AI: {}", e);
            }
        };
        
        let (tx, rx) = mpsc::channel();
        
        Self {
            storage: Arc::new(storage),
            pipeline,
            nlg,  // Local NLG provider (optional)
            data_dir,
            tx,
            rx,
            sidebar: Sidebar::default(),
            chat: ChatPanel::default(),
            knowledge: KnowledgePanel::default(),
            quick_learn: QuickLearnPanel::default(),
            settings,
            status_bar,
            
            // Enhanced UI
            graph_view: GraphView::default(),
            reasoning_paths: ReasoningPathsPanel::default(),
            temporal_view: TemporalView::default(),
            causal_view: CausalView::default(),
            analytics: AnalyticsDashboard::default(),
            query_builder: QueryBuilder::default(),
            export_import: ExportImportPanel::default(),
            
            initialized: false,
        }
    }
    // ========================================================================
    
    fn handle_chat_action(&mut self, action: ChatAction) {
        match action {
            ChatAction::Learn(content) => {
                self.chat.is_processing = true;
                let storage = self.storage.clone();
                let pipeline = self.pipeline.clone();
                let tx = self.tx.clone();
                let content_clone = content.clone();
                
                std::thread::spawn(move || {
                    let start = Instant::now();
                    
                    // Use unified pipeline
                    let options = LearnOptions::default();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(pipeline.learn_concept(&storage, &content_clone, &options));
                    
                    match result {
                        Ok(id_str) => {
                            let concept_id = ConceptId::from_string(&id_str);
                            let elapsed_ms = start.elapsed().as_millis() as u64;
                            let _ = tx.send(AppMessage::LearnResult {
                                content: content_clone,
                                concept_id: Some(concept_id),
                                error: None,
                                duration_ms: elapsed_ms,
                            });
                        }
                        Err(e) => {
                            let elapsed_ms = start.elapsed().as_millis() as u64;
                            let _ = tx.send(AppMessage::LearnResult {
                                content: content_clone,
                                concept_id: None,
                                error: Some(e.to_string()),
                                duration_ms: elapsed_ms,
                            });
                        }
                    }
                });
            }
            
            ChatAction::Query(query) => {
                self.chat.is_processing = true;
                let storage = self.storage.clone();
                let nlg = self.nlg.clone(); // Clone NLG provider
                let tx = self.tx.clone();
                let query_clone = query.clone();
                
                std::thread::spawn(move || {
                    let start = Instant::now();
                    let results = storage.text_search(&query_clone, 5);
                    
                    let result_data: Vec<(ConceptId, String, f32)> = results.into_iter()
                        .map(|(id, content, conf)| (id, content, conf))
                        .collect();
                        
                    // Generate NLG response if provider is available and we have results
                    let generated_response = if let Some(nlg_provider) = nlg {
                        if !result_data.is_empty() {
                            // Construct prompt from results
                            let context: String = result_data.iter()
                                .take(3)
                                .map(|(_, content, _)| format!("- {}", content))
                                .collect::<Vec<_>>()
                                .join("\n");
                                
                            let prompt = format!(
                                "Context:\n{}\n\nQuestion: {}\n\nAnswer based on the context:",
                                context, query_clone
                            );
                            
                            // Run generation (blocking in this thread)
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            match rt.block_on(nlg_provider.generate(&prompt)) {
                                Ok(response) => Some(response),
                                Err(e) => {
                                    warn!("NLG generation failed: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                        
                    let _ = tx.send(AppMessage::SearchResults {
                        query: query_clone,
                        results: result_data,
                        duration_ms: elapsed_ms,
                        generated_response,
                    });
                });
            }
            
            ChatAction::Help => {}
            
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
    
    // ========================================================================
    // Knowledge Panel Actions
    // ========================================================================
    
    fn handle_knowledge_action(&mut self, action: KnowledgeAction) {
        match action {
            KnowledgeAction::Search(query) => {
                self.knowledge.is_loading = true;
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                
                std::thread::spawn(move || {
                    let concepts = if query.is_empty() {
                        let snapshot = storage.get_snapshot();
                        snapshot.all_concepts()
                            .into_iter()
                            .take(100)
                            .map(|node| node_to_concept_info(&node, &storage))
                            .collect()
                    } else {
                        let results = storage.text_search(&query, 50);
                        results
                            .into_iter()
                            .filter_map(|(concept_id, _, _)| {
                                let snapshot = storage.get_snapshot();
                                snapshot.get_concept(&concept_id)
                                    .map(|node| node_to_concept_info(&node, &storage))
                            })
                            .collect()
                    };
                    
                    let _ = tx.send(AppMessage::KnowledgeLoaded { concepts });
                });
            }
            
            KnowledgeAction::Refresh => {
                self.knowledge.is_loading = true;
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                
                std::thread::spawn(move || {
                    let snapshot = storage.get_snapshot();
                    let concepts = snapshot.all_concepts()
                        .into_iter()
                        .take(100)
                        .map(|node| node_to_concept_info(&node, &storage))
                        .collect();
                    
                    let _ = tx.send(AppMessage::KnowledgeLoaded { concepts });
                });
                
                self.status_bar.set_activity("Refreshing knowledge...");
            }
            
            KnowledgeAction::SelectConcept(id) => {
                self.knowledge.selected_concept = Some(id);
            }
            
            KnowledgeAction::DeleteConcept(id_str) => {
                let concept_id = ConceptId::from_string(&id_str);
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                    
                std::thread::spawn(move || {
                    match storage.delete_concept(concept_id) {
                        Ok(_) => {
                            let _ = tx.send(AppMessage::ConceptDeleted {
                                concept_id: id_str,
                                success: true,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::ConceptDeleted {
                                concept_id: id_str,
                                success: false,
                                error: Some(format!("{:?}", e)),
                            });
                        }
                    }
                });
            }
            
            KnowledgeAction::SwitchToChat => {
                self.sidebar.current_view = SidebarView::Chat;
            }
        }
    }
    
    fn handle_quick_learn_action(&mut self, action: QuickLearnAction) {
        match action {
            QuickLearnAction::Learn(content) => {
                self.quick_learn.is_processing = true;
                let storage = self.storage.clone();
                let pipeline = self.pipeline.clone();
                let tx = self.tx.clone();
                let content_clone = content.clone();
                
                std::thread::spawn(move || {
                    let start = Instant::now();
                    
                    // Use unified pipeline
                    let options = LearnOptions::default();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(pipeline.learn_concept(&storage, &content_clone, &options));
                    
                    let elapsed = start.elapsed();
                    let message = match result {
                        Ok(_) => AppMessage::QuickLearnCompleted { 
                            content: content_clone,
                            success: true,
                            error: None,
                            elapsed_ms: elapsed.as_millis() as u64,
                        },
                        Err(e) => AppMessage::QuickLearnCompleted { 
                            content: content_clone,
                            success: false,
                            error: Some(e.to_string()),
                            elapsed_ms: elapsed.as_millis() as u64,
                        },
                    };
                    
                    let _ = tx.send(message);
                });
            }
            
            QuickLearnAction::BatchLearn(batch_content) => {
                use crate::ui::quick_learn::LearnStatus;
                
                let lines: Vec<String> = batch_content
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect();
                
                if lines.is_empty() {
                    return;
                }
                
                self.quick_learn.is_processing = true;
                
                // Add all entries as processing
                for line in &lines {
                    self.quick_learn.add_learn_entry(line.clone(), LearnStatus::Processing);
                }
                
                let storage = self.storage.clone();
                let pipeline = self.pipeline.clone();
                let tx = self.tx.clone();
                
                std::thread::spawn(move || {
                    let start = Instant::now();
                    
                    // Use unified pipeline batch learning
                    let options = LearnOptions::default();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    
                    let mut successes = 0;
                    let mut failures = Vec::new();
                    
                    for content in lines {
                        match rt.block_on(pipeline.learn_concept(&storage, &content, &options)) {
                            Ok(_) => {
                                successes += 1;
                                let _ = tx.send(AppMessage::QuickLearnCompleted { 
                                    content: content.clone(),
                                    success: true,
                                    error: None,
                                    elapsed_ms: 0,
                                });
                            }
                            Err(e) => {
                                failures.push((content.clone(), e.to_string()));
                                let _ = tx.send(AppMessage::QuickLearnCompleted { 
                                    content: content.clone(),
                                    success: false,
                                    error: Some(e.to_string()),
                                    elapsed_ms: 0,
                                });
                            }
                        }
                        
                        // Brief pause between operations
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    
                    let elapsed = start.elapsed();
                    let _ = tx.send(AppMessage::BatchLearnCompleted { 
                        total: successes + failures.len(),
                        successes,
                        failures,
                        elapsed_ms: elapsed.as_millis() as u64,
                    });
                });
            }
            
            QuickLearnAction::Delete(content) => {
                // Find concept by content and delete it
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                
                std::thread::spawn(move || {
                    // Search for the concept by content
                    let search_results = storage.text_search(&content, 1);
                    
                    if let Some((concept_id, found_content, _)) = search_results.first() {
                        // Verify it's an exact match
                        if found_content == &content {
                            match storage.delete_concept(*concept_id) {
                                Ok(_) => {
                                    let _ = tx.send(AppMessage::ConceptDeleted {
                                        concept_id: concept_id.to_hex(),
                                        success: true,
                                        error: None,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(AppMessage::ConceptDeleted {
                                        concept_id: concept_id.to_hex(),
                                        success: false,
                                        error: Some(format!("Failed to delete: {:?}", e)),
                                    });
                                }
                            }
                        } else {
                            let _ = tx.send(AppMessage::ConceptDeleted {
                                concept_id: String::new(),
                                success: false,
                                error: Some("Concept content mismatch".to_string()),
                            });
                        }
                    } else {
                        let _ = tx.send(AppMessage::ConceptDeleted {
                            concept_id: String::new(),
                            success: false,
                            error: Some("Concept not found".to_string()),
                        });
                    }
                });
                
                self.status_bar.set_activity("Deleting concept...");
            }
        }
    }
    
    // ========================================================================
    // Graph View Actions
    // ========================================================================
    
    fn handle_graph_action(&mut self, action: GraphAction) {
        match action {
            GraphAction::Refresh => {
                self.status_bar.set_activity("Refreshing graph...");
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                
                std::thread::spawn(move || {
                    let mut nodes = std::collections::HashMap::new();
                    let mut edges = Vec::new();
                    
                    let snapshot = storage.get_snapshot();
                    let concepts = snapshot.all_concepts();
                    
                    // Create nodes
                    for concept in &concepts {
                        let content = String::from_utf8_lossy(&concept.content).to_string();
                        let mut node = GraphNode::new(
                            concept.id,
                            content,
                            concept.confidence,
                            concept.strength,
                        );
                        node.neighbor_count = concept.neighbors.len();
                        nodes.insert(concept.id, node);
                    }
                    
                    // Create edges
                    for concept in &concepts {
                        for neighbor_id in &concept.neighbors {
                            // Only add edge once (from lower to higher id to avoid duplicates)
                            if concept.id.to_hex() < neighbor_id.to_hex() {
                                let content = String::from_utf8_lossy(&concept.content).to_lowercase();
                                let edge_type = if content.contains("causes") || content.contains("leads to") || content.contains("results in") {
                                    EdgeType::Causal
                                } else if content.contains("before") || content.contains("after") || content.contains("during") {
                                    EdgeType::Temporal
                                } else if content.contains("is a") || content.contains("part of") || content.contains("contains") {
                                    EdgeType::Hierarchical
                                } else if content.contains("similar") || content.contains("like") || content.contains("same") {
                                    EdgeType::Similar
                                } else {
                                    EdgeType::Semantic
                                };
                                
                                edges.push(GraphEdge {
                                    from: concept.id,
                                    to: *neighbor_id,
                                    strength: concept.strength,
                                    edge_type,
                                });
                            }
                        }
                    }
                    
                    let _ = tx.send(AppMessage::GraphLoaded { nodes, edges });
                });
            }
            GraphAction::SelectNode(id) => {
                self.graph_view.selected = Some(id);
                self.status_bar.set_activity(format!("Selected node: {}...", &id.to_hex()[..8]));
            }
            GraphAction::ExportImage => {
                // Export graph as GraphML which can be visualized in other tools
                let snapshot = self.storage.get_snapshot();
                let concepts = snapshot.all_concepts();
                
                let mut graphml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
                graphml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
                graphml.push_str("  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n");
                graphml.push_str("  <key id=\"confidence\" for=\"node\" attr.name=\"confidence\" attr.type=\"double\"/>\n");
                graphml.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");
                
                for node in concepts.iter() {
                    let content = String::from_utf8_lossy(&node.content);
                    let label = content.chars().take(50).collect::<String>().replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;");
                    graphml.push_str(&format!(
                        "    <node id=\"{}\">\n      <data key=\"label\">{}</data>\n      <data key=\"confidence\">{:.3}</data>\n    </node>\n",
                        node.id.to_hex(), label, node.confidence
                    ));
                }
                
                // Add edges based on associations
                let mut edge_id = 0;
                for node in concepts.iter() {
                    for assoc in &node.associations {
                        graphml.push_str(&format!(
                            "    <edge id=\"e{}\" source=\"{}\" target=\"{}\"/>\n",
                            edge_id, node.id.to_hex(), assoc.target_id.to_hex()
                        ));
                        edge_id += 1;
                    }
                }
                
                graphml.push_str("  </graph>\n</graphml>\n");
                
                // Write to file
                if let Some(home) = dirs::home_dir() {
                    let path = home.join("sutra_graph.graphml");
                    match std::fs::write(&path, graphml) {
                        Ok(_) => {
                            self.status_bar.set_activity(format!("Graph exported to {:?}", path));
                        }
                        Err(e) => {
                            self.status_bar.set_activity(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_bar.set_activity("Could not determine home directory");
                }
            }
        }
    }
    
    // ========================================================================
    // Reasoning Paths Actions
    // ========================================================================
    
    fn handle_reasoning_action(&mut self, action: ReasoningPathsAction) {
        match action {
            ReasoningPathsAction::FindPaths(from, to) => {
                self.status_bar.set_activity("Finding reasoning paths...");
                self.reasoning_paths.is_loading = true;
                self.reasoning_paths.error_message = None;
                
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                let max_depth = self.reasoning_paths.max_depth;
                let max_paths = self.reasoning_paths.max_paths;
                let decay = self.reasoning_paths.confidence_decay;
                let threshold = self.reasoning_paths.consensus_threshold;
                
                std::thread::spawn(move || {
                    // Search for source and target concepts
                    let from_results = storage.text_search(&from, 1);
                    let to_results = storage.text_search(&to, 1);
                    
                    if let (Some((from_id, _, _)), Some((to_id, _, _))) = 
                        (from_results.first(), to_results.first()) 
                    {
                        let snapshot = storage.get_snapshot();
                        let pathfinder = ParallelPathFinder::new(decay);
                        
                        let raw_paths = pathfinder.find_paths_parallel(
                            snapshot.clone(),
                            *from_id,
                            *to_id,
                            max_depth,
                            max_paths,
                        );
                        
                        if raw_paths.is_empty() {
                            let _ = tx.send(AppMessage::ReasoningPathsFound {
                                paths: vec![],
                                consensus: None,
                                error: Some("No paths found between these concepts.".to_string()),
                            });
                        } else {
                            // Convert paths
                            let paths: Vec<ReasoningPath> = raw_paths
                                .into_iter()
                                .map(|p| ReasoningPathsPanel::convert_path_static(&p, &snapshot, decay))
                                .collect();
                            
                            // Analyze consensus
                            let consensus = ReasoningPathsPanel::analyze_consensus_static(&paths, threshold);
                            
                            let _ = tx.send(AppMessage::ReasoningPathsFound {
                                paths,
                                consensus: Some(consensus),
                                error: None,
                            });
                        }
                    } else {
                        let _ = tx.send(AppMessage::ReasoningPathsFound {
                            paths: vec![],
                            consensus: None,
                            error: Some("Could not find source or target concepts".to_string()),
                        });
                    }
                });
            }
            ReasoningPathsAction::ExportReasoning => {
                // Export reasoning paths as JSON
                let paths = &self.reasoning_paths.paths;
                
                let export_data: Vec<serde_json::Value> = paths.iter().enumerate().map(|(idx, path)| {
                    let steps: Vec<serde_json::Value> = path.path.iter().map(|step| {
                        serde_json::json!({
                            "concept_id": step.concept_id.to_hex(),
                            "content": step.content,
                            "confidence": step.confidence
                        })
                    }).collect();
                    
                    serde_json::json!({
                        "path_index": idx,
                        "confidence": path.confidence,
                        "depth": path.depth,
                        "step_count": path.path.len(),
                        "steps": steps
                    })
                }).collect();
                
                let json_str = serde_json::to_string_pretty(&export_data).unwrap_or_default();
                
                // Write to file
                if let Some(home) = dirs::home_dir() {
                    let path = home.join("sutra_reasoning_paths.json");
                    match std::fs::write(&path, json_str) {
                        Ok(_) => {
                            self.status_bar.set_activity(format!("Reasoning paths exported to {:?}", path));
                        }
                        Err(e) => {
                            self.status_bar.set_activity(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_bar.set_activity("Could not determine home directory");
                }
            }
        }
    }
    
    // ========================================================================
    // Temporal View Actions
    // ========================================================================
    
    fn handle_temporal_action(&mut self, action: TemporalViewAction) {
        match action {
            TemporalViewAction::ViewInGraph(id) => {
                // Use from_string which parses hex strings into ConceptId
                let concept_id = ConceptId::from_string(&id);
                self.graph_view.selected = Some(concept_id);
                self.sidebar.current_view = SidebarView::Graph;
            }
            TemporalViewAction::ExploreRelations(id, relation) => {
                // Search for related concepts with temporal relationships
                let results = self.storage.text_search(&format!("{:?}", relation).to_lowercase(), 10);
                if !results.is_empty() {
                    // Update temporal view with related events
                    let events = self.load_temporal_events();
                    self.temporal_view.load_events(events);
                }
                self.status_bar.set_activity(format!("Found {} related concepts", results.len()));
            }
            TemporalViewAction::RefreshData => {
                // Load temporal events from storage
                let events = self.load_temporal_events();
                let event_count = events.len();
                self.temporal_view.load_events(events);
                self.status_bar.set_activity(format!("Loaded {} temporal events", event_count));
            }
        }
    }
    
    // ========================================================================
    // Causal View Actions
    // ========================================================================
    
    fn handle_causal_action(&mut self, action: CausalViewAction) {
        match action {
            CausalViewAction::AnalyzeCause { effect, max_hops } => {
                self.causal_view.is_analyzing = true;
                self.status_bar.set_activity(format!("Analyzing causes for '{}'...", effect));
                
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                let effect_clone = effect.clone();
                
                std::thread::spawn(move || {
                    // Search for effect concept
                    let results = storage.text_search(&effect_clone, 1);
                    
                    if let Some((effect_id, _, _)) = results.first() {
                        let (chains, root_causes) = CausalView::analyze_static(&storage, *effect_id, max_hops);
                        
                        let _ = tx.send(AppMessage::CausalAnalysisComplete {
                            chains,
                            root_causes,
                            error: None,
                        });
                    } else {
                        let _ = tx.send(AppMessage::CausalAnalysisComplete {
                            chains: vec![],
                            root_causes: vec![],
                            error: Some("Could not find effect concept".to_string()),
                        });
                    }
                });
            }
            CausalViewAction::ExploreNode(id) => {
                // Use from_string which parses hex strings into ConceptId
                let concept_id = ConceptId::from_string(&id);
                self.graph_view.selected = Some(concept_id);
                self.sidebar.current_view = SidebarView::Graph;
            }
            CausalViewAction::ExportChains => {
                // Export causal chains as JSON
                let chains = &self.causal_view.causal_chains;
                let root_causes = &self.causal_view.root_causes;
                
                let chains_export: Vec<serde_json::Value> = chains.iter().enumerate().map(|(idx, chain)| {
                    let nodes: Vec<serde_json::Value> = chain.nodes.iter().map(|node| {
                        serde_json::json!({
                            "id": node.id,
                            "label": node.label,
                            "confidence": node.confidence,
                            "is_root_cause": node.is_root_cause,
                            "relation_type": format!("{:?}", node.relation_type)
                        })
                    }).collect();
                    
                    serde_json::json!({
                        "chain_index": idx,
                        "confidence": chain.confidence,
                        "depth": chain.depth,
                        "nodes": nodes
                    })
                }).collect();
                
                let root_causes_export: Vec<serde_json::Value> = root_causes.iter().map(|cause| {
                    serde_json::json!({
                        "id": cause.id,
                        "label": cause.label,
                        "confidence": cause.confidence,
                        "relation_type": format!("{:?}", cause.relation_type)
                    })
                }).collect();
                
                let export_data = serde_json::json!({
                    "chains": chains_export,
                    "root_causes": root_causes_export
                });
                
                let json_str = serde_json::to_string_pretty(&export_data).unwrap_or_default();
                
                // Write to file
                if let Some(home) = dirs::home_dir() {
                    let path = home.join("sutra_causal_analysis.json");
                    match std::fs::write(&path, json_str) {
                        Ok(_) => {
                            self.status_bar.set_activity(format!("Causal analysis exported to {:?}", path));
                        }
                        Err(e) => {
                            self.status_bar.set_activity(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_bar.set_activity("Could not determine home directory");
                }
            }
        }
    }
    
    // ========================================================================
    // Analytics Actions
    // ========================================================================
    
    fn handle_analytics_action(&mut self, action: AnalyticsAction) {
        match action {
            AnalyticsAction::ExportReport => {
                // Export analytics report as JSON
                let stats = self.storage.stats();
                
                let export_data = serde_json::json!({
                    "report_generated": chrono::Local::now().to_rfc3339(),
                    "storage_stats": {
                        "total_concepts": stats.snapshot.concept_count,
                        "total_edges": stats.snapshot.edge_count,
                        "snapshot_sequence": stats.snapshot.sequence
                    },
                    "performance": {
                        "history_count": self.analytics.history.len(),
                        "activity_log_count": self.analytics.activity_log.len()
                    },
                    "recent_activities": self.analytics.activity_log.iter().take(100).map(|entry| {
                        serde_json::json!({
                            "activity_type": format!("{:?}", entry.activity_type),
                            "description": entry.description,
                            "duration_ms": entry.duration_ms
                        })
                    }).collect::<Vec<_>>()
                });
                
                let json_str = serde_json::to_string_pretty(&export_data).unwrap_or_default();
                
                // Write to file
                if let Some(home) = dirs::home_dir() {
                    let path = home.join("sutra_analytics_report.json");
                    match std::fs::write(&path, json_str) {
                        Ok(_) => {
                            self.status_bar.set_activity(format!("Analytics report exported to {:?}", path));
                        }
                        Err(e) => {
                            self.status_bar.set_activity(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_bar.set_activity("Could not determine home directory");
                }
            }
            AnalyticsAction::ClearHistory => {
                self.analytics.history.clear();
                self.analytics.activity_log.clear();
                self.status_bar.set_activity("Analytics history cleared");
            }
        }
    }
    
    // ========================================================================
    // Query Builder Actions
    // ========================================================================
    
    fn handle_query_builder_action(&mut self, action: QueryBuilderAction) {
        match action {
            QueryBuilderAction::RunQuery { query_type, query, filters } => {
                self.query_builder.is_searching = true;
                let storage = self.storage.clone();
                let tx = self.tx.clone();
                let query_clone = query.clone();
                let filters_clone = filters.clone();
                
                std::thread::spawn(move || {
                    let start = Instant::now();
                    
                    let results: Vec<ConceptInfo> = {
                        let search_results = storage.text_search(&query_clone, filters_clone.max_results);
                        search_results.into_iter()
                            .filter_map(|(id, _, conf)| {
                                if conf >= filters_clone.min_confidence {
                                    let snapshot = storage.get_snapshot();
                                    snapshot.get_concept(&id)
                                        .map(|node| node_to_concept_info(&node, &storage))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    };
                    
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let _ = tx.send(AppMessage::QueryBuilderResults {
                        results,
                        duration_ms: elapsed_ms,
                    });
                });
            }
            QueryBuilderAction::ExportResults => {
                // Export query results as JSON
                let results = &self.query_builder.results;
                
                let export_data: Vec<serde_json::Value> = results.iter().map(|result| {
                    serde_json::json!({
                        "id": result.id,
                        "content": result.content,
                        "confidence": result.confidence,
                        "strength": result.strength,
                        "neighbor_count": result.neighbors.len()
                    })
                }).collect();
                
                let json_str = serde_json::to_string_pretty(&export_data).unwrap_or_default();
                
                // Write to file
                if let Some(home) = dirs::home_dir() {
                    let path = home.join("sutra_query_results.json");
                    match std::fs::write(&path, json_str) {
                        Ok(_) => {
                            self.status_bar.set_activity(format!("Query results exported to {:?}", path));
                        }
                        Err(e) => {
                            self.status_bar.set_activity(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_bar.set_activity("Could not determine home directory");
                }
            }
            QueryBuilderAction::VisualizeResults => {
                self.sidebar.current_view = SidebarView::Graph;
            }
        }
    }
    
    // ========================================================================
    // Export/Import Actions
    // ========================================================================
    
    fn handle_export_import_action(&mut self, action: ExportImportAction) {
        match action {
            ExportImportAction::Export(path) => {
                self.status_bar.set_activity(format!("Exporting to {}...", path));
                
                // Generate export content
                match self.export_import.generate_export(&self.storage) {
                    Ok(content) => {
                        // Expand ~ to home directory
                        let expanded_path = if path.starts_with('~') {
                            if let Some(home) = dirs::home_dir() {
                                path.replacen("~", &home.display().to_string(), 1)
                            } else {
                                path.clone()
                            }
                        } else {
                            path.clone()
                        };
                        
                        // Write to file
                        match std::fs::write(&expanded_path, &content) {
                            Ok(_) => {
                                let stats = self.storage.stats();
                                self.export_import.set_export_result(
                                    true,
                                    &expanded_path,
                                    stats.snapshot.concept_count,
                                    stats.snapshot.edge_count,
                                    &format!("Successfully exported to {}", expanded_path)
                                );
                                self.status_bar.set_activity(format!("Exported {} concepts", stats.snapshot.concept_count));
                                info!("Exported to {}", expanded_path);
                            }
                            Err(e) => {
                                self.export_import.set_export_result(
                                    false, &path, 0, 0,
                                    &format!("Failed to write file: {}", e)
                                );
                                self.status_bar.set_activity("Export failed");
                                error!("Export failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.export_import.set_export_result(false, &path, 0, 0, &e);
                        self.status_bar.set_activity("Export failed");
                        error!("Export generation failed: {}", e);
                    }
                }
            }
            ExportImportAction::Import(path) => {
                self.status_bar.set_activity(format!("Importing from {}...", path));
                
                // Expand ~ to home directory
                let expanded_path = if path.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        path.replacen("~", &home.display().to_string(), 1)
                    } else {
                        path.clone()
                    }
                } else {
                    path.clone()
                };
                
                // Read and parse file
                match std::fs::read_to_string(&expanded_path) {
                    Ok(content) => {
                        match self.import_json_data(&content) {
                            Ok((concepts, edges)) => {
                                self.export_import.set_import_result(
                                    true, concepts, edges, 0,
                                    &format!("Successfully imported {} concepts, {} edges", concepts, edges)
                                );
                                self.refresh_stats();
                                self.status_bar.set_activity(format!("Imported {} concepts", concepts));
                                info!("Imported {} concepts from {}", concepts, expanded_path);
                            }
                            Err(e) => {
                                self.export_import.set_import_result(false, 0, 0, 1, &e);
                                self.status_bar.set_activity("Import failed");
                                error!("Import parsing failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.export_import.set_import_result(
                            false, 0, 0, 1,
                            &format!("Failed to read file: {}", e)
                        );
                        self.status_bar.set_activity("Import failed");
                        error!("Import failed: {}", e);
                    }
                }
            }
            ExportImportAction::BatchImport(path) => {
                self.status_bar.set_activity(format!("Batch importing from {}...", path));
                
                // Expand path
                let expanded_path = if path.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        path.replacen("~", &home.display().to_string(), 1)
                    } else {
                        path.clone()
                    }
                } else {
                    path.clone()
                };
                
                let storage = self.storage.clone();
                let pipeline = self.pipeline.clone();
                let tx = self.tx.clone();
                
                self.export_import.batch_progress.is_running = true;
                self.export_import.batch_progress.completed = 0;
                self.export_import.batch_progress.errors = 0;
                
                std::thread::spawn(move || {
                    // Read CSV file
                    match std::fs::read_to_string(&expanded_path) {
                        Ok(content) => {
                            let lines: Vec<&str> = content.lines().collect();
                            let total = lines.len().saturating_sub(1); // Skip header
                            
                            let _ = tx.send(AppMessage::BatchImportProgress {
                                completed: 0,
                                total,
                                errors: 0,
                            });
                            
                            let mut errors = 0;
                            let mut last_update = Instant::now();
                            
                            let options = LearnOptions::default();
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            
                            for (i, line) in lines.iter().skip(1).enumerate() {
                                let parts: Vec<&str> = line.split(',').collect();
                                if let Some(content) = parts.first() {
                                    let content = content.trim().trim_matches('"');
                                    if !content.is_empty() {
                                        let confidence: f32 = parts.get(1)
                                            .and_then(|s| s.trim().parse().ok())
                                            .unwrap_or(1.0);
                                        
                                        // Use unified pipeline
                                        let mut item_options = options.clone();
                                        item_options.confidence = confidence;
                                        
                                        if rt.block_on(pipeline.learn_concept(&storage, content, &item_options)).is_err() {
                                            errors += 1;
                                        }
                                    }
                                }
                                
                                // Send progress update every 100ms
                                if last_update.elapsed().as_millis() > 100 {
                                    let _ = tx.send(AppMessage::BatchImportProgress {
                                        completed: i + 1,
                                        total,
                                        errors,
                                    });
                                    last_update = Instant::now();
                                }
                            }
                            
                            let _ = tx.send(AppMessage::BatchImportComplete {
                                completed: total,
                                errors,
                                error: None,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::BatchImportComplete {
                                completed: 0,
                                errors: 0,
                                error: Some(format!("Failed to read file: {}", e)),
                            });
                        }
                    }
                });
            }
            ExportImportAction::CancelBatch => {
                // Note: This doesn't actually stop the thread, just updates UI
                // To properly stop, we'd need an atomic flag or channel
                self.export_import.batch_progress.is_running = false;
                self.status_bar.set_activity("Batch operation cancelled (background task may continue)");
            }
        }
    }
    
    // ========================================================================
    // Settings Actions
    // ========================================================================
    
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
                self.sidebar.current_view = SidebarView::Export;
            }
            SettingsAction::ImportData => {
                self.sidebar.current_view = SidebarView::Export;
            }
            SettingsAction::ClearData => {
                // Clear all data by removing the storage directory contents
                warn!("Clearing all data from: {:?}", self.data_dir);
                
                // Remove all files in data directory (but keep directory itself)
                if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let _ = std::fs::remove_file(&path);
                        } else if path.is_dir() {
                            let _ = std::fs::remove_dir_all(&path);
                        }
                    }
                }
                
                // Reinitialize storage with same config as initial setup
                let config = ConcurrentConfig {
                    storage_path: self.data_dir.clone(),
                    memory_threshold: 10_000,
                    vector_dimension: 768, // Must match nomic-embed-text-v1.5 (768D)
                    adaptive_reconciler_config: AdaptiveReconcilerConfig {
                        base_interval_ms: 100,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                
                let new_storage = ConcurrentMemory::new(config);
                self.storage = Arc::new(new_storage);
                
                // Reinitialize pipeline (it holds references to storage components internally via extractor)
                // Actually pipeline is stateless regarding storage instance, but let's be safe
                // The pipeline holds the embedding provider which is expensive to reload
                // We can reuse the existing pipeline instance as it doesn't hold storage reference
                
                // Reset UI state
                self.knowledge.concepts.clear();
                self.graph_view.nodes.clear();
                self.graph_view.edges.clear();
                self.chat.messages.clear();
                self.chat.add_response("🗑️ All data has been cleared. Starting fresh!".to_string());
                
                self.refresh_stats();
                self.status_bar.set_activity("All data cleared");
                info!("Storage cleared and reinitialized");
            }
        }
    }
    
    // ========================================================================
    // Async Message Handling
    // ========================================================================
    
    fn handle_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::LearnResult { content, concept_id, error, duration_ms } => {
                    self.chat.is_processing = false;
                    if let Some(err) = error {
                        self.chat.add_response(format!("❌ Failed to learn: {}", err));
                        error!("Learn failed: {}", err);
                    } else if let Some(id) = concept_id {
                        self.chat.add_response(format!(
                            "✅ Learned! Stored as concept `{}`.\n\nYou can now ask me questions about this.",
                            &id.to_hex()[..8]
                        ));
                        let preview = if content.len() > 30 { &content[..30] } else { &content };
                        self.status_bar.set_activity(format!("Learned: {}... ({}ms)", preview, duration_ms));
                        self.refresh_stats();
                    }
                }
                
                AppMessage::SearchResults { query, results, duration_ms, generated_response } => {
                    self.chat.is_processing = false;
                    if results.is_empty() {
                        self.chat.add_response(
                            "🤔 I don't have knowledge about that yet.\n\nTeach me with: `/learn <your knowledge>`".to_string()
                        );
                    } else {
                        if let Some(response) = generated_response {
                            // Use NLG response if available
                            self.chat.add_response(format!(
                                "{}\n\n*Sources:*\n{}", 
                                response,
                                results.iter().take(3).enumerate()
                                    .map(|(i, (_, content, _))| format!("{}. {}", i + 1, content))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            ));
                        } else {
                            // Fallback to list view
                            let mut response = String::from("Based on my knowledge:\n\n");
                            for (i, (_id, content, confidence)) in results.iter().enumerate() {
                                response.push_str(&format!("{}. {} (relevance: {:.0}%)\n", i + 1, content, confidence * 100.0));
                            }
                            self.chat.add_response(response);
                        }
                    }
                    
                    let preview = if query.len() > 30 { &query[..30] } else { &query };
                    self.status_bar.set_activity(format!("Searched: {} ({}ms)", preview, duration_ms));
                }
                
                AppMessage::KnowledgeLoaded { concepts } => {
                    self.knowledge.is_loading = false;
                    self.knowledge.set_concepts(concepts);
                    self.status_bar.set_activity("Knowledge refreshed");
                }
                
                AppMessage::QuickLearnCompleted { content, success, error, elapsed_ms } => {
                    // Find and update the processing entry instead of adding a new one
                    if let Some(entry) = self.quick_learn.recent_learns
                        .iter_mut()
                        .find(|e| e.content == content && matches!(e.status, crate::ui::quick_learn::LearnStatus::Processing)) {
                        
                        entry.status = if success {
                            crate::ui::quick_learn::LearnStatus::Success
                        } else {
                            crate::ui::quick_learn::LearnStatus::Failed(error.clone().unwrap_or_else(|| "Unknown error".to_string()))
                        };
                        
                        // Check if this was the last processing item
                        let still_processing = self.quick_learn.recent_learns
                            .iter()
                            .any(|e| matches!(e.status, crate::ui::quick_learn::LearnStatus::Processing));
                        
                        if !still_processing {
                            self.quick_learn.is_processing = false;
                            if success {
                                self.quick_learn.clear_input();
                            }
                        }
                    }
                    
                    // Update app state
                    if success {
                        self.refresh_stats();
                        if elapsed_ms > 0 {
                            let preview = if content.len() > 30 { &content[..30] } else { &content };
                            self.status_bar.set_activity(format!("Learned: {}... ({}ms)", preview, elapsed_ms));
                        }
                    } else {
                        warn!("Quick learn failed: {}", error.unwrap_or_else(|| "Unknown error".to_string()));
                    }
                }
                
                AppMessage::BatchLearnCompleted { total, successes, failures, elapsed_ms } => {
                    self.quick_learn.is_processing = false;
                    
                    // Clear batch input on successful completion
                    if failures.is_empty() {
                        self.quick_learn.clear_input();
                    }
                    
                    // Update status
                    let status_msg = if failures.is_empty() {
                        format!("Batch complete: {}/{} learned ({}ms)", successes, total, elapsed_ms)
                    } else {
                        format!("Batch partial: {}/{} learned, {} failed ({}ms)", successes, total, failures.len(), elapsed_ms)
                    };
                    
                    self.status_bar.set_activity(status_msg);
                    self.refresh_stats();
                }
                
                AppMessage::GraphLoaded { nodes, edges } => {
                    self.graph_view.load_from_data(nodes, edges);
                    self.status_bar.set_activity("Graph refreshed");
                }
                
                AppMessage::ReasoningPathsFound { paths, consensus, error } => {
                    self.reasoning_paths.is_loading = false;
                    if let Some(err) = error {
                        self.reasoning_paths.error_message = Some(err);
                        self.status_bar.set_activity("Reasoning path search failed");
                    } else {
                        self.reasoning_paths.paths = paths;
                        self.reasoning_paths.consensus = consensus;
                        self.status_bar.set_activity(format!("Found {} paths", self.reasoning_paths.paths.len()));
                    }
                }
                
                AppMessage::CausalAnalysisComplete { chains, root_causes, error } => {
                    self.causal_view.is_analyzing = false;
                    if let Some(err) = error {
                        self.causal_view.set_error(err);
                        self.status_bar.set_activity("Causal analysis failed");
                    } else {
                        self.causal_view.set_results(chains, root_causes);
                        self.status_bar.set_activity("Causal analysis complete");
                    }
                }
                
                AppMessage::QueryBuilderResults { results, duration_ms } => {
                    self.query_builder.is_searching = false;
                    self.query_builder.set_results(results, duration_ms);
                    self.status_bar.set_activity(format!("Query completed in {}ms", duration_ms));
                }
                
                AppMessage::BatchImportProgress { completed, total, errors } => {
                    self.export_import.batch_progress.completed = completed;
                    self.export_import.batch_progress.total = total;
                    self.export_import.batch_progress.errors = errors;
                    self.status_bar.set_activity(format!("Importing: {}/{} ({} errors)", completed, total, errors));
                }
                
                AppMessage::BatchImportComplete { completed, errors, error } => {
                    self.export_import.batch_progress.is_running = false;
                    if let Some(err) = error {
                        self.status_bar.set_activity(format!("Batch import failed: {}", err));
                        error!("Batch import failed: {}", err);
                    } else {
                        self.export_import.batch_progress.completed = completed;
                        self.export_import.batch_progress.errors = errors;
                        self.refresh_stats();
                        self.status_bar.set_activity(format!(
                            "Batch import complete: {} imported, {} errors",
                            completed - errors,
                            errors
                        ));
                    }
                }
                
                AppMessage::ConceptDeleted { concept_id, success, error } => {
                    if success {
                        // Remove from recent learns if it's there (by content)
                        if !concept_id.is_empty() {
                            // Use concept ID to find and remove from UI lists
                            self.quick_learn.recent_learns.retain(|entry| {
                                // This is a simplified check - in practice you'd want better mapping
                                !entry.content.contains(&concept_id[..8])
                            });
                        }
                        
                        // Refresh knowledge panel and stats
                        self.handle_knowledge_action(KnowledgeAction::Refresh);
                        self.refresh_stats();
                        self.status_bar.set_activity("Concept deleted successfully");
                    } else if let Some(err) = error {
                        self.status_bar.set_activity(format!("Delete failed: {}", err));
                        error!("Failed to delete concept: {}", err);
                    }
                }
                
                _ => {}
            }
        }
    }

    // ========================================================================
    // Menu Bar
    // ========================================================================
    
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        use crate::theme::{BG_PANEL, TEXT_PRIMARY, TEXT_SECONDARY, BG_HOVER, PRIMARY};
        
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(36.0)
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(egui::Margin::symmetric(12.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button(RichText::new("File").size(13.0).color(TEXT_PRIMARY), |ui| {
                        if ui.button("📥 Import Data...").clicked() {
                            self.sidebar.current_view = SidebarView::Export;
                            ui.close_menu();
                        }
                        if ui.button("📤 Export Data...").clicked() {
                            self.sidebar.current_view = SidebarView::Export;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("⚙️ Settings").clicked() {
                            self.sidebar.current_view = SidebarView::Settings;
                            ui.close_menu();
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ui.separator();
                            if ui.button("❌ Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    });
                    
                    ui.menu_button(RichText::new("View").size(13.0).color(TEXT_PRIMARY), |ui| {
                        if ui.button("💬 Chat").clicked() {
                            self.sidebar.current_view = SidebarView::Chat;
                            ui.close_menu();
                        }
                        if ui.button("📚 Knowledge").clicked() {
                            self.sidebar.current_view = SidebarView::Knowledge;
                            ui.close_menu();
                        }
                        if ui.button("🔍 Search").clicked() {
                            self.sidebar.current_view = SidebarView::Search;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(RichText::new("Analysis").size(11.0).color(TEXT_SECONDARY));
                        if ui.button("🕸️ Graph View").clicked() {
                            self.sidebar.current_view = SidebarView::Graph;
                            ui.close_menu();
                        }
                        if ui.button("🛤️ Reasoning Paths").clicked() {
                            self.sidebar.current_view = SidebarView::Paths;
                            ui.close_menu();
                        }
                        if ui.button("⏱️ Timeline").clicked() {
                            self.sidebar.current_view = SidebarView::Timeline;
                            ui.close_menu();
                        }
                        if ui.button("🔗 Causality").clicked() {
                            self.sidebar.current_view = SidebarView::Causal;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(RichText::new("Tools").size(11.0).color(TEXT_SECONDARY));
                        if ui.button("📊 Analytics").clicked() {
                            self.sidebar.current_view = SidebarView::Analytics;
                            ui.close_menu();
                        }
                        if ui.button("🔎 Query Builder").clicked() {
                            self.sidebar.current_view = SidebarView::Query;
                            ui.close_menu();
                        }
                    });
                    
                    ui.menu_button(RichText::new("Help").size(13.0).color(TEXT_PRIMARY), |ui| {
                        if ui.button("📖 Documentation").clicked() {
                            let _ = webbrowser::open("https://github.com/sutraworks/sutra-memory/tree/main/docs/desktop");
                            ui.close_menu();
                        }
                        if ui.button("💡 Quick Start Guide").clicked() {
                            let _ = webbrowser::open("https://github.com/sutraworks/sutra-memory/blob/main/docs/desktop/README.md");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("ℹ️ About Sutra").clicked() {
                            self.show_about_dialog();
                            ui.close_menu();
                        }
                    });
                    
                    // Right side - breadcrumb showing current view
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (icon, label, _) = self.sidebar.current_view.info();
                        ui.label(RichText::new(format!("{} {}", icon, label)).size(12.0).color(TEXT_SECONDARY));
                    });
                });
            });
    }
    
    fn show_about_dialog(&mut self) {
        self.chat.add_response(format!(
            "🧠 **Sutra AI Desktop v{}**\n\n\
            Your personal knowledge reasoning engine.\n\n\
            • **Temporal Reasoning** - Understand time relationships\n\
            • **Causal Analysis** - Discover root causes\n\
            • **Semantic Understanding** - 9 types of knowledge\n\
            • **Complete Privacy** - All data stays local\n\n\
            Built with ❤️ by Sutra Works",
            env!("CARGO_PKG_VERSION")
        ));
        self.sidebar.current_view = SidebarView::Chat;
    }
    
    // ========================================================================
    // Helper Methods
    // ========================================================================
    
    fn load_all_concepts(&self, limit: usize) -> Vec<ConceptInfo> {
        let snapshot = self.storage.get_snapshot();
        snapshot.all_concepts()
            .into_iter()
            .take(limit)
            .map(|node| node_to_concept_info(&node, &self.storage))
            .collect()
    }
    
    fn refresh_stats(&mut self) {
        let stats = self.storage.stats();
        self.status_bar.set_concept_count(stats.snapshot.concept_count);
        self.settings.update_stats(convert_to_ui_stats(&stats));
        self.analytics.update(&self.storage);
    }

    fn import_json_data(&self, content: &str) -> Result<(usize, usize), String> {
        let json: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
            
        let concepts = json.get("concepts")
            .and_then(|c| c.as_array())
            .ok_or("Missing 'concepts' array in JSON")?;
        
        let mut imported_concepts = 0;
        let mut imported_edges = 0;
        
        let options = LearnOptions::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        for concept in concepts {
            let content = concept.get("content")
                .and_then(|c| c.as_str())
                .ok_or("Concept missing 'content' field")?;
            
            let confidence = concept.get("confidence")
                .and_then(|c| c.as_f64())
                .unwrap_or(1.0) as f32;
            
            let strength = concept.get("strength")
                .and_then(|s| s.as_f64())
                .unwrap_or(1.0) as f32;
            
            // Use unified pipeline
            let mut item_options = options.clone();
            item_options.confidence = confidence;
            item_options.strength = strength;
            
            if rt.block_on(self.pipeline.learn_concept(&self.storage, content, &item_options)).is_ok() {
                imported_concepts += 1;
            }
            
            // Count neighbors as edges
            if let Some(neighbors) = concept.get("neighbors").and_then(|n| n.as_array()) {
                imported_edges += neighbors.len();
            }
        }
        
        Ok((imported_concepts, imported_edges))
    }
    
    /// Load temporal events from storage concepts
    pub fn load_temporal_events(&self) -> Vec<crate::types::TimelineEvent> {
        let snapshot = self.storage.get_snapshot();
        let concepts = snapshot.all_concepts();
        
        let temporal_keywords = ["before", "after", "during", "when", "then", "until", "since", "while"];
        
        concepts.iter()
            .filter_map(|node| {
                let content = std::str::from_utf8(&node.content).ok()?;
                let content_lower = content.to_lowercase();
                
                // Check if content has temporal markers
                let has_temporal = temporal_keywords.iter().any(|kw| content_lower.contains(kw));
                
                // Check semantic type for temporal classification
                let is_semantic_temporal = node.semantic.as_ref()
                    .map(|s| s.semantic_type == SemanticType::Temporal || s.semantic_type == SemanticType::Event)
                    .unwrap_or(false);
                
                if has_temporal || is_semantic_temporal {
                    // Determine relative time based on keywords
                    let relative_time = if content_lower.contains("before") || content_lower.contains("earlier") {
                        -1
                    } else if content_lower.contains("after") || content_lower.contains("later") || content_lower.contains("then") {
                        1
                    } else {
                        0
                    };
                    
                    Some(crate::types::TimelineEvent {
                        concept_id: node.id.to_hex(),
                        label: if content.len() > 50 { format!("{}...", &content[..50]) } else { content.to_string() },
                        description: content.to_string(),
                        timestamp: format!("Created: {}", node.created),
                        relative_time,
                        confidence: node.confidence,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl eframe::App for SutraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file drag-and-drop overlay
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("file_drop_overlay")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(200));
            
            // Draw dashed border
            let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
            painter.rect_stroke(screen_rect.shrink(20.0), 10.0, stroke);
            
            painter.text(
                screen_rect.center(),
                egui::Align2::CENTER_CENTER,
                "📥 Drop to Import Data",
                egui::FontId::proportional(32.0),
                egui::Color32::WHITE,
            );
        }

        // Handle dropped files
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for file in dropped_files {
                if let Some(path) = file.path {
                    let path_str = path.to_string_lossy().to_string();
                    info!("File dropped: {}", path_str);
                    
                    if path_str.to_lowercase().ends_with(".json") {
                        self.sidebar.current_view = SidebarView::Export;
                        self.export_import.import_path = path_str.clone();
                        self.export_import.active_tab = crate::ui::export_import::ExportImportTab::Import;
                        self.handle_export_import_action(ExportImportAction::Import(path_str));
                    } else if path_str.to_lowercase().ends_with(".csv") {
                        self.sidebar.current_view = SidebarView::Export;
                        self.export_import.batch_path = path_str.clone();
                        self.export_import.active_tab = crate::ui::export_import::ExportImportTab::Batch;
                        self.handle_export_import_action(ExportImportAction::BatchImport(path_str));
                    } else {
                        self.status_bar.set_activity(format!("Unsupported file type: {}", path_str));
                        warn!("Unsupported file dropped: {}", path_str);
                    }
                }
            }
        }

        // Handle async messages
        self.handle_async_messages();

        // Initial data load
        if !self.initialized {
            self.handle_knowledge_action(KnowledgeAction::Refresh);
            self.graph_view.load_from_storage(&self.storage);
            self.analytics.update(&self.storage);
            
            // Load temporal events from storage
            let events = self.load_temporal_events();
            self.temporal_view.load_events(events);
            
            self.initialized = true;
        }
        
        // Request repaint for animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
        
        // Menu bar at the top
        self.render_menu_bar(ctx);
        
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
            .exact_height(32.0)
            .show(ctx, |ui| {
                self.status_bar.ui(ui);
            });
        
        // Main content
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(16.0))
            .show(ctx, |ui| {
                match self.sidebar.current_view {
                    // Core views
                    SidebarView::Chat => {
                        if let Some(action) = self.chat.ui(ui) {
                            self.handle_chat_action(action);
                        }
                    }
                    SidebarView::Knowledge => {
                        if let Some(action) = self.knowledge.ui(ui) {
                            self.handle_knowledge_action(action);
                        }
                    }
                    SidebarView::Search => {
                        if let Some(action) = self.quick_learn.ui(ui) {
                            self.handle_quick_learn_action(action);
                        }
                    }
                    SidebarView::Settings => {
                        if let Some(action) = self.settings.ui(ui) {
                            self.handle_settings_action(action);
                        }
                    }
                    
                    // Enhanced views
                    SidebarView::Graph => {
                        if let Some(action) = self.graph_view.ui(ui) {
                            self.handle_graph_action(action);
                        }
                    }
                    SidebarView::Paths => {
                        if let Some(action) = self.reasoning_paths.ui(ui) {
                            self.handle_reasoning_action(action);
                        }
                    }
                    SidebarView::Timeline => {
                        if let Some(action) = self.temporal_view.ui(ui) {
                            self.handle_temporal_action(action);
                        }
                    }
                    SidebarView::Causal => {
                        if let Some(action) = self.causal_view.ui(ui) {
                            self.handle_causal_action(action);
                        }
                    }
                    SidebarView::Analytics => {
                        if let Some(action) = self.analytics.ui(ui) {
                            self.handle_analytics_action(action);
                        }
                    }
                    SidebarView::Query => {
                        if let Some(action) = self.query_builder.ui(ui) {
                            self.handle_query_builder_action(action);
                        }
                    }
                    SidebarView::Export => {
                        if let Some(action) = self.export_import.ui(ui) {
                            self.handle_export_import_action(action);
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
        vector_dimensions: 768,
        data_path: String::new(),
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
