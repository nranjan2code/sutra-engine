/// Streaming Embeddings
/// 
/// Real-time streaming embedding generation for live applications.
/// 
/// Key features:
/// - Async streaming API with backpressure
/// - Chunked text processing
/// - Real-time latency optimization
/// - Buffering and batching strategies
/// - WebSocket/gRPC support ready
/// 
/// Use cases:
/// - Live transcription embedding
/// - Real-time search indexing
/// - Streaming chat embeddings
/// - Continuous document processing

use anyhow::{anyhow, Result};
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, info, warn};

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum buffer size before applying backpressure
    pub buffer_size: usize,
    /// Chunk size for text splitting (characters)
    pub chunk_size: usize,
    /// Overlap between chunks (characters)
    pub chunk_overlap: usize,
    /// Batch multiple chunks for processing
    pub batch_size: usize,
    /// Timeout for stream inactivity (seconds)
    pub timeout_secs: u64,
    /// Enable automatic batching
    pub auto_batch: bool,
    /// Maximum latency target (milliseconds)
    pub max_latency_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            chunk_size: 512,
            chunk_overlap: 64,
            batch_size: 8,
            timeout_secs: 30,
            auto_batch: true,
            max_latency_ms: 100, // 100ms target for real-time
        }
    }
}

/// Streaming embedder
pub struct StreamingEmbedder {
    config: StreamingConfig,
    embedder: Arc<Mutex<crate::embedder::Embedder>>,
    request_tx: mpsc::Sender<StreamRequest>,
    stats: Arc<Mutex<StreamingStats>>,
}

/// Streaming request
struct StreamRequest {
    text: String,
    response_tx: tokio::sync::oneshot::Sender<Result<Vec<f32>>>,
}

/// Streaming statistics
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    pub total_requests: usize,
    pub total_chunks: usize,
    pub total_bytes: usize,
    pub average_latency_ms: f32,
    pub peak_latency_ms: f32,
    pub throughput_per_sec: f32,
    pub buffer_overflows: usize,
}

impl StreamingEmbedder {
    /// Create new streaming embedder
    pub fn new(config: StreamingConfig, embedder_config: crate::embedder::EmbedderConfig) -> Result<Self> {
        info!("Initializing streaming embedder");
        info!("  Buffer size: {}", config.buffer_size);
        info!("  Chunk size: {}", config.chunk_size);
        info!("  Batch size: {}", config.batch_size);
        info!("  Max latency: {}ms", config.max_latency_ms);
        
        let embedder = crate::embedder::Embedder::new(embedder_config)?;
        let (request_tx, request_rx) = mpsc::channel(config.buffer_size);
        
        let streaming_embedder = Self {
            config: config.clone(),
            embedder: Arc::new(Mutex::new(embedder)),
            request_tx,
            stats: Arc::new(Mutex::new(StreamingStats::default())),
        };
        
        // Spawn processing worker
        let embedder_clone = streaming_embedder.embedder.clone();
        let config_clone = config.clone();
        let stats_clone = streaming_embedder.stats.clone();
        
        tokio::spawn(async move {
            Self::process_stream(embedder_clone, config_clone, request_rx, stats_clone).await;
        });
        
        info!("Streaming embedder initialized");
        
        Ok(streaming_embedder)
    }
    
