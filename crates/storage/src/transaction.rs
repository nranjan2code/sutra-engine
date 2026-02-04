/// Two-Phase Commit (2PC) Transaction Coordinator
///
/// Ensures atomicity for cross-shard operations (associations spanning multiple shards).
/// Protocol:
/// 1. PREPARE phase: All shards lock resources and prepare to commit
/// 2. COMMIT phase: If all prepared successfully, commit; otherwise ROLLBACK
///
/// ACID guarantees:
/// - Atomicity: All shards commit or none do
/// - Consistency: No partial associations
/// - Isolation: Transactions are serialized via locks
/// - Durability: WAL ensures recovery
use crate::types::{AssociationType, ConceptId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Transaction ID generator (monotonic)
static NEXT_TXN_ID: AtomicU64 = AtomicU64::new(1);

/// Generate unique transaction ID
pub fn generate_txn_id() -> u64 {
    NEXT_TXN_ID.fetch_add(1, Ordering::SeqCst)
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxnState {
    /// Transaction started, preparing resources
    Preparing,
    /// All participants prepared successfully
    Prepared,
    /// Transaction committed
    Committed,
    /// Transaction aborted/rolled back
    Aborted,
}

/// Transaction participant (shard)
#[derive(Debug, Clone)]
pub struct Participant {
    pub shard_id: u32,
    pub state: TxnState,
    pub prepared_at: Option<Instant>,
}

/// Transaction record
#[derive(Debug, Clone)]
pub struct Transaction {
    pub txn_id: u64,
    pub operation: TxnOperation,
    pub participants: Vec<Participant>,
    pub started_at: Instant,
    pub state: TxnState,
}

/// Transaction operation types
#[derive(Debug, Clone)]
pub enum TxnOperation {
    /// Cross-shard association creation
    CreateAssociation {
        source: ConceptId,
        target: ConceptId,
        source_shard: u32,
        target_shard: u32,
        assoc_type: AssociationType,
        strength: f32,
    },
}

/// Transaction coordinator (manages 2PC protocol)
pub struct TransactionCoordinator {
    /// Active transactions (txn_id -> Transaction)
    active: Arc<RwLock<HashMap<u64, Transaction>>>,
    /// Transaction timeout (default: 5 seconds)
    timeout: Duration,
}

impl TransactionCoordinator {
    /// Create new transaction coordinator
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            active: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Start new transaction
    pub fn begin(&self, operation: TxnOperation) -> u64 {
        let txn_id = generate_txn_id();

        let participants = match &operation {
            TxnOperation::CreateAssociation {
                source_shard,
                target_shard,
                ..
            } => {
                let mut parts = vec![Participant {
                    shard_id: *source_shard,
                    state: TxnState::Preparing,
                    prepared_at: None,
                }];

                // Only add target shard if different (cross-shard case)
                if source_shard != target_shard {
                    parts.push(Participant {
                        shard_id: *target_shard,
                        state: TxnState::Preparing,
                        prepared_at: None,
                    });
                }

                parts
            }
        };

        let txn = Transaction {
            txn_id,
            operation,
            participants,
            started_at: Instant::now(),
            state: TxnState::Preparing,
        };

        self.active.write().insert(txn_id, txn);
        log::debug!("🔄 2PC: Started transaction {}", txn_id);

        txn_id
    }

    /// Mark participant as prepared
    pub fn mark_prepared(&self, txn_id: u64, shard_id: u32) -> Result<(), TxnError> {
        let mut active = self.active.write();
        let txn = active.get_mut(&txn_id).ok_or(TxnError::NotFound(txn_id))?;

        // Check timeout
        if txn.started_at.elapsed() > self.timeout {
            log::warn!("⚠️ 2PC: Transaction {} timed out", txn_id);
            txn.state = TxnState::Aborted;
            return Err(TxnError::Timeout(txn_id));
        }

        // Find participant
        let participant = txn
            .participants
            .iter_mut()
            .find(|p| p.shard_id == shard_id)
            .ok_or(TxnError::InvalidParticipant(shard_id))?;

        participant.state = TxnState::Prepared;
        participant.prepared_at = Some(Instant::now());

        log::debug!("✅ 2PC: Shard {} prepared for txn {}", shard_id, txn_id);

        // Check if all participants prepared
        let all_prepared = txn
            .participants
            .iter()
            .all(|p| p.state == TxnState::Prepared);

        if all_prepared {
            txn.state = TxnState::Prepared;
            log::debug!("✅ 2PC: All participants prepared for txn {}", txn_id);
        }

        Ok(())
    }

    /// Check if transaction is ready to commit (all participants prepared)
    pub fn is_ready_to_commit(&self, txn_id: u64) -> Result<bool, TxnError> {
        let active = self.active.read();
        let txn = active.get(&txn_id).ok_or(TxnError::NotFound(txn_id))?;

        // Check timeout
        if txn.started_at.elapsed() > self.timeout {
            return Err(TxnError::Timeout(txn_id));
        }

        Ok(txn.state == TxnState::Prepared)
    }

    /// Commit transaction (Phase 2)
    pub fn commit(&self, txn_id: u64) -> Result<(), TxnError> {
        let mut active = self.active.write();
        let txn = active.get_mut(&txn_id).ok_or(TxnError::NotFound(txn_id))?;

        if txn.state != TxnState::Prepared {
            return Err(TxnError::InvalidState {
                txn_id,
                expected: TxnState::Prepared,
                actual: txn.state,
            });
        }

        txn.state = TxnState::Committed;

        // Mark all participants as committed
        for participant in &mut txn.participants {
            participant.state = TxnState::Committed;
        }

        log::info!("✅ 2PC: Transaction {} committed", txn_id);
        Ok(())
    }

    /// Abort transaction (rollback)
    pub fn abort(&self, txn_id: u64) -> Result<(), TxnError> {
        let mut active = self.active.write();
        let txn = active.get_mut(&txn_id).ok_or(TxnError::NotFound(txn_id))?;

        txn.state = TxnState::Aborted;

        // Mark all participants as aborted
        for participant in &mut txn.participants {
            participant.state = TxnState::Aborted;
        }

        log::warn!("❌ 2PC: Transaction {} aborted", txn_id);
        Ok(())
    }

    /// Complete transaction (remove from active set)
    pub fn complete(&self, txn_id: u64) {
        let mut active = self.active.write();
        if let Some(txn) = active.remove(&txn_id) {
            log::debug!(
                "🧹 2PC: Cleaned up transaction {} (state: {:?}, duration: {:?})",
                txn_id,
                txn.state,
                txn.started_at.elapsed()
            );
        }
    }

    /// Get transaction details
    pub fn get_transaction(&self, txn_id: u64) -> Option<Transaction> {
        self.active.read().get(&txn_id).cloned()
    }

    /// Cleanup timed-out transactions (call periodically)
    pub fn cleanup_timedout(&self) -> usize {
        let mut active = self.active.write();
        let now = Instant::now();

        let timed_out: Vec<u64> = active
            .iter()
            .filter(|(_, txn)| now.duration_since(txn.started_at) > self.timeout)
            .map(|(id, _)| *id)
            .collect();

        for txn_id in &timed_out {
            if let Some(txn) = active.get_mut(txn_id) {
                txn.state = TxnState::Aborted;
                log::warn!("⏰ 2PC: Transaction {} timed out and aborted", txn_id);
            }
        }

        let count = timed_out.len();

        // Remove aborted transactions
        for txn_id in timed_out {
            active.remove(&txn_id);
        }

        count
    }

    /// Get statistics
    pub fn stats(&self) -> TxnCoordinatorStats {
        let active = self.active.read();

        let mut preparing = 0;
        let mut prepared = 0;
        let mut committed = 0;
        let mut aborted = 0;

        for txn in active.values() {
            match txn.state {
                TxnState::Preparing => preparing += 1,
                TxnState::Prepared => prepared += 1,
                TxnState::Committed => committed += 1,
                TxnState::Aborted => aborted += 1,
            }
        }

        TxnCoordinatorStats {
            active_count: active.len(),
            preparing,
            prepared,
            committed,
            aborted,
        }
    }
}

impl Default for TransactionCoordinator {
    fn default() -> Self {
        Self::new(5) // 5 second timeout
    }
}

/// Transaction coordinator statistics
#[derive(Debug, Clone)]
pub struct TxnCoordinatorStats {
    pub active_count: usize,
    pub preparing: usize,
    pub prepared: usize,
    pub committed: usize,
    pub aborted: usize,
}

/// Transaction errors
#[derive(Debug, Clone)]
pub enum TxnError {
    /// Transaction not found
    NotFound(u64),
    /// Invalid participant shard
    InvalidParticipant(u32),
    /// Transaction timed out
    Timeout(u64),
    /// Invalid state transition
    InvalidState {
        txn_id: u64,
        expected: TxnState,
        actual: TxnState,
    },
}

impl std::fmt::Display for TxnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxnError::NotFound(id) => write!(f, "Transaction {} not found", id),
            TxnError::InvalidParticipant(shard) => {
                write!(f, "Invalid participant shard: {}", shard)
            }
            TxnError::Timeout(id) => write!(f, "Transaction {} timed out", id),
            TxnError::InvalidState {
                txn_id,
                expected,
                actual,
            } => write!(
                f,
                "Transaction {} invalid state: expected {:?}, got {:?}",
                txn_id, expected, actual
            ),
        }
    }
}

