use std::io::Write;
use tempfile::TempDir;

use sutra_storage::{ConceptId, Operation, WriteAheadLog};

#[test]
fn test_wal_replay_with_truncated_entry_is_handled() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("wal.log");

    let mut wal = WriteAheadLog::create(&wal_path, true).unwrap();
    wal.append(Operation::WriteConcept {
        concept_id: ConceptId::from_string("corrupt-1"),
        content_len: 12,
        vector_len: 0,
        created: 123,
        modified: 123,
    })
    .unwrap();
    wal.flush().unwrap();

    // Append a corrupted partial entry (length prefix but no payload)
    {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .unwrap();
        file.write_all(&1234u32.to_le_bytes()).unwrap();
        file.flush().unwrap();
    }

    // Replay should not panic, and should return committed entries it can parse.
    let entries = WriteAheadLog::replay(&wal_path).unwrap();
    assert!(!entries.is_empty());
}
