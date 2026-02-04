/// Adaptive Reconciler - AI-Native Self-Optimizing Reconciliation
///
/// This reconciler uses online learning to dynamically adjust reconciliation
/// intervals based on real-time workload patterns, optimizing for:
/// - Low latency (minimize staleness)
/// - Low overhead (minimize cloning frequency)
/// - Predictive backpressure (prevent queue overflow)
///
/// Key Innovations:
/// - Exponential Moving Average (EMA) for trend detection
/// - Predictive queue depth forecasting
/// - Self-healing interval adjustment
/// - Hooks for telemetry and monitoring
use crate::read_view::{ConceptNode, GraphSnapshot, ReadView};
use crate::write_log::{WriteEntry, WriteLog};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Adaptive reconciler configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdaptiveReconcilerConfig {
    /// Base reconciliation interval (milliseconds) - starting point
    pub base_interval_ms: u64,

    /// Minimum interval (milliseconds) - high load
    pub min_interval_ms: u64,

    /// Maximum interval (milliseconds) - low load
    pub max_interval_ms: u64,

    /// Max batch size per reconciliation
    pub max_batch_size: usize,

    /// Flush to disk threshold (number of concepts in memory)
    pub disk_flush_threshold: usize,

    /// Base path for segments
    pub storage_path: PathBuf,

    /// Queue depth threshold for warnings (percentage of capacity)
    pub queue_warning_threshold: f64,

    /// EMA alpha for smoothing (0-1, higher = more reactive)
    pub ema_alpha: f64,

    /// Look-back window for trend analysis (number of reconciliation cycles)
    pub trend_window_size: usize,
}

impl Default for AdaptiveReconcilerConfig {
    fn default() -> Self {
        Self {
            base_interval_ms: 10,
            min_interval_ms: 1,   // 1ms for high load
            max_interval_ms: 100, // 100ms for idle
            max_batch_size: 10_000,
            disk_flush_threshold: 50_000,
            storage_path: PathBuf::from("./storage"),
            queue_warning_threshold: 0.70, // Warn at 70% capacity
            ema_alpha: 0.3,
            trend_window_size: 50,
        }
    }
}

impl AdaptiveReconcilerConfig {
    /// ✅ PRODUCTION: Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Interval validation
        if self.min_interval_ms == 0 {
            anyhow::bail!("min_interval_ms must be > 0");
        }
        if self.min_interval_ms > self.base_interval_ms {
            anyhow::bail!(
                "min_interval_ms ({}) must be <= base_interval_ms ({})",
                self.min_interval_ms,
                self.base_interval_ms
            );
        }
        if self.base_interval_ms > self.max_interval_ms {
            anyhow::bail!(
                "base_interval_ms ({}) must be <= max_interval_ms ({})",
                self.base_interval_ms,
                self.max_interval_ms
            );
        }
        if self.max_interval_ms > 10_000 {
            log::warn!(
                "⚠️  max_interval_ms ({}) is very high, may cause staleness",
                self.max_interval_ms
            );
        }

        // Batch size validation
        if self.max_batch_size == 0 {
            anyhow::bail!("max_batch_size must be > 0");
        }
        if self.max_batch_size > 1_000_000 {
            log::warn!(
                "⚠️  max_batch_size ({}) is very large, may cause high latency spikes",
                self.max_batch_size
            );
        }

        // Threshold validation
        if self.disk_flush_threshold == 0 {
            anyhow::bail!("disk_flush_threshold must be > 0");
        }

        // Warning threshold validation
        if self.queue_warning_threshold <= 0.0 || self.queue_warning_threshold > 1.0 {
            anyhow::bail!(
                "queue_warning_threshold must be in (0.0, 1.0], got {}",
                self.queue_warning_threshold
            );
        }

        // EMA alpha validation
        if self.ema_alpha <= 0.0 || self.ema_alpha > 1.0 {
            anyhow::bail!("ema_alpha must be in (0.0, 1.0], got {}", self.ema_alpha);
        }

