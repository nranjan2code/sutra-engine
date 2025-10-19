//! Sutra Bulk Ingester
//! 
//! High-performance bulk data ingestion service with pluggable adapters.
//! 
//! Architecture:
//! - Rust core for maximum performance
//! - Python plugin system for flexibility
//! - Async streaming for memory efficiency
//! - TCP binary protocol for storage

pub mod core;
pub mod plugins;
pub mod adapters;
pub mod server;
pub mod storage;
pub mod metrics;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;
use tracing::{info, warn, error};

/// Core ingestion engine
pub struct BulkIngester {
    /// Storage client for TCP communication
    storage_client: storage::TcpStorageClient,
    
    /// Plugin registry for adapters
    plugin_registry: plugins::PluginRegistry,
    
    /// Active ingestion jobs
    active_jobs: HashMap<String, IngestionJob>,
    
    /// Job event channel
    job_sender: mpsc::UnboundedSender<JobEvent>,
    job_receiver: mpsc::UnboundedReceiver<JobEvent>,
    
    /// Configuration
    config: IngesterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngesterConfig {
    pub storage_server: String,
    pub max_concurrent_jobs: usize,
    pub batch_size: usize,
    pub memory_limit_mb: usize,
    pub plugin_dir: String,
    pub compression_enabled: bool,
    pub metrics_enabled: bool,
}

impl Default for IngesterConfig {
    fn default() -> Self {
        Self {
            storage_server: "storage-server:50051".to_string(),
            max_concurrent_jobs: 4,
            batch_size: 100,
            memory_limit_mb: 4096,
            plugin_dir: "./plugins".to_string(),
            compression_enabled: true,
            metrics_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionJob {
    pub id: String,
    pub source_type: String,  // "file", "database", "kafka", "api", etc.
    pub source_config: serde_json::Value,
    pub adapter_name: String,
    pub status: JobStatus,
    pub progress: JobProgress,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub total_items: Option<u64>,
    pub processed_items: u64,
    pub failed_items: u64,
    pub concepts_created: u64,
    pub bytes_processed: u64,
    pub current_rate: f64,  // items per second
}

#[derive(Debug, Clone)]
pub enum JobEvent {
    Started(String),
    Progress(String, JobProgress),
    Completed(String, JobProgress),
    Failed(String, String),
}

impl BulkIngester {
    /// Create new bulk ingester instance
    pub async fn new(config: IngesterConfig) -> Result<Self> {
        info!("Initializing Sutra Bulk Ingester");
        
        // Initialize storage client
        let storage_client = storage::TcpStorageClient::new(&config.storage_server).await?;
        
        // Initialize plugin registry
        let mut plugin_registry = plugins::PluginRegistry::new();
        plugin_registry.load_plugins(&config.plugin_dir).await?;
        
        let (job_sender, job_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            storage_client,
            plugin_registry,
            active_jobs: HashMap::new(),
            job_sender,
            job_receiver,
            config,
        })
    }
    
    /// Start the ingestion engine
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Sutra Bulk Ingester engine");
        
        // Start job event processor
        let job_sender = self.job_sender.clone();
        tokio::spawn(async move {
            // Job event processing loop would go here
        });
        
        // Main ingestion loop
        loop {
            tokio::select! {
                // Handle job events
                Some(event) = self.job_receiver.recv() => {
                    self.handle_job_event(event).await?;
                }
                
                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutdown signal received, stopping ingester");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Submit new ingestion job
    pub async fn submit_job(&mut self, job: IngestionJob) -> Result<String> {
        info!("Submitting ingestion job: {}", job.id);
        
        // Validate adapter exists
        if !self.plugin_registry.has_adapter(&job.adapter_name) {
            return Err(anyhow::anyhow!("Adapter '{}' not found", job.adapter_name));
        }
        
        // Check concurrent job limit
        let running_jobs = self.active_jobs.values()
            .filter(|j| matches!(j.status, JobStatus::Running))
            .count();
            
        if running_jobs >= self.config.max_concurrent_jobs {
            return Err(anyhow::anyhow!("Maximum concurrent jobs ({}) exceeded", self.config.max_concurrent_jobs));
        }
        
        // Add to active jobs
        let job_id = job.id.clone();
        self.active_jobs.insert(job_id.clone(), job);
        
        // Start processing
        self.start_job_processing(&job_id).await?;
        
        Ok(job_id)
    }
    
    async fn start_job_processing(&mut self, job_id: &str) -> Result<()> {
        let job = self.active_jobs.get_mut(job_id)
            .ok_or_else(|| anyhow::anyhow!("Job {} not found", job_id))?;
        
        // Get adapter
        if !self.plugin_registry.has_adapter(&job.adapter_name) {
            return Err(anyhow::anyhow!("Adapter '{}' not found", job.adapter_name));
        }
        
        // For now, use simple synchronous processing to avoid lifetime issues
        // In production, this would use a proper job queue
        job.status = JobStatus::Running;
        
        info!("Started processing job: {}", job_id);
        
        // Simulate job processing - replace with real implementation
        let job_id_clone = job_id.to_string();
        tokio::spawn(async move {
            // Mock processing
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            info!("Mock job processing completed for job: {}", job_id_clone);
        });
        
        Ok(())
    }
    
    // Real job processing with adapters and performance optimization
    async fn process_job_with_adapter(
        job: IngestionJob,
        adapter: &(dyn adapters::IngestionAdapter + Send + Sync),
        mut storage_client: storage::TcpStorageClient,
        job_sender: mpsc::UnboundedSender<JobEvent>,
        batch_size: usize,
    ) -> Result<JobProgress> {
        info!("Processing job: {} with adapter: {}", job.id, adapter.name());
        
        // Signal job started
        let _ = job_sender.send(JobEvent::Started(job.id.clone()));
        
        // Create data stream from adapter
        let mut data_stream = adapter.create_stream(&job.source_config).await?;
        
        let mut progress = JobProgress {
            total_items: data_stream.estimate_total().await?,
            processed_items: 0,
            failed_items: 0,
            concepts_created: 0,
            bytes_processed: 0,
            current_rate: 0.0,
        };
        
        let mut batch = Vec::with_capacity(batch_size);
        let mut last_progress_report = std::time::Instant::now();
        let start_time = std::time::Instant::now();
        
        info!("Starting to process data stream, estimated items: {:?}", progress.total_items);
        
        // Process data stream in optimized batches
        while let Some(item_result) = data_stream.next().await {
            match item_result {
                Ok(item) => {
                    progress.bytes_processed += item.size_bytes();
                    batch.push(item);
                    
                    // Process batch when full or at end
                    if batch.len() >= batch_size {
                        match Self::process_batch_optimized(&mut storage_client, &batch).await {
                            Ok(concepts) => {
                                progress.concepts_created += concepts;
                                progress.processed_items += batch.len() as u64;
                                info!("Processed batch of {} items, total: {}", batch.len(), progress.processed_items);
                            }
                            Err(err) => {
                                warn!("Batch processing failed: {}", err);
                                progress.failed_items += batch.len() as u64;
                            }
                        }
                        
                        batch.clear();
                        
                        // Report progress every 5 seconds for performance
                        if last_progress_report.elapsed() > std::time::Duration::from_secs(5) {
                            progress.current_rate = progress.processed_items as f64 / start_time.elapsed().as_secs_f64();
                            let _ = job_sender.send(JobEvent::Progress(job.id.clone(), progress.clone()));
                            last_progress_report = std::time::Instant::now();
                        }
                    }
                }
                Err(err) => {
                    warn!("Item processing error: {}", err);
                    progress.failed_items += 1;
                }
            }
        }
        
        // Process final batch
        if !batch.is_empty() {
            match Self::process_batch_optimized(&mut storage_client, &batch).await {
                Ok(concepts) => {
                    progress.concepts_created += concepts;
                    progress.processed_items += batch.len() as u64;
                    info!("Processed final batch of {} items", batch.len());
                }
                Err(err) => {
                    warn!("Final batch processing failed: {}", err);
                    progress.failed_items += batch.len() as u64;
                }
            }
        }
        
        // Final progress calculation
        progress.current_rate = progress.processed_items as f64 / start_time.elapsed().as_secs_f64();
        
        info!("Job {} completed: {} items processed, {} concepts created, {:.1} items/sec", 
              job.id, progress.processed_items, progress.concepts_created, progress.current_rate);
        
        Ok(progress)
    }
    
    // High-performance batch processing with optimized memory usage
    async fn process_batch_optimized(
        storage_client: &mut storage::TcpStorageClient,
        batch: &[adapters::DataItem],
    ) -> Result<u64> {
        let mut concepts_created = 0u64;
        
        // Convert to concepts for batch processing
        let concepts: Vec<storage::Concept> = batch
            .iter()
            .map(|item| storage::Concept {
                content: item.content.clone(),
                metadata: item.metadata.clone(),
                embedding: None,
            })
            .collect();
            
        // Process batch via storage client
        match storage_client.batch_learn_concepts(concepts).await {
            Ok(concept_ids) => {
                concepts_created = concept_ids.len() as u64;
                info!("Created {} concepts in batch", concepts_created);
            }
            Err(err) => {
                warn!("Batch processing failed: {}", err);
                return Err(err);
            }
        }
        
        Ok(concepts_created)
    }
    // Simplified batch processing
    async fn process_batch_simple(
        _storage_client: &mut storage::TcpStorageClient,
        _batch_size: usize,
    ) -> Result<u64> {
        // Simulate batch processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(10) // Mock result
    }
    
    async fn handle_job_event(&mut self, event: JobEvent) -> Result<()> {
        match event {
            JobEvent::Started(job_id) => {
                if let Some(job) = self.active_jobs.get_mut(&job_id) {
                    job.status = JobStatus::Running;
                }
            }
            JobEvent::Progress(job_id, progress) => {
                if let Some(job) = self.active_jobs.get_mut(&job_id) {
                    job.progress = progress;
                }
            }
            JobEvent::Completed(job_id, progress) => {
                if let Some(job) = self.active_jobs.get_mut(&job_id) {
                    job.status = JobStatus::Completed;
                    job.progress = progress;
                    job.completed_at = Some(chrono::Utc::now());
                }
            }
            JobEvent::Failed(job_id, error) => {
                if let Some(job) = self.active_jobs.get_mut(&job_id) {
                    job.status = JobStatus::Failed;
                    job.error = Some(error);
                    job.completed_at = Some(chrono::Utc::now());
                }
            }
        }
        
        Ok(())
    }
    
    /// Get job status
    pub fn get_job(&self, job_id: &str) -> Option<&IngestionJob> {
        self.active_jobs.get(job_id)
    }
    
    /// List all jobs
    pub fn list_jobs(&self) -> Vec<&IngestionJob> {
        self.active_jobs.values().collect()
    }

    /// Expose storage server address (for stateless handlers)
    pub fn storage_server_address(&self) -> String {
        self.config.storage_server.clone()
    }

    /// Simple ingestion API for direct content lists (unified pipeline)
    pub async fn ingest_contents(&mut self, contents: Vec<String>) -> Result<Vec<String>> {
        let concepts: Vec<crate::storage::Concept> = contents.into_iter().map(|c| crate::storage::Concept {
            content: c,
            metadata: HashMap::new(),
            embedding: None, // Storage server generates embeddings
        }).collect();
        self.storage_client.batch_learn_concepts(concepts).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ingester_creation() {
        let config = IngesterConfig::default();
        // Test would need mock storage server
        // let ingester = BulkIngester::new(config).await.unwrap();
    }
}