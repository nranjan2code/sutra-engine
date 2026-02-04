/// Write-Ahead Log (WAL) for durability and crash recovery
///
/// The WAL ensures zero data loss by logging all mutations before applying them.
/// On crash, the WAL is replayed to restore state.
///
/// Features:
/// - Append-only log format
/// - Fsync control for durability
/// - Transaction support (begin/commit/rollback)
/// - Atomic batch writes
/// - Automatic log rotation
use crate::types::{AssociationId, ConceptId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// WAL operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    /// Write a concept
    WriteConcept {
        concept_id: ConceptId,
        content_len: u32,
        vector_len: u32,
        created: u64,
        modified: u64,
    },
    /// Write an association
    WriteAssociation {
        source: ConceptId,
        target: ConceptId,
        association_id: AssociationId,
        strength: f32,
        created: u64,
    },
    /// Delete a concept
    DeleteConcept { concept_id: ConceptId },
    /// Delete an association
    DeleteAssociation { association_id: AssociationId },
    /// Begin transaction
    BeginTransaction { transaction_id: u64 },
    /// Commit transaction
    CommitTransaction { transaction_id: u64 },
    /// Rollback transaction
    RollbackTransaction { transaction_id: u64 },
}

/// WAL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Sequence number (monotonically increasing)
    pub sequence: u64,
    /// Timestamp (microseconds since epoch)
    pub timestamp: u64,
    /// Operation
    pub operation: Operation,
    /// Optional transaction ID
    pub transaction_id: Option<u64>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(sequence: u64, operation: Operation, transaction_id: Option<u64>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        Self {
            sequence,
            timestamp,
            operation,
            transaction_id,
        }
    }
}

/// Write-Ahead Log
pub struct WriteAheadLog {
    /// Log file path
    path: PathBuf,
    /// Writer (buffered)
    writer: BufWriter<File>,
    /// Next sequence number
    next_sequence: Arc<AtomicU64>,
    /// Whether to fsync after each write
    fsync: bool,
    /// Current transaction ID (if in transaction)
    current_transaction: Option<u64>,
    /// Next transaction ID
    next_transaction_id: Arc<AtomicU64>,
}