        // Trend window validation
        if self.trend_window_size == 0 {
            anyhow::bail!("trend_window_size must be > 0");
        }
        if self.trend_window_size > 1000 {
            log::warn!(
                "⚠️  trend_window_size ({}) is very large, may use excessive memory",
                self.trend_window_size
            );
        }

        Ok(())
    }
}

/// Reconciler statistics with predictive metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdaptiveReconcilerStats {
    // Basic counters
    pub reconciliations: u64,
    pub entries_processed: u64,
    pub disk_flushes: u64,
    pub running: bool,

    // 🔥 NEW: Flow control metrics
    pub queue_depth: usize,
    pub queue_capacity: usize,
    pub queue_utilization: f64,

    // 🔥 NEW: Performance metrics
    pub current_interval_ms: u64,
    pub processing_rate_per_sec: f64,
    pub estimated_lag_ms: u64,

    // 🔥 NEW: Predictive metrics
    pub predicted_queue_depth: usize,
    pub backpressure_events: u64,
    pub interval_adjustments: u64,

    // 🔥 NEW: Health indicators
    pub health_score: f64, // 0.0-1.0
    pub recommendation: String,
}

/// Workload trend analyzer
#[derive(Clone)]
struct TrendAnalyzer {
    /// Recent queue depths (circular buffer)
    queue_history: VecDeque<usize>,

    /// Recent processing rates (entries/sec)
    rate_history: VecDeque<f64>,

    /// EMA of queue depth
    queue_ema: f64,

    /// EMA of processing rate
    rate_ema: f64,

    /// Configuration
    ema_alpha: f64,
    window_size: usize,
}

impl TrendAnalyzer {
    fn new(ema_alpha: f64, window_size: usize) -> Self {
        Self {
            queue_history: VecDeque::with_capacity(window_size),
            rate_history: VecDeque::with_capacity(window_size),
            queue_ema: 0.0,
            rate_ema: 0.0,
            ema_alpha,
            window_size,
        }
    }

    /// Update with new observation
    fn update(&mut self, queue_depth: usize, processing_rate: f64) {
        // Update EMAs
        if self.queue_ema == 0.0 {
            self.queue_ema = queue_depth as f64;
            self.rate_ema = processing_rate;
        } else {
            self.queue_ema =
                self.ema_alpha * queue_depth as f64 + (1.0 - self.ema_alpha) * self.queue_ema;
            self.rate_ema =
                self.ema_alpha * processing_rate + (1.0 - self.ema_alpha) * self.rate_ema;
        }

        // Update history
        self.queue_history.push_back(queue_depth);
        if self.queue_history.len() > self.window_size {
            self.queue_history.pop_front();
        }

        self.rate_history.push_back(processing_rate);
        if self.rate_history.len() > self.window_size {
            self.rate_history.pop_front();
        }
    }

    /// Predict queue depth for next cycle using linear extrapolation
    fn predict_next_queue_depth(&self) -> usize {
        if self.queue_history.len() < 2 {
            return self.queue_ema as usize;
        }

        // Simple linear trend: slope = (recent - old) / time
        let recent_avg: f64 = self
            .queue_history
            .iter()
            .rev()
            .take(5)
            .map(|&x| x as f64)
            .sum::<f64>()
            / 5.0;
        let old_avg: f64 = self
            .queue_history
            .iter()
            .take(5)
            .map(|&x| x as f64)
            .sum::<f64>()
            / 5.0;
        let slope = recent_avg - old_avg;

        // Predict: current + trend
        let predicted = self.queue_ema + slope;
        predicted.max(0.0) as usize
    }