impl std::error::Error for TxnError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_shard_transaction() {
        let coordinator = TransactionCoordinator::new(5);

        let txn_id = coordinator.begin(TxnOperation::CreateAssociation {
            source: ConceptId([1; 16]),
            target: ConceptId([2; 16]),
            source_shard: 0,
            target_shard: 0, // Same shard
            assoc_type: AssociationType::Semantic,
            strength: 0.9,
        });

        // Should have only 1 participant (same shard)
        let txn = coordinator.get_transaction(txn_id).unwrap();
        assert_eq!(txn.participants.len(), 1);
        assert_eq!(txn.participants[0].shard_id, 0);
    }

    #[test]
    fn test_cross_shard_transaction() {
        let coordinator = TransactionCoordinator::new(5);

        let txn_id = coordinator.begin(TxnOperation::CreateAssociation {
            source: ConceptId([1; 16]),
            target: ConceptId([2; 16]),
            source_shard: 0,
            target_shard: 1, // Different shard
            assoc_type: AssociationType::Causal,
            strength: 0.8,
        });

        // Should have 2 participants
        let txn = coordinator.get_transaction(txn_id).unwrap();
        assert_eq!(txn.participants.len(), 2);
        assert_eq!(txn.participants[0].shard_id, 0);
        assert_eq!(txn.participants[1].shard_id, 1);
    }

    #[test]
    fn test_2pc_protocol() {
        let coordinator = TransactionCoordinator::new(5);

        let txn_id = coordinator.begin(TxnOperation::CreateAssociation {
            source: ConceptId([1; 16]),
            target: ConceptId([2; 16]),
            source_shard: 0,
            target_shard: 1,
            assoc_type: AssociationType::Temporal,
            strength: 0.7,
        });

        // Phase 1: Prepare
        assert!(!coordinator.is_ready_to_commit(txn_id).unwrap());

        coordinator.mark_prepared(txn_id, 0).unwrap();
        assert!(!coordinator.is_ready_to_commit(txn_id).unwrap()); // Not all prepared yet

        coordinator.mark_prepared(txn_id, 1).unwrap();
        assert!(coordinator.is_ready_to_commit(txn_id).unwrap()); // All prepared

        // Phase 2: Commit
        coordinator.commit(txn_id).unwrap();

        let txn = coordinator.get_transaction(txn_id).unwrap();
        assert_eq!(txn.state, TxnState::Committed);
    }

    #[test]
    fn test_abort_transaction() {
        let coordinator = TransactionCoordinator::new(5);

        let txn_id = coordinator.begin(TxnOperation::CreateAssociation {
            source: ConceptId([1; 16]),
            target: ConceptId([2; 16]),
            source_shard: 0,
            target_shard: 1,
            assoc_type: AssociationType::Hierarchical,
            strength: 0.6,
        });

        // Prepare first shard
        coordinator.mark_prepared(txn_id, 0).unwrap();

        // Second shard fails - abort
        coordinator.abort(txn_id).unwrap();

        let txn = coordinator.get_transaction(txn_id).unwrap();
        assert_eq!(txn.state, TxnState::Aborted);
    }

    #[test]
    fn test_timeout() {
        let coordinator = TransactionCoordinator::new(1); // 1 second timeout

        let txn_id = coordinator.begin(TxnOperation::CreateAssociation {
            source: ConceptId([1; 16]),
            target: ConceptId([2; 16]),
            source_shard: 0,
            target_shard: 1,
            assoc_type: AssociationType::Compositional,
            strength: 0.5,
        });

        std::thread::sleep(Duration::from_millis(1100));

        // Should timeout
        let result = coordinator.mark_prepared(txn_id, 0);
        assert!(matches!(result, Err(TxnError::Timeout(_))));
    }

    #[test]
    fn test_cleanup_timedout() {
        let coordinator = TransactionCoordinator::new(1);

        // Create multiple transactions
        for _ in 0..5 {
            coordinator.begin(TxnOperation::CreateAssociation {
                source: ConceptId([1; 16]),
                target: ConceptId([2; 16]),
                source_shard: 0,
                target_shard: 1,
                assoc_type: AssociationType::Semantic,
                strength: 0.8,
            });
        }

        assert_eq!(coordinator.stats().active_count, 5);

        std::thread::sleep(Duration::from_millis(1100));

        let cleaned = coordinator.cleanup_timedout();
        assert_eq!(cleaned, 5);
        assert_eq!(coordinator.stats().active_count, 0);
    }
}