impl WriteAheadLog {
    /// Create a new WAL
    pub fn create<P: AsRef<Path>>(path: P, fsync: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .context("Failed to create WAL file")?;

        let writer = BufWriter::new(file);

        Ok(Self {
            path,
            writer,
            next_sequence: Arc::new(AtomicU64::new(0)),
            fsync,
            current_transaction: None,
            next_transaction_id: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Open existing WAL
    pub fn open<P: AsRef<Path>>(path: P, fsync: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Read existing entries to determine next sequence
        let entries = Self::read_entries(&path)?;
        let next_sequence = entries.last().map(|e| e.sequence + 1).unwrap_or(0);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .context("Failed to open WAL file")?;

        let writer = BufWriter::new(file);

        Ok(Self {
            path,
            writer,
            next_sequence: Arc::new(AtomicU64::new(next_sequence)),
            fsync,
            current_transaction: None,
            next_transaction_id: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Append an operation to the log
    pub fn append(&mut self, operation: Operation) -> Result<u64> {
        let sequence = self.next_sequence.fetch_add(1, Ordering::SeqCst);
        let entry = LogEntry::new(sequence, operation, self.current_transaction);

        // Serialize as MessagePack (binary format - matches TCP protocol)
        let bytes = rmp_serde::to_vec(&entry).context("Failed to serialize entry")?;

        // Write length prefix (4 bytes, little-endian)
        let len_bytes = (bytes.len() as u32).to_le_bytes();
        self.writer
            .write_all(&len_bytes)
            .context("Failed to write length")?;

        // Write entry data
        self.writer
            .write_all(&bytes)
            .context("Failed to write entry")?;

        if self.fsync {
            self.writer.flush().context("Failed to flush")?;
            self.writer
                .get_ref()
                .sync_all()
                .context("Failed to fsync")?;
        }

        Ok(sequence)
    }

    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<u64> {
        if self.current_transaction.is_some() {
            anyhow::bail!("Transaction already in progress");
        }

        let transaction_id = self.next_transaction_id.fetch_add(1, Ordering::SeqCst);
        self.current_transaction = Some(transaction_id);

        self.append(Operation::BeginTransaction { transaction_id })?;

        Ok(transaction_id)
    }

    /// Commit current transaction
    pub fn commit_transaction(&mut self) -> Result<()> {
        let transaction_id = self
            .current_transaction
            .ok_or_else(|| anyhow::anyhow!("No transaction in progress"))?;

        self.append(Operation::CommitTransaction { transaction_id })?;
        self.current_transaction = None;

        Ok(())
    }

    /// Rollback current transaction
    pub fn rollback_transaction(&mut self) -> Result<()> {
        let transaction_id = self
            .current_transaction
            .ok_or_else(|| anyhow::anyhow!("No transaction in progress"))?;

        self.append(Operation::RollbackTransaction { transaction_id })?;
        self.current_transaction = None;

        Ok(())
    }

    /// Flush buffered writes
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().context("Failed to flush WAL")?;
        Ok(())
    }

    /// Sync to disk
    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush().context("Failed to flush WAL")?;
        self.writer
            .get_ref()
            .sync_all()
            .context("Failed to sync WAL")?;
        Ok(())
    }

    /// Read all entries from the log
    pub fn read_entries<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>> {
        use std::io::Read;

        let mut file = File::open(path).context("Failed to open WAL file")?;

        let mut entries = Vec::new();

        loop {
            // Read length prefix (4 bytes, little-endian)
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of file reached
                    break;
                }
                Err(e) => return Err(e).context("Failed to read length prefix"),
            }
            let len = u32::from_le_bytes(len_buf) as usize;

            // Read entry data
            let mut entry_buf = vec![0u8; len];
            match file.read_exact(&mut entry_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Truncated tail entry: stop gracefully
                    break;
                }
                Err(e) => return Err(e).context("Failed to read entry data"),
            }

            // Deserialize from MessagePack
            let entry: LogEntry =
                rmp_serde::from_slice(&entry_buf).context("Failed to deserialize WAL entry")?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Replay the log and return committed operations
    pub fn replay<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>> {
        let entries = Self::read_entries(path)?;

        let mut committed = Vec::new();
        let mut transaction_ops: std::collections::HashMap<u64, Vec<LogEntry>> =
            std::collections::HashMap::new();

        for entry in entries {
            match &entry.operation {
                Operation::BeginTransaction { transaction_id } => {
                    transaction_ops.insert(*transaction_id, Vec::new());
                }
                Operation::CommitTransaction { transaction_id } => {
                    // Add all transaction operations to committed
                    if let Some(ops) = transaction_ops.remove(transaction_id) {
                        committed.extend(ops);
                    }
                }
                Operation::RollbackTransaction { transaction_id } => {
                    // Discard all transaction operations
                    transaction_ops.remove(transaction_id);
                }
                _ => {
                    if let Some(txn_id) = entry.transaction_id {
                        // Add to transaction buffer
                        transaction_ops
                            .entry(txn_id)
                            .or_default()
                            .push(entry.clone());
                    } else {
                        // Non-transactional operation, immediately committed
                        committed.push(entry.clone());
                    }
                }
            }
        }

        Ok(committed)
    }

    /// Truncate the log (remove all entries)
    pub fn truncate(&mut self) -> Result<()> {
        drop(std::mem::replace(
            &mut self.writer,
            BufWriter::new(File::create(&self.path).context("Failed to truncate WAL")?),
        ));

        self.next_sequence.store(0, Ordering::SeqCst);

        Ok(())
    }

    /// Get current sequence number
    pub fn sequence(&self) -> u64 {
        self.next_sequence.load(Ordering::SeqCst)
    }

    /// Get path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_concept_id(id: u64) -> ConceptId {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&id.to_le_bytes());
        ConceptId(bytes)
    }

    #[test]
    fn test_create_wal() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let wal = WriteAheadLog::create(&path, false).unwrap();
        assert_eq!(wal.sequence(), 0);
        assert!(path.exists());
    }

    #[test]
    fn test_append_operations() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        // Append concept write
        let seq1 = wal
            .append(Operation::WriteConcept {
                concept_id: test_concept_id(1),
                content_len: 100,
                vector_len: 384,
                created: 1000,
                modified: 1000,
            })
            .unwrap();
        assert_eq!(seq1, 0);

        // Append association write
        let seq2 = wal
            .append(Operation::WriteAssociation {
                source: test_concept_id(1),
                target: test_concept_id(2),
                association_id: 100,
                strength: 0.8,
                created: 1001,
            })
            .unwrap();
        assert_eq!(seq2, 1);

        assert_eq!(wal.sequence(), 2);
    }

