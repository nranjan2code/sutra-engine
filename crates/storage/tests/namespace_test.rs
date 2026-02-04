use sutra_storage::{ConcurrentConfig, LearningStorage, NamespaceManager};
use tempfile::TempDir;

#[tokio::test]
async fn test_multi_namespace_isolation() {
    async fn wait_for_concept(
        storage: &std::sync::Arc<sutra_storage::ConcurrentMemory>,
        id: &sutra_storage::ConceptId,
        should_exist: bool,
    ) {
        let timeout = std::time::Duration::from_secs(1);
        let start = std::time::Instant::now();

        loop {
            let exists = storage.query_concept(id).is_some();
            if exists == should_exist {
                break;
            }
            if start.elapsed() > timeout {
                panic!(
                    "timeout waiting for concept {:?} to exist={}",
                    id, should_exist
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();

    let config_template = ConcurrentConfig {
        storage_path: base_path.clone(),
        ..Default::default()
    };
    let manager = NamespaceManager::new(base_path.clone(), config_template).unwrap();

    // Create two namespaces
    let ns1 = "agent_alpha";
    let ns2 = "agent_beta";

    let storage1 = manager.get_namespace(ns1);
    let storage2 = manager.get_namespace(ns2);

    // Learn in NS1
    let id1 = sutra_storage::ConceptId::from_string("concept_1");
    storage1
        .learn_concept(
            id1,
            b"Data for alpha".to_vec(),
            None,
            1.0,
            1.0,
            std::collections::HashMap::new(),
        )
        .unwrap();
    wait_for_concept(&storage1, &id1, true).await;

    // Verify isolation
    assert!(storage1.query_concept(&id1).is_some());
    assert!(storage2.query_concept(&id1).is_none());

    // Learn in NS2
    storage2
        .learn_concept(
            id1,
            b"Data for beta".to_vec(),
            None,
            1.0,
            1.0,
            std::collections::HashMap::new(),
        )
        .unwrap();
    wait_for_concept(&storage2, &id1, true).await;

    // Verify both have their own data
    let node1 = storage1.query_concept(&id1).unwrap();
    let node2 = storage2.query_concept(&id1).unwrap();

    assert_eq!(node1.content.as_ref(), b"Data for alpha");
    assert_eq!(node2.content.as_ref(), b"Data for beta");

    // Test listing namespaces
    let namespaces = manager.list_namespaces();
    assert!(namespaces.contains(&ns1.to_string()));
    assert!(namespaces.contains(&ns2.to_string()));

    // Test clear
    manager.clear_namespace(ns1).unwrap();
    wait_for_concept(&storage1, &id1, false).await;
}
