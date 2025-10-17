/// Python bindings for ConcurrentMemory (the real new storage)

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use numpy::PyReadonlyArray1;
use std::sync::Arc;

use crate::concurrent_memory::{ConcurrentMemory, ConcurrentConfig};
use crate::types::{ConceptId, AssociationType};

/// Python wrapper for ConcurrentMemory
#[pyclass(name = "ConcurrentStorage")]
pub struct PyConcurrentStorage {
    inner: Arc<ConcurrentMemory>,
}

#[pymethods]
impl PyConcurrentStorage {
    #[new]
    #[pyo3(signature = (path, reconcile_interval_ms=10, memory_threshold=50000, vector_dimension=768))]
    fn new(
        path: &str,
        reconcile_interval_ms: u64,
        memory_threshold: usize,
        vector_dimension: usize,
    ) -> PyResult<Self> {
        let config = ConcurrentConfig {
            storage_path: path.into(),
            reconcile_interval_ms,
            memory_threshold,
            vector_dimension,
        };
        
        let memory = ConcurrentMemory::new(config);
        
        Ok(Self {
            inner: Arc::new(memory),
        })
    }
    
    /// Learn a concept with content and optional embedding
    #[pyo3(signature = (concept_id, content, embedding=None, strength=1.0, confidence=1.0))]
    fn learn_concept(
        &self,
        concept_id: &str,
        content: &str,
        embedding: Option<PyReadonlyArray1<f32>>,
        strength: f32,
        confidence: f32,
    ) -> PyResult<u64> {
        let id = ConceptId::from_string(concept_id);
        let content_vec = content.as_bytes().to_vec();
        let vector = embedding.map(|arr| arr.as_slice().unwrap().to_vec());
        
        self.inner
            .learn_concept(id, content_vec, vector, strength, confidence)
            .map_err(|e| PyException::new_err(format!("{:?}", e)))
    }
    
    /// Learn an association between two concepts
    #[pyo3(signature = (source_id, target_id, assoc_type=0, confidence=1.0))]
    fn learn_association(
        &self,
        source_id: &str,
        target_id: &str,
        assoc_type: u8,
        confidence: f32,
    ) -> PyResult<u64> {
        let source = ConceptId::from_string(source_id);
        let target = ConceptId::from_string(target_id);
        let atype = AssociationType::from_u8(assoc_type).unwrap_or(AssociationType::Semantic);
        
        self.inner
            .learn_association(source, target, atype, confidence)
            .map_err(|e| PyException::new_err(format!("{:?}", e)))
    }
    
    /// Query a concept by ID
    fn query_concept(&self, concept_id: &str, py: Python) -> PyResult<Option<PyObject>> {
        let id = ConceptId::from_string(concept_id);
        
        if let Some(node) = self.inner.query_concept(&id) {
            let dict = PyDict::new(py);
            dict.set_item("id", id.to_hex())?;
            dict.set_item("content", String::from_utf8_lossy(&node.content))?;
            dict.set_item("strength", node.strength)?;
            dict.set_item("confidence", node.confidence)?;
            Ok(Some(dict.into()))
        } else {
            Ok(None)
        }
    }
    
    /// Get neighbors of a concept
    fn get_neighbors(&self, concept_id: &str, py: Python) -> PyResult<PyObject> {
        let id = ConceptId::from_string(concept_id);
        let neighbors = self.inner.query_neighbors(&id);
        let hex_ids: Vec<String> = neighbors.iter().map(|id| id.to_hex()).collect();
        Ok(hex_ids.into_py(py))
    }
    
    /// Find path between two concepts
    #[pyo3(signature = (start_id, end_id, max_depth=6))]
    fn find_path(
        &self,
        start_id: &str,
        end_id: &str,
        max_depth: usize,
        py: Python,
    ) -> PyResult<Option<PyObject>> {
        let start = ConceptId::from_string(start_id);
        let end = ConceptId::from_string(end_id);
        
        if let Some(path) = self.inner.find_path(start, end, max_depth) {
            let hex_path: Vec<String> = path.iter().map(|id| id.to_hex()).collect();
            Ok(Some(hex_path.into_py(py)))
        } else {
            Ok(None)
        }
    }
    
    /// Check if concept exists
    fn contains(&self, concept_id: &str) -> PyResult<bool> {
        let id = ConceptId::from_string(concept_id);
        Ok(self.inner.contains(&id))
    }
    
    /// Vector similarity search (k-NN)
    #[pyo3(signature = (query_vector, k=10, ef_search=50))]
    fn vector_search(
        &self,
        query_vector: PyReadonlyArray1<f32>,
        k: usize,
        ef_search: usize,
        py: Python,
    ) -> PyResult<PyObject> {
        let query = query_vector.as_slice().unwrap();
        let results = self.inner.vector_search(query, k, ef_search);
        
        // Convert to Python list of (concept_id, similarity) tuples
        let py_results: Vec<(String, f32)> = results
            .into_iter()
            .map(|(id, similarity)| (id.to_hex(), similarity))
            .collect();
        
        Ok(py_results.into_py(py))
    }
    
    /// Get statistics
    fn stats(&self, py: Python) -> PyResult<PyObject> {
        let stats = self.inner.stats();
        let dict = PyDict::new(py);
        
        // Write log stats
        dict.set_item("written", stats.write_log.written)?;
        dict.set_item("dropped", stats.write_log.dropped)?;
        dict.set_item("pending", stats.write_log.pending)?;
        
        // Reconciler stats
        dict.set_item("reconciliations", stats.reconciler.reconciliations)?;
        dict.set_item("entries_processed", stats.reconciler.entries_processed)?;
        
        // Snapshot info
        dict.set_item("concepts", stats.snapshot.concept_count)?;
        dict.set_item("edges", stats.snapshot.edge_count)?;
        dict.set_item("sequence", stats.snapshot.sequence)?;
        
        Ok(dict.into())
    }
    
    /// Force flush to disk
    fn flush(&self) -> PyResult<()> {
        self.inner.flush()
            .map_err(|e| PyException::new_err(format!("Flush failed: {:?}", e)))
    }
}

#[pymodule]
fn sutra_storage_concurrent(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyConcurrentStorage>()?;
    Ok(())
}
