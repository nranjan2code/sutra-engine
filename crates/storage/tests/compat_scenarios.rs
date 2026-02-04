use std::io::Write;

use tempfile::TempDir;

use sutra_storage::{ConceptId, ConcurrentConfig, ConcurrentMemory};

#[test]
fn test_v2_snapshot_compatibility_loads() {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().join("storage.dat");

    let concept_id = ConceptId::from_string("compat-v2-concept");
    let content = b"hello-v2";
    let vector = [0.1_f32, 0.2, 0.3, 0.4];

    let mut file = std::io::BufWriter::new(std::fs::File::create(&storage_path).unwrap());

    // Header (64 bytes): magic + version(2) + counts
    let mut header = vec![0u8; 64];
    header[0..8].copy_from_slice(b"SUTRADAT");
    header[8..12].copy_from_slice(&2u32.to_le_bytes());
    header[12..16].copy_from_slice(&1u32.to_le_bytes()); // concepts
    header[16..20].copy_from_slice(&0u32.to_le_bytes()); // edges
    header[20..24].copy_from_slice(&1u32.to_le_bytes()); // vectors
    file.write_all(&header).unwrap();

    // Concept header (36 bytes) + content
    let mut concept_header = vec![0u8; 36];
    concept_header[0..16].copy_from_slice(&concept_id.0);
    concept_header[16..20].copy_from_slice(&(content.len() as u32).to_le_bytes());
    concept_header[20..24].copy_from_slice(&1.0_f32.to_le_bytes()); // strength
    concept_header[24..28].copy_from_slice(&0.9_f32.to_le_bytes()); // confidence
    concept_header[28..32].copy_from_slice(&0u32.to_le_bytes()); // access_count
    concept_header[32..36].copy_from_slice(&123u32.to_le_bytes()); // created
    file.write_all(&concept_header).unwrap();
    file.write_all(content).unwrap();

    // V2 format has no edges, write vectors directly
    let mut vector_header = vec![0u8; 20];
    vector_header[0..16].copy_from_slice(&concept_id.0);
    vector_header[16..20].copy_from_slice(&(vector.len() as u32).to_le_bytes());
    file.write_all(&vector_header).unwrap();
    for component in vector {
        file.write_all(&component.to_le_bytes()).unwrap();
    }
    file.flush().unwrap();

    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 4,
        memory_threshold: 1000,
        ..Default::default()
    };
    let memory = ConcurrentMemory::new(config);

    let loaded = memory
        .query_concept(&concept_id)
        .expect("v2 snapshot concept should load");
    assert_eq!(&*loaded.content, content);
    assert!(loaded.semantic.is_none());
    assert!(loaded.attributes.is_empty());
    assert_eq!(loaded.vector.as_ref().unwrap().len(), 4);

    memory.shutdown();
}