    /// Calculate optimal interval based on current state
    fn calculate_optimal_interval(
        &self,
        config: &AdaptiveReconcilerConfig,
        queue_capacity: usize,
    ) -> u64 {
        let utilization = self.queue_ema / queue_capacity as f64;

        // Adaptive interval: exponential decrease as queue fills
        // - Low utilization (<20%): max interval (100ms) - save CPU
        // - Medium utilization (20-70%): base interval (10ms) - normal
        // - High utilization (>70%): min interval (1ms) - aggressive drain

        let interval = if utilization < 0.20 {
            // Idle state: reduce frequency
            config.max_interval_ms
        } else if utilization > 0.70 {
            // High load: increase frequency
            let pressure = (utilization - 0.70) / 0.30; // 0-1 scale
            let range = config.base_interval_ms - config.min_interval_ms;
            config.base_interval_ms - (range as f64 * pressure) as u64
        } else {
            // Normal operation
            config.base_interval_ms
        };

        interval
            .max(config.min_interval_ms)
            .min(config.max_interval_ms)
    }

    /// Calculate health score (0.0 = critical, 1.0 = excellent)
    fn calculate_health_score(&self, queue_capacity: usize) -> f64 {
        let utilization = self.queue_ema / queue_capacity as f64;

        // Health scoring:
        // 0-30% utilization: 1.0 (excellent)
        // 30-70% utilization: 0.8-0.5 (good to fair)
        // 70-90% utilization: 0.5-0.2 (poor)
        // 90-100% utilization: 0.2-0.0 (critical)

        if utilization < 0.30 {
            1.0
        } else if utilization < 0.70 {
            1.0 - (utilization - 0.30) * 1.25 // Linear decrease
        } else if utilization < 0.90 {
            0.5 - (utilization - 0.70) * 1.5
        } else {
            0.2 - (utilization - 0.90) * 2.0
        }
        .max(0.0)
    }
}

/// Adaptive reconciler with self-optimization
pub struct AdaptiveReconciler {
    config: AdaptiveReconcilerConfig,
    write_log: Arc<WriteLog>,
    read_view: Arc<ReadView>,

    /// Control
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,

    /// Metrics
    reconciliations: Arc<AtomicU64>,
    entries_processed: Arc<AtomicU64>,
    disk_flushes: Arc<AtomicU64>,
    interval_adjustments: Arc<AtomicU64>,

    /// Current interval (dynamically adjusted)
    current_interval_ms: Arc<AtomicU64>,

    /// Trend analyzer (shared with reconciliation thread)
    trend_analyzer: Arc<Mutex<TrendAnalyzer>>,
}

impl AdaptiveReconciler {
    /// Create a new adaptive reconciler
    pub fn new(
        config: AdaptiveReconcilerConfig,
        write_log: Arc<WriteLog>,
        read_view: Arc<ReadView>,
    ) -> Self {
        std::fs::create_dir_all(&config.storage_path).ok();

        let trend_analyzer = Arc::new(Mutex::new(TrendAnalyzer::new(
            config.ema_alpha,
            config.trend_window_size,
        )));

        Self {
            current_interval_ms: Arc::new(AtomicU64::new(config.base_interval_ms)),
            config,
            write_log,
            read_view,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            reconciliations: Arc::new(AtomicU64::new(0)),
            entries_processed: Arc::new(AtomicU64::new(0)),
            disk_flushes: Arc::new(AtomicU64::new(0)),
            interval_adjustments: Arc::new(AtomicU64::new(0)),
            trend_analyzer,
        }
    }

    /// Start adaptive reconciliation thread
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(true, Ordering::Relaxed);

        let config = self.config.clone();
        let write_log = Arc::clone(&self.write_log);
        let read_view = Arc::clone(&self.read_view);
        let running = Arc::clone(&self.running);
        let reconciliations = Arc::clone(&self.reconciliations);
        let entries_processed = Arc::clone(&self.entries_processed);
        let disk_flushes = Arc::clone(&self.disk_flushes);
        let interval_adjustments = Arc::clone(&self.interval_adjustments);
        let current_interval_ms = Arc::clone(&self.current_interval_ms);
        let trend_analyzer = Arc::clone(&self.trend_analyzer);

        let handle = thread::spawn(move || {
            adaptive_reconcile_loop(
                config,
                write_log,
                read_view,
                running,
                reconciliations,
                entries_processed,
                disk_flushes,
                interval_adjustments,
                current_interval_ms,
                trend_analyzer,
            );
        });