    /// Process incoming stream
    async fn process_stream(
        embedder: Arc<Mutex<crate::embedder::Embedder>>,
        config: StreamingConfig,
        mut request_rx: mpsc::Receiver<StreamRequest>,
        stats: Arc<Mutex<StreamingStats>>,
    ) {
        info!("Stream processor started");
        
        let mut batch_buffer: Vec<StreamRequest> = Vec::new();
        let mut last_process = std::time::Instant::now();
        
        loop {
            tokio::select! {
                // Receive new request
                request = request_rx.recv() => {
                    match request {
                        Some(req) => {
                            batch_buffer.push(req);
                            
                            // Process batch if full or latency target exceeded
                            let should_process = batch_buffer.len() >= config.batch_size
                                || last_process.elapsed().as_millis() >= config.max_latency_ms as u128;
                            
                            if should_process && !batch_buffer.is_empty() {
                                Self::process_batch(
                                    &embedder,
                                    &mut batch_buffer,
                                    &stats,
                                ).await;
                                last_process = std::time::Instant::now();
                            }
                        }
                        None => {
                            info!("Stream processor stopped (channel closed)");
                            break;
                        }
                    }
                }
                
                // Timeout: process pending batch
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(config.max_latency_ms)) => {
                    if !batch_buffer.is_empty() {
                        Self::process_batch(
                            &embedder,
                            &mut batch_buffer,
                            &stats,
                        ).await;
                        last_process = std::time::Instant::now();
                    }
                }
            }
        }
    }
    
    /// Process a batch of requests
    async fn process_batch(
        embedder: &Arc<Mutex<crate::embedder::Embedder>>,
        batch: &mut Vec<StreamRequest>,
        stats: &Arc<Mutex<StreamingStats>>,
    ) {
        if batch.is_empty() {
            return;
        }
        
        let start = std::time::Instant::now();
        
        // Extract texts
        let texts: Vec<String> = batch.iter().map(|r| r.text.clone()).collect();
        
        // Generate embeddings
        let result = {
            let mut emb = embedder.lock().await;
            emb.embed_batch(&texts)
        };
        
        let elapsed = start.elapsed().as_millis() as f32;
        
        // Send responses
        match result {
            Ok(embeddings) => {
                for (request, embedding) in batch.drain(..).zip(embeddings.into_iter()) {
                    let _ = request.response_tx.send(Ok(embedding));
                }
                
                // Update stats
                let mut stats = stats.lock().await;
                stats.total_requests += texts.len();
                stats.total_chunks += texts.len();
                
                // Update latency (exponential moving average)
                stats.average_latency_ms = if stats.average_latency_ms == 0.0 {
                    elapsed
                } else {
                    stats.average_latency_ms * 0.9 + elapsed * 0.1
                };
                
                if elapsed > stats.peak_latency_ms {
                    stats.peak_latency_ms = elapsed;
                }
                
                debug!("Processed batch of {} in {:.2}ms", texts.len(), elapsed);
            }
            Err(e) => {
                warn!("Batch processing error: {}", e);
                for request in batch.drain(..) {
                    let _ = request.response_tx.send(Err(anyhow!("Processing failed: {}", e)));
                }
            }
        }
    }
    
    /// Embed single text (streaming)
    pub async fn embed_stream(&self, text: String) -> Result<Vec<f32>> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        self.request_tx.send(StreamRequest {
            text,
            response_tx,
        }).await.map_err(|e| {
            let mut stats_guard = self.stats.blocking_lock();
            stats_guard.buffer_overflows += 1;
            anyhow!("Stream buffer full: {}", e)
        })?;
        
        response_rx.await
            .map_err(|e| anyhow!("Failed to receive response: {}", e))?
    }
    
    /// Create text chunk stream
    pub fn create_chunk_stream(&self, text: String) -> ChunkStream {
        ChunkStream::new(text, self.config.chunk_size, self.config.chunk_overlap)
    }
    
    /// Embed text chunks as stream
    pub async fn embed_chunks_stream(&self, text: String) -> Result<Pin<Box<dyn Stream<Item = Result<Vec<f32>>> + Send>>> {
        let chunks = self.create_chunk_stream(text).collect_chunks();
        let embedder = self.clone_for_stream();
        
        let (tx, rx) = mpsc::channel(self.config.buffer_size);
        
        tokio::spawn(async move {
            for chunk in chunks {
                let result = embedder.embed_stream(chunk).await;
                if tx.send(result).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(Box::pin(ReceiverStream::new(rx)))
    }
    
    /// Clone for async stream processing
    fn clone_for_stream(&self) -> Self {
        Self {
            config: self.config.clone(),
            embedder: self.embedder.clone(),
            request_tx: self.request_tx.clone(),
            stats: self.stats.clone(),
        }
    }
    
    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        self.stats.lock().await.clone()
    }
    
    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = StreamingStats::default();
    }
}

/// Text chunking stream
pub struct ChunkStream {
    text: String,
    chunk_size: usize,
    overlap: usize,
    position: usize,
}

impl ChunkStream {
    /// Create new chunk stream
    pub fn new(text: String, chunk_size: usize, overlap: usize) -> Self {
        Self {
            text,
            chunk_size,
            overlap,
            position: 0,
        }
    }
    
