use sutra_storage::{ConcurrentConfig, ConcurrentMemory, NamespaceManager, LearningStorage};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_multi_namespace_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();
    
    let manager = NamespaceManager::new(base_path.clone());
    
    // Create two namespaces
    let ns1 = "agent_alpha";
    let ns2 = "agent_beta";
    
    let storage1 = manager.create_namespace(ns1, ConcurrentConfig::default()).unwrap();
    let storage2 = manager.create_namespace(ns2, ConcurrentConfig::default()).unwrap();
    
    // Learn in NS1
    let id1 = sutra_storage::ConceptId::from_string("concept_1");
    storage1.learn_concept(
        id1, 
        b"Data for alpha".to_vec(), 
        None, 
        1.0, 1.0, 
        std::collections::HashMap::new()
    ).unwrap();
    
    // Verify isolation
    assert!(storage1.get_concept(id1).is_some());
    assert!(storage2.get_concept(id1).is_none());
    
    // Learn in NS2
    storage2.learn_concept(
        id1, 
        b"Data for beta".to_vec(), 
        None, 
        1.0, 1.0, 
        std::collections::HashMap::new()
    ).unwrap();
    
    // Verify both have their own data
    let node1 = storage1.get_concept(id1).unwrap();
    let node2 = storage2.get_concept(id1).unwrap();
    
    assert_eq!(node1.content, b"Data for alpha");
    assert_eq!(node2.content, b"Data for beta");
    
    // Test listing namespaces
    let namespaces = manager.list_namespaces();
    assert!(namespaces.contains(&ns1.to_string()));
    assert!(namespaces.contains(&ns2.to_string()));
    
    // Test removal
    manager.remove_namespace(ns1);
    assert!(!manager.list_namespaces().contains(&ns1.to_string()));
}