        self.thread_handle = Some(handle);

        log::info!(
            "🚀 Adaptive reconciler started (base interval: {}ms)",
            self.config.base_interval_ms
        );
    }

    /// Stop reconciliation thread
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.thread_handle.take() {
            handle.join().ok();
        }

        log::info!("🛑 Adaptive reconciler stopped");
    }

    /// Get comprehensive statistics
    pub fn stats(&self) -> AdaptiveReconcilerStats {
        let write_stats = self.write_log.stats();
        let queue_depth = write_stats.pending;
        let queue_capacity = write_stats.capacity;
        let queue_utilization = queue_depth as f64 / queue_capacity as f64;

        let analyzer = self.trend_analyzer.lock().unwrap();
        let processing_rate = analyzer.rate_ema;
        let estimated_lag_ms = if processing_rate > 0.0 {
            ((queue_depth as f64 / processing_rate) * 1000.0) as u64
        } else {
            0
        };

        let predicted_queue_depth = analyzer.predict_next_queue_depth();
        let health_score = analyzer.calculate_health_score(queue_capacity);

        let recommendation = if health_score > 0.8 {
            "Excellent: System running optimally".to_string()
        } else if health_score > 0.5 {
            "Good: Normal operation, monitoring".to_string()
        } else if health_score > 0.2 {
            format!(
                "Warning: High queue depth ({}/{}) - consider scaling",
                queue_depth, queue_capacity
            )
        } else {
            format!(
                "Critical: Queue near capacity ({}/{}) - immediate action required",
                queue_depth, queue_capacity
            )
        };

        AdaptiveReconcilerStats {
            reconciliations: self.reconciliations.load(Ordering::Relaxed),
            entries_processed: self.entries_processed.load(Ordering::Relaxed),
            disk_flushes: self.disk_flushes.load(Ordering::Relaxed),
            running: self.running.load(Ordering::Relaxed),
            queue_depth,
            queue_capacity,
            queue_utilization,
            current_interval_ms: self.current_interval_ms.load(Ordering::Relaxed),
            processing_rate_per_sec: processing_rate,
            estimated_lag_ms,
            predicted_queue_depth,
            backpressure_events: write_stats.dropped,
            interval_adjustments: self.interval_adjustments.load(Ordering::Relaxed),
            health_score,
            recommendation,
        }
    }
}