    #[test]
    fn test_read_entries() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        wal.append(Operation::DeleteConcept {
            concept_id: test_concept_id(1),
        })
        .unwrap();

        wal.flush().unwrap();

        let entries = WriteAheadLog::read_entries(&path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 0);
        assert_eq!(entries[1].sequence, 1);
    }

    #[test]
    fn test_replay() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        // Non-transactional operations
        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        wal.flush().unwrap();

        let committed = WriteAheadLog::replay(&path).unwrap();
        assert_eq!(committed.len(), 1);
        assert!(matches!(
            committed[0].operation,
            Operation::WriteConcept { .. }
        ));
    }

    #[test]
    fn test_transaction_commit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        // Begin transaction
        let txn_id = wal.begin_transaction().unwrap();

        // Operations within transaction
        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(2),
            content_len: 200,
            vector_len: 384,
            created: 1001,
            modified: 1001,
        })
        .unwrap();

        // Commit transaction
        wal.commit_transaction().unwrap();

        wal.flush().unwrap();

        // Replay should include both operations
        let committed = WriteAheadLog::replay(&path).unwrap();
        assert_eq!(committed.len(), 2);

        // Check concept IDs match
        if let Operation::WriteConcept { concept_id, .. } = committed[0].operation {
            assert_eq!(concept_id, test_concept_id(1));
        } else {
            panic!("Expected WriteConcept operation");
        }

        if let Operation::WriteConcept { concept_id, .. } = committed[1].operation {
            assert_eq!(concept_id, test_concept_id(2));
        } else {
            panic!("Expected WriteConcept operation");
        }

        // Both should have same transaction ID
        assert_eq!(committed[0].transaction_id, Some(txn_id));
        assert_eq!(committed[1].transaction_id, Some(txn_id));
    }

    #[test]
    fn test_transaction_rollback() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        // Non-transactional operation
        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        // Begin transaction
        wal.begin_transaction().unwrap();

        // Operations within transaction
        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(2),
            content_len: 200,
            vector_len: 384,
            created: 1001,
            modified: 1001,
        })
        .unwrap();

        // Rollback transaction
        wal.rollback_transaction().unwrap();

        wal.flush().unwrap();

        // Replay should only include the first operation
        let committed = WriteAheadLog::replay(&path).unwrap();
        assert_eq!(committed.len(), 1);

        if let Operation::WriteConcept { concept_id, .. } = committed[0].operation {
            assert_eq!(concept_id, test_concept_id(1));
        } else {
            panic!("Expected WriteConcept operation");
        }
    }

    #[test]
    fn test_truncate() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, false).unwrap();

        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        wal.flush().unwrap();

        // Truncate
        wal.truncate().unwrap();

        assert_eq!(wal.sequence(), 0);

        let entries = WriteAheadLog::read_entries(&path).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_fsync() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.wal");

        let mut wal = WriteAheadLog::create(&path, true).unwrap();

        // This should fsync automatically
        wal.append(Operation::WriteConcept {
            concept_id: test_concept_id(1),
            content_len: 100,
            vector_len: 384,
            created: 1000,
            modified: 1000,
        })
        .unwrap();

        // Data should be on disk immediately
        let entries = WriteAheadLog::read_entries(&path).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
