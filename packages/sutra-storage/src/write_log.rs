/// Lock-free write log for continuous learning
/// 
/// Append-only structure optimized for burst writes.
/// Writers never block, readers never see partial writes.
/// 
/// Design:
/// - Crossbeam channel for lock-free producer-consumer
/// - Bounded capacity with backpressure (drop old on overflow)
/// - Batch drain for reconciliation
/// - Zero-copy where possible

use crate::types::{ConceptId, AssociationRecord};
use crossbeam::channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Maximum write log entries before backpressure
const MAX_WRITE_LOG_SIZE: usize = 100_000;

/// Write log entry types
#[derive(Debug, Clone)]
pub enum WriteEntry {
    /// Add a new concept with content and optional vector
    AddConcept {
        id: ConceptId,
        content: Box<[u8]>,      // Arbitrary binary content
        vector: Option<Box<[f32]>>,
        strength: f32,
        confidence: f32,
        timestamp: u64,
    },
    
    /// Add an association between concepts
    AddAssociation {
        record: AssociationRecord,
    },
    
    /// Update concept strength (from temporal decay)
    UpdateStrength {
        id: ConceptId,
        strength: f32,
    },
    
    /// Record access (for heat tracking)
    RecordAccess {
        id: ConceptId,
        timestamp: u64,
    },
    
    /// Batch marker (for checkpointing)
    BatchMarker {
        sequence: u64,
    },
}

/// Lock-free write log
pub struct WriteLog {
    /// Write channel (producers)
    sender: Sender<WriteEntry>,
    
    /// Read channel (reconciler)
    receiver: Receiver<WriteEntry>,
    
    /// Sequence counter
    sequence: Arc<AtomicU64>,
    
    /// Dropped entries counter (backpressure metric)
    dropped: Arc<AtomicU64>,
    
    /// Total written
    written: Arc<AtomicU64>,
}

impl WriteLog {
    /// Create a new write log
    pub fn new() -> Self {
        let (sender, receiver) = bounded(MAX_WRITE_LOG_SIZE);
        
        Self {
            sender,
            receiver,
            sequence: Arc::new(AtomicU64::new(0)),
            dropped: Arc::new(AtomicU64::new(0)),
            written: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Append an entry (non-blocking)
    pub fn append(&self, entry: WriteEntry) -> Result<u64, WriteLogError> {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        
        match self.sender.try_send(entry) {
            Ok(()) => {
                self.written.fetch_add(1, Ordering::Relaxed);
                Ok(seq)
            }
            Err(TrySendError::Full(_)) => {
                // Backpressure: drop oldest, accept newest
                self.dropped.fetch_add(1, Ordering::Relaxed);
                Err(WriteLogError::Full)
            }
            Err(TrySendError::Disconnected(_)) => {
                Err(WriteLogError::Disconnected)
            }
        }
    }
    
    /// Append concept (convenience)
    pub fn append_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
    ) -> Result<u64, WriteLogError> {
        let timestamp = current_timestamp_us();
        
        self.append(WriteEntry::AddConcept {
            id,
            content: content.into_boxed_slice(),
            vector: vector.map(|v| v.into_boxed_slice()),
            strength,
            confidence,
            timestamp,
        })
    }
    
    /// Append association (convenience)
    pub fn append_association(
        &self,
        record: AssociationRecord,
    ) -> Result<u64, WriteLogError> {
        self.append(WriteEntry::AddAssociation { record })
    }
    
    /// Drain up to N entries (for reconciler)
    pub fn drain_batch(&self, max_entries: usize) -> Vec<WriteEntry> {
        let mut batch = Vec::with_capacity(max_entries);
        
        // Non-blocking drain
        for _ in 0..max_entries {
            match self.receiver.try_recv() {
                Ok(entry) => batch.push(entry),
                Err(_) => break,
            }
        }
        
        batch
    }
    
    /// Drain all available entries
    pub fn drain_all(&self) -> Vec<WriteEntry> {
        self.drain_batch(MAX_WRITE_LOG_SIZE)
    }
    
    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.sequence.load(Ordering::Relaxed)
    }
    
    /// Get statistics
    pub fn stats(&self) -> WriteLogStats {
        WriteLogStats {
            sequence: self.sequence.load(Ordering::Relaxed),
            written: self.written.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            pending: self.receiver.len(),
            capacity: MAX_WRITE_LOG_SIZE,
        }
    }
    
    /// Get receiver for reconciler
    pub fn receiver(&self) -> &Receiver<WriteEntry> {
        &self.receiver
    }
}

impl Default for WriteLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Write log statistics
#[derive(Debug, Clone, Copy)]
pub struct WriteLogStats {
    pub sequence: u64,
    pub written: u64,
    pub dropped: u64,
    pub pending: usize,
    pub capacity: usize,
}

/// Write log errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteLogError {
    /// Log is full (backpressure)
    Full,
    /// Channel disconnected
    Disconnected,
}

impl std::fmt::Display for WriteLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "Write log full (backpressure)"),
            Self::Disconnected => write!(f, "Write log disconnected"),
        }
    }
}

impl std::error::Error for WriteLogError {}

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
    use crate::types::AssociationType;
    
    #[test]
    fn test_write_log_basic() {
        let log = WriteLog::new();
        
        let id = ConceptId([1; 16]);
        let content = b"test concept".to_vec();
        
        let seq = log.append_concept(id, content, None, 1.0, 0.9).unwrap();
        assert_eq!(seq, 0);
        
        let stats = log.stats();
        assert_eq!(stats.written, 1);
        assert_eq!(stats.pending, 1);
    }
    
    #[test]
    fn test_drain_batch() {
        let log = WriteLog::new();
        
        // Write 10 entries
        for i in 0..10 {
            let id = ConceptId([i; 16]);
            log.append_concept(id, vec![i], None, 1.0, 0.9).unwrap();
        }
        
        // Drain 5
        let batch = log.drain_batch(5);
        assert_eq!(batch.len(), 5);
        
        // 5 remaining
        assert_eq!(log.stats().pending, 5);
    }
    
    #[test]
    fn test_drain_all() {
        let log = WriteLog::new();
        
        for i in 0..100 {
            let id = ConceptId([i as u8; 16]);
            log.append_concept(id, vec![i as u8], None, 1.0, 0.9).unwrap();
        }
        
        let all = log.drain_all();
        assert_eq!(all.len(), 100);
        assert_eq!(log.stats().pending, 0);
    }
    
    #[test]
    fn test_sequence_increment() {
        let log = WriteLog::new();
        
        let id = ConceptId([1; 16]);
        
        let seq1 = log.append_concept(id, vec![1], None, 1.0, 0.9).unwrap();
        let seq2 = log.append_concept(id, vec![2], None, 1.0, 0.9).unwrap();
        let seq3 = log.append_concept(id, vec![3], None, 1.0, 0.9).unwrap();
        
        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert_eq!(seq3, 2);
    }
    
    #[test]
    fn test_association_append() {
        let log = WriteLog::new();
        
        let record = AssociationRecord::new(
            ConceptId([1; 16]),
            ConceptId([2; 16]),
            AssociationType::Semantic,
            0.8,
        );
        
        let seq = log.append_association(record).unwrap();
        assert_eq!(seq, 0);
        
        let batch = log.drain_all();
        assert_eq!(batch.len(), 1);
        
        match &batch[0] {
            WriteEntry::AddAssociation { record: r } => {
                assert_eq!(r.source_id, ConceptId([1; 16]));
                assert_eq!(r.target_id, ConceptId([2; 16]));
            }
            _ => panic!("Expected AddAssociation"),
        }
    }
}