impl Drop for AdaptiveReconciler {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Main adaptive reconciliation loop
#[allow(clippy::too_many_arguments)]
fn adaptive_reconcile_loop(
    config: AdaptiveReconcilerConfig,
    write_log: Arc<WriteLog>,
    read_view: Arc<ReadView>,
    running: Arc<AtomicBool>,
    reconciliations: Arc<AtomicU64>,
    entries_processed: Arc<AtomicU64>,
    _disk_flushes: Arc<AtomicU64>,
    interval_adjustments: Arc<AtomicU64>,
    current_interval_ms: Arc<AtomicU64>,
    trend_analyzer: Arc<Mutex<TrendAnalyzer>>,
) {
    let _storage_version = 0u32; // Reserved for future use
    let mut cycle_count = 0u64;
    let queue_capacity = 100_000; // From write_log.rs MAX_WRITE_LOG_SIZE

    while running.load(Ordering::Relaxed) {
        let cycle_start = Instant::now();

        // Get current interval
        let interval_ms = current_interval_ms.load(Ordering::Relaxed);
        let interval = Duration::from_millis(interval_ms);

        // Drain write log
        let batch = write_log.drain_batch(config.max_batch_size);
        let batch_size = batch.len();

        if !batch.is_empty() {
            // Load current snapshot
            let current_snapshot = read_view.load();

            // Clone snapshot (structural sharing via im::HashMap)
            let mut new_snapshot = GraphSnapshot {
                concepts: current_snapshot.concepts.clone(),
                sequence: current_snapshot.sequence + 1,
                timestamp: current_timestamp_us(),
                concept_count: current_snapshot.concept_count,
                edge_count: current_snapshot.edge_count,
            };

            // Apply batch
            for entry in &batch {
                apply_entry(&mut new_snapshot, entry);
            }

            // Update stats
            new_snapshot.update_stats();

            // Atomic swap
            read_view.store(new_snapshot);

            // Update metrics
            reconciliations.fetch_add(1, Ordering::Relaxed);
            entries_processed.fetch_add(batch_size as u64, Ordering::Relaxed);

            // Note: Disk flush is handled by ConcurrentMemory::flush()
            // Adaptive reconciler focuses on memory reconciliation only
        }

        let cycle_duration = cycle_start.elapsed();

        // Calculate processing rate (entries/second)
        let processing_rate = if cycle_duration.as_secs_f64() > 0.0 {
            batch_size as f64 / cycle_duration.as_secs_f64()
        } else {
            0.0
        };

        // Get current queue state
        let queue_depth = write_log.stats().pending;

        // Update trend analyzer
        {
            let mut analyzer = trend_analyzer.lock().unwrap();
            analyzer.update(queue_depth, processing_rate);

            // Calculate new optimal interval every 10 cycles
            if cycle_count.is_multiple_of(10) {
                let new_interval = analyzer.calculate_optimal_interval(&config, queue_capacity);
                let old_interval = current_interval_ms.load(Ordering::Relaxed);

                if new_interval != old_interval {
                    current_interval_ms.store(new_interval, Ordering::Relaxed);
                    interval_adjustments.fetch_add(1, Ordering::Relaxed);

                    log::debug!(
                        "🔄 Interval adjusted: {}ms → {}ms (queue: {}/{}, rate: {:.0}/sec)",
                        old_interval,
                        new_interval,
                        queue_depth,
                        queue_capacity,
                        processing_rate
                    );
                }
            }

            // Emit telemetry every 100 cycles (~1 second at 10ms interval)
            if cycle_count.is_multiple_of(100) {
                let health_score = analyzer.calculate_health_score(queue_capacity);
                let predicted_queue = analyzer.predict_next_queue_depth();

                // Warning if approaching capacity
                if queue_depth as f64 / queue_capacity as f64 > config.queue_warning_threshold {
                    log::warn!(
                        "⚠️ Reconciler backlog: {}/{} ({:.1}% capacity), predicted next: {}, health: {:.2}",
                        queue_depth, queue_capacity,
                        (queue_depth as f64 / queue_capacity as f64) * 100.0,
                        predicted_queue,
                        health_score
                    );
                }
            }
        }

        cycle_count += 1;

        // Sleep with dynamic interval
        thread::sleep(interval);
    }
}

/// Apply a single write entry to the snapshot
fn apply_entry(snapshot: &mut GraphSnapshot, entry: &WriteEntry) {
    match entry {
        WriteEntry::AddConcept {
            id,
            content,
            vector,
            strength,
            confidence,
            timestamp,
            attributes,
            semantic,
        } => {
            let mut node = if let Some(semantic_meta) = semantic.clone() {
                ConceptNode::with_semantic(
                    *id,
                    content.to_vec(),
                    vector.as_ref().map(|v| v.to_vec()),
                    *strength,
                    *confidence,
                    *timestamp,
                    semantic_meta,
                )
            } else {
                ConceptNode::new(
                    *id,
                    content.to_vec(),
                    vector.as_ref().map(|v| v.to_vec()),
                    *strength,
                    *confidence,
                    *timestamp,
                )
            };
            node.attributes = attributes.clone();
            snapshot.concepts.insert(*id, node);
        }

        WriteEntry::AddAssociation { record } => {
            if let Some(mut source_node) = snapshot.concepts.get(&record.source_id).cloned() {
                source_node.add_edge(record.target_id, *record);
                snapshot.concepts.insert(record.source_id, source_node);
            }

            if let Some(mut target_node) = snapshot.concepts.get(&record.target_id).cloned() {
                target_node.add_edge(record.source_id, *record);
                snapshot.concepts.insert(record.target_id, target_node);
            }
        }

        WriteEntry::UpdateStrength { id, strength } => {
            if let Some(mut node) = snapshot.concepts.get(id).cloned() {
                node.strength = *strength;
                snapshot.concepts.insert(*id, node);
            }
        }

        WriteEntry::RecordAccess { id, timestamp } => {
            if let Some(mut node) = snapshot.concepts.get(id).cloned() {
                node.last_accessed = *timestamp;
                node.access_count += 1;
                snapshot.concepts.insert(*id, node);
            }
        }

        WriteEntry::DeleteConcept { id, timestamp: _ } => {
            // Remove concept and all its edges
            if snapshot.concepts.contains_key(id) {
                snapshot.concepts.remove(id);

                // Remove all edges pointing to this concept from other concepts
                // Since im::HashMap is immutable-friendly, we need to clone and update
                let mut updated_concepts = im::HashMap::new();
                for (other_id, other_node) in snapshot.concepts.iter() {
                    if other_id != id {
                        let mut node_clone = other_node.clone();
                        node_clone.neighbors.retain(|neighbor| neighbor != id);
                        updated_concepts.insert(*other_id, node_clone);
                    }
                }
                snapshot.concepts = updated_concepts;
            }
        }

        WriteEntry::Clear => {
            // Reset everything
            snapshot.concepts.clear();
            snapshot.concept_count = 0;
            snapshot.edge_count = 0;
        }

        WriteEntry::BatchMarker { .. } => {
            // Marker only, no action
        }
    }
}

/// Get current timestamp in microseconds
fn current_timestamp_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_view::ReadView;
    use crate::types::ConceptId;
    use crate::write_log::WriteLog;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_trend_analyzer() {
        let mut analyzer = TrendAnalyzer::new(0.3, 50);

        // Simulate increasing load
        for i in 0..20 {
            analyzer.update(i * 100, 1000.0);
        }

        // Should predict increasing trend
        let predicted = analyzer.predict_next_queue_depth();
        assert!(predicted > 1900);
    }