    /// Get next chunk
    pub fn next_chunk(&mut self) -> Option<String> {
        if self.position >= self.text.len() {
            return None;
        }
        
        let start = self.position;
        let end = (self.position + self.chunk_size).min(self.text.len());
        
        let chunk = self.text[start..end].to_string();
        
        // Move position forward (with overlap)
        self.position += self.chunk_size - self.overlap;
        
        Some(chunk)
    }
    
    /// Collect all chunks
    pub fn collect_chunks(mut self) -> Vec<String> {
        let mut chunks = Vec::new();
        while let Some(chunk) = self.next_chunk() {
            chunks.push(chunk);
        }
        chunks
    }
}

/// Streaming aggregator (for combining chunk embeddings)
pub struct StreamingAggregator {
    embeddings: Vec<Vec<f32>>,
    weights: Vec<f32>,
}

impl StreamingAggregator {
    /// Create new aggregator
    pub fn new() -> Self {
        Self {
            embeddings: Vec::new(),
            weights: Vec::new(),
        }
    }
    
    /// Add embedding with optional weight
    pub fn add(&mut self, embedding: Vec<f32>, weight: Option<f32>) {
        self.embeddings.push(embedding);
        self.weights.push(weight.unwrap_or(1.0));
    }
    
    /// Compute weighted average
    pub fn aggregate(self) -> Option<Vec<f32>> {
        if self.embeddings.is_empty() {
            return None;
        }
        
        let dim = self.embeddings[0].len();
        let mut result = vec![0.0; dim];
        let mut total_weight = 0.0;
        
        for (embedding, weight) in self.embeddings.iter().zip(self.weights.iter()) {
            for (i, &val) in embedding.iter().enumerate() {
                result[i] += val * weight;
            }
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            for val in &mut result {
                *val /= total_weight;
            }
        }
        
        Some(result)
    }
}

impl Default for StreamingAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for StreamingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Streaming Statistics:\n  Total Requests: {}\n  Total Chunks: {}\n  Total Bytes: {}\n  Avg Latency: {:.2}ms\n  Peak Latency: {:.2}ms\n  Throughput: {:.1}/sec\n  Buffer Overflows: {}",
            self.total_requests,
            self.total_chunks,
            self.total_bytes,
            self.average_latency_ms,
            self.peak_latency_ms,
            self.throughput_per_sec,
            self.buffer_overflows
        )
    }
}

/// WebSocket streaming adapter (ready for integration)
#[cfg(feature = "websocket")]
pub mod websocket {
    use super::*;
    use tokio_tungstenite::tungstenite::Message;
    
    /// WebSocket streaming handler
    pub struct WebSocketHandler {
        embedder: StreamingEmbedder,
    }
    
    impl WebSocketHandler {
        pub fn new(embedder: StreamingEmbedder) -> Self {
            Self { embedder }
        }
        
        /// Handle incoming WebSocket message
        pub async fn handle_message(&self, message: Message) -> Result<Message> {
            match message {
                Message::Text(text) => {
                    let embedding = self.embedder.embed_stream(text).await?;
                    let json = serde_json::to_string(&embedding)?;
                    Ok(Message::Text(json))
                }
                _ => Err(anyhow!("Unsupported message type")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.buffer_size, 100);
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.max_latency_ms, 100);
    }
    
    #[test]
    fn test_chunk_stream() {
        let text = "a".repeat(1000);
        let mut stream = ChunkStream::new(text, 512, 64);
        
        let chunks = stream.collect_chunks();
        assert!(chunks.len() > 1);
        assert!(chunks[0].len() <= 512);
        
        // Check overlap
        if chunks.len() > 1 {
            let first_end = &chunks[0][512 - 64..];
            let second_start = &chunks[1][..64];
            assert_eq!(first_end, second_start);
        }
    }
    
    #[test]
    fn test_aggregator() {
        let mut aggregator = StreamingAggregator::new();
        
        aggregator.add(vec![1.0, 2.0, 3.0], Some(1.0));
        aggregator.add(vec![2.0, 4.0, 6.0], Some(1.0));
        
        let result = aggregator.aggregate().unwrap();
        assert_eq!(result, vec![1.5, 3.0, 4.5]);
    }
    
    #[tokio::test]
    async fn test_streaming_embedder_creation() {
        let config = StreamingConfig::default();
        let embedder_config = crate::embedder::EmbedderConfig::from_name("efficient").unwrap();
        
        let streaming = StreamingEmbedder::new(config, embedder_config);
        assert!(streaming.is_ok());
    }
}