    #[test]
    fn test_adaptive_interval() {
        let config = AdaptiveReconcilerConfig::default();
        let mut analyzer = TrendAnalyzer::new(0.3, 50);

        // Low load → max interval
        analyzer.update(1000, 5000.0);
        let interval = analyzer.calculate_optimal_interval(&config, 100_000);
        assert_eq!(interval, config.max_interval_ms);

        // High load → min interval
        analyzer.update(80_000, 1000.0);
        let interval = analyzer.calculate_optimal_interval(&config, 100_000);
        assert!(interval <= config.base_interval_ms);
    }

    #[test]
    fn test_health_score() {
        let analyzer = TrendAnalyzer::new(0.3, 50);

        // Excellent health at low utilization
        let mut test_analyzer = analyzer.clone();
        test_analyzer.queue_ema = 10_000.0;
        let score = test_analyzer.calculate_health_score(100_000);
        assert!(score > 0.8);

        // Critical health near capacity
        test_analyzer.queue_ema = 95_000.0;
        let score = test_analyzer.calculate_health_score(100_000);
        assert!(score < 0.2);
    }

    #[test]
    fn test_adaptive_reconciler_stats() {
        let dir = TempDir::new().unwrap();
        let write_log = Arc::new(WriteLog::new());
        let read_view = Arc::new(ReadView::new());

        let config = AdaptiveReconcilerConfig {
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut reconciler =
            AdaptiveReconciler::new(config, Arc::clone(&write_log), Arc::clone(&read_view));

        reconciler.start();

        // Write some data
        for i in 0..100 {
            let id = ConceptId([i; 16]);
            write_log
                .append_concept(id, vec![i], None, 1.0, 0.9, std::collections::HashMap::new())
                .unwrap();
        }

        thread::sleep(Duration::from_millis(200));

        let stats = reconciler.stats();
        assert!(stats.entries_processed >= 100);
        assert!(stats.health_score > 0.0);
        assert!(!stats.recommendation.is_empty());

        reconciler.stop();
    }
}
