#![allow(non_local_definitions)]

/// Simplified Python bindings for Sutra Storage Engine
/// 
/// This is a minimal working version to establish the Python interface.
/// Full integration will be completed incrementally.

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::types::{PyDict, PyList};
use numpy::{PyArray1, PyReadonlyArray1};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::types::{ConceptId, AssociationType};
use crate::vectors::{VectorStore, VectorConfig};
use crate::index::GraphIndex;
use crate::reasoning_store::{ReasoningStore, ConceptData, AssociationData};

// =============================================================================
// Error Handling
// =============================================================================

#[derive(Debug)]
struct StorageError(String);

impl From<StorageError> for PyErr {
    fn from(err: StorageError) -> PyErr {
        PyException::new_err(format!("Storage error: {}", err.0))
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> StorageError {
        StorageError(format!("IO error: {}", err))
    }
}

impl From<anyhow::Error> for StorageError {
    fn from(err: anyhow::Error) -> StorageError {
        StorageError(format!("{}", err))
    }
}

// =============================================================================
// Python GraphStore
// =============================================================================

/// High-level Python interface to the storage engine
#[pyclass(name = "GraphStore")]
struct PyGraphStore {
    vectors: Arc<Mutex<VectorStore>>,
    index: Arc<Mutex<GraphIndex>>,
    #[allow(dead_code)]
    path: PathBuf,
}

#[pymethods]
impl PyGraphStore {
    #[new]
    #[pyo3(signature = (path, vector_dimension=384, use_compression=true))]
    fn new(
        path: &str,
        vector_dimension: usize,
        use_compression: bool,
    ) -> PyResult<Self> {
        let path_buf = PathBuf::from(path);
        
        // Create directory
        std::fs::create_dir_all(&path_buf)
            .map_err(|e| StorageError(format!("Failed to create directory: {}", e)))?;
        
        // Initialize vector store - load if exists, create if new
        let vectors_path = path_buf.join("vectors");
        let vectors = Arc::new(Mutex::new(
            if vectors_path.join("config.json").exists() {
                // Load existing vector store
                VectorStore::load(&vectors_path)
                    .map_err(|e| StorageError(format!("Failed to load vector store: {}", e)))?
            } else {
                // Create new vector store
                let vector_config = VectorConfig {
                    dimension: vector_dimension,
                    use_compression,
                    num_subvectors: vector_dimension / 8,
                    num_centroids: 256,
                };
                VectorStore::new(&vectors_path, vector_config)
                    .map_err(|e| StorageError(format!("Failed to create vector store: {}", e)))?
            }
        ));        // Initialize index
        let index = Arc::new(Mutex::new(GraphIndex::new()));
        
        Ok(Self {
            vectors,
            index,
            path: path_buf,
        })
    }
    
    // -------------------------------------------------------------------------
    // Vector Operations
    // -------------------------------------------------------------------------
    
    /// Add a vector for a concept
    fn add_vector(
        &self,
        concept_id: &str,
        vector: PyReadonlyArray1<f32>,
    ) -> PyResult<()> {
        let id = ConceptId::from_string(concept_id);
        let vec = vector.as_slice()?.to_vec();
        
        let vectors = self.vectors.lock().unwrap();
        vectors.add_vector(id, vec)
            .map_err(|e| StorageError(format!("Failed to add vector: {}", e)))?;
        
        Ok(())
    }
    
    /// Get vector for a concept
    fn get_vector<'py>(
        &self,
        concept_id: &str,
        py: Python<'py>,
    ) -> PyResult<Option<&'py PyArray1<f32>>> {
        let id = ConceptId::from_string(concept_id);
        
        let vectors = self.vectors.lock().unwrap();
        match vectors.get_vector(id) {
            Some(vec) => Ok(Some(PyArray1::from_vec(py, vec))),
            None => Ok(None),
        }
    }
    
    /// Remove a vector
    fn remove_vector(&self, concept_id: &str) -> PyResult<()> {
        let id = ConceptId::from_string(concept_id);
        
        let vectors = self.vectors.lock().unwrap();
        vectors.remove_vector(id)
            .map_err(|e| StorageError(format!("Failed to remove vector: {}", e)))?;
        
        Ok(())
    }
    
    /// Compute exact distance between two vectors
    fn distance(&self, concept_id1: &str, concept_id2: &str) -> PyResult<f32> {
        let id1 = ConceptId::from_string(concept_id1);
        let id2 = ConceptId::from_string(concept_id2);
        
        let vectors = self.vectors.lock().unwrap();
        vectors.distance(id1, id2)
            .map_err(|e| StorageError(format!("Failed to compute distance: {}", e)).into())
    }
    
    /// Compute approximate distance (using quantization)
    fn approximate_distance(&self, concept_id1: &str, concept_id2: &str) -> PyResult<f32> {
        let id1 = ConceptId::from_string(concept_id1);
        let id2 = ConceptId::from_string(concept_id2);
        
        let vectors = self.vectors.lock().unwrap();
        vectors.approximate_distance(id1, id2)
            .map_err(|e| StorageError(format!("Failed to compute approximate distance: {}", e)).into())
    }
    
    /// Train vector quantizer
    fn train_quantizer(&self, training_vectors: Vec<PyReadonlyArray1<f32>>) -> PyResult<()> {
        let vectors_data: Result<Vec<Vec<f32>>, _> = training_vectors
            .iter()
            .map(|arr| arr.as_slice().map(|s| s.to_vec()))
            .collect();
        
        let vectors_data = vectors_data?;
        
        let vectors = self.vectors.lock().unwrap();
        vectors.train_quantizer(Some(vectors_data))
            .map_err(|e| StorageError(format!("Failed to train quantizer: {}", e)))?;
        
        Ok(())
    }
    
    // -------------------------------------------------------------------------
    // Index Operations
    // -------------------------------------------------------------------------
    
    /// Add association between concepts
    fn add_association(&self, source_id: &str, target_id: &str) -> PyResult<()> {
        let source = ConceptId::from_string(source_id);
        let target = ConceptId::from_string(target_id);
        
        let index = self.index.lock().unwrap();
        index.add_edge(source, target);
        
        Ok(())
    }
    
    /// Get neighbors of a concept
    fn get_neighbors(&self, concept_id: &str, py: Python) -> PyResult<PyObject> {
        let id = ConceptId::from_string(concept_id);
        
        let index = self.index.lock().unwrap();
        let neighbors: Vec<String> = index.get_neighbors(id)
            .into_iter()
            .map(|id| id.to_hex())
            .collect();
        
        Ok(PyList::new(py, neighbors).into())
    }
    
    /// Search by text
    fn search_text(&self, query: &str, py: Python) -> PyResult<PyObject> {
        let words: Vec<String> = query.split_whitespace().map(|s| s.to_lowercase()).collect();
        
        let index = self.index.lock().unwrap();
        let results: Vec<String> = index.search_by_words(&words)
            .into_iter()
            .map(|id| id.to_hex())
            .collect();
        
        Ok(PyList::new(py, results).into())
    }
    
    /// Get concepts in time range
    fn get_concepts_in_range(&self, start: u64, end: u64, py: Python) -> PyResult<PyObject> {
        let index = self.index.lock().unwrap();
        let results: Vec<String> = index.query_time_range(start, end)
            .into_iter()
            .map(|id| id.to_hex())
            .collect();
        
        Ok(PyList::new(py, results).into())
    }
    
    // -------------------------------------------------------------------------
    // Maintenance
    // -------------------------------------------------------------------------
    
    /// Save all data to disk
    fn save(&self) -> PyResult<()> {
        let vectors = self.vectors.lock().unwrap();
        vectors.save()
            .map_err(|e| StorageError(format!("Failed to save: {}", e)))?;
        
        Ok(())
    }
    
    /// Get storage statistics
    fn stats(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        
        let vectors = self.vectors.lock().unwrap();
        let vec_stats = vectors.stats();
        
        dict.set_item("total_vectors", vec_stats.total_vectors)?;
        dict.set_item("compressed_vectors", vec_stats.compressed_vectors)?;
        dict.set_item("dimension", vec_stats.dimension)?;
        dict.set_item("raw_size_bytes", vec_stats.raw_size_bytes)?;
        dict.set_item("compressed_size_bytes", vec_stats.compressed_size_bytes)?;
        dict.set_item("compression_ratio", vec_stats.compression_ratio)?;
        dict.set_item("quantizer_trained", vec_stats.quantizer_trained)?;
        
        let index = self.index.lock().unwrap();
        let index_stats = index.stats();
        dict.set_item("total_concepts", index_stats.total_concepts)?;
        dict.set_item("total_edges", index_stats.total_edges)?;
        dict.set_item("total_words", index_stats.total_words)?;
        
        Ok(dict.into())
    }
    
    // -------------------------------------------------------------------------
    // Context Manager
    // -------------------------------------------------------------------------
    
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    
    fn __exit__(
        &self,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<bool> {
        self.save()?;
        Ok(false)
    }
}

// =============================================================================
// Module Definition
// =============================================================================

#[pymodule]
fn sutra_storage(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGraphStore>()?;
    m.add_class::<PyReasoningStore>()?;
    m.add_class::<crate::python_concurrent::PyConcurrentStorage>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

// =============================================================================
// Python ReasoningStore (New API)
// =============================================================================

#[pyclass(name = "ReasoningStore")]
pub struct PyReasoningStore {
    store: Arc<Mutex<ReasoningStore>>,    
}

#[pymethods]
impl PyReasoningStore {
    #[new]
    #[pyo3(signature = (path, vector_dimension=None))]
    fn new(path: &str, vector_dimension: Option<usize>) -> PyResult<Self> {
        let store = ReasoningStore::new(path, vector_dimension)
            .map_err(|e| PyException::new_err(format!("{}", e)))?;
        Ok(Self { store: Arc::new(Mutex::new(store)) })
    }

    // Concepts
    fn put_concept(&self, concept: &PyDict, embedding: Option<PyReadonlyArray1<f32>>) -> PyResult<()> {
        let id_str: String = concept.get_item("id")?
            .ok_or_else(|| PyException::new_err("Missing 'id'"))?
            .extract()?;
        let concept_id = ConceptId::from_string(&id_str);

        let content: String = concept.get_item("content")?
            .ok_or_else(|| PyException::new_err("Missing 'content'"))?
            .extract()?;
        let created: u64 = concept.get_item("created")?
            .ok_or_else(|| PyException::new_err("Missing 'created'"))?
            .extract()?;
        let last_accessed: u64 = concept.get_item("last_accessed")?
            .ok_or_else(|| PyException::new_err("Missing 'last_accessed'"))?
            .extract()?;
        let access_count: u32 = concept.get_item("access_count")?
            .ok_or_else(|| PyException::new_err("Missing 'access_count'"))?
            .extract()?;
        let strength: f32 = concept.get_item("strength")?
            .ok_or_else(|| PyException::new_err("Missing 'strength'"))?
            .extract()?;
        let confidence: f32 = concept.get_item("confidence")?
            .ok_or_else(|| PyException::new_err("Missing 'confidence'"))?
            .extract()?;
        let source: Option<String> = concept.get_item("source")?
            .map(|o| o.extract()).transpose()?.flatten();
        let category: Option<String> = concept.get_item("category")?
            .map(|o| o.extract()).transpose()?.flatten();

        let data = ConceptData {
            id: concept_id,
            content,
            created,
            last_accessed,
            access_count,
            strength,
            confidence,
            source,
            category,
        };

        let emb = embedding.map(|e| e.as_slice().unwrap().to_vec());
        self.store.lock().unwrap().put_concept(data, emb)
            .map_err(|e| PyException::new_err(format!("{}", e)))
    }

    fn get_concept(&self, concept_id: &str, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let id = ConceptId::from_string(concept_id);
        if let Some(c) = self.store.lock().unwrap().get_concept(id) {
            let dict = PyDict::new(py);
            dict.set_item("id", c.id.to_hex())?;
            dict.set_item("content", c.content)?;
            dict.set_item("created", c.created)?;
            dict.set_item("last_accessed", c.last_accessed)?;
            dict.set_item("access_count", c.access_count)?;
            dict.set_item("strength", c.strength)?;
            dict.set_item("confidence", c.confidence)?;
            dict.set_item("source", c.source)?;
            dict.set_item("category", c.category)?;
            Ok(Some(dict.into()))
        } else {
            Ok(None)
        }
    }

    fn get_all_concept_ids(&self, py: Python<'_>) -> PyResult<PyObject> {
        let ids = self.store.lock().unwrap().get_all_concept_ids();
        let ids_str: Vec<String> = ids.iter().map(|id| id.to_hex()).collect();
        Ok(PyList::new(py, ids_str).into())
    }

    // Associations
    fn put_association(&self, assoc: &PyDict) -> PyResult<()> {
        let source: String = assoc.get_item("source_id")?
            .ok_or_else(|| PyException::new_err("Missing 'source_id'"))?
            .extract()?;
        let target: String = assoc.get_item("target_id")?
            .ok_or_else(|| PyException::new_err("Missing 'target_id'"))?
            .extract()?;
        let assoc_type_val: u8 = assoc.get_item("assoc_type")?
            .ok_or_else(|| PyException::new_err("Missing 'assoc_type'"))?
            .extract()?;
        let weight: f32 = assoc.get_item("weight")?
            .ok_or_else(|| PyException::new_err("Missing 'weight'"))?
            .extract()?;
        let confidence: f32 = assoc.get_item("confidence")?
            .ok_or_else(|| PyException::new_err("Missing 'confidence'"))?
            .extract()?;
        let created: u64 = assoc.get_item("created")?
            .ok_or_else(|| PyException::new_err("Missing 'created'"))?
            .extract()?;
        let last_used: u64 = assoc.get_item("last_used")?
            .ok_or_else(|| PyException::new_err("Missing 'last_used'"))?
            .extract()?;

        let data = AssociationData {
            source_id: ConceptId::from_string(&source),
            target_id: ConceptId::from_string(&target),
            assoc_type: AssociationType::from_u8(assoc_type_val).unwrap_or(AssociationType::Semantic),
            weight,
            confidence,
            created,
            last_used,
        };

        self.store.lock().unwrap().put_association(data)
            .map_err(|e| PyException::new_err(format!("{}", e)))
    }

    /// Atomically learn a concept (with optional embedding) and a list of associations
    #[pyo3(signature = (concept, associations, embedding=None))]
    fn learn_atomic(&self, concept: &PyDict, associations: &PyList, embedding: Option<PyReadonlyArray1<f32>>) -> PyResult<()> {
        // Parse concept
        let id_str: String = concept.get_item("id")?
            .ok_or_else(|| PyException::new_err("Missing 'id'"))?
            .extract()?;
        let concept_id = ConceptId::from_string(&id_str);
        let content: String = concept.get_item("content")?
            .ok_or_else(|| PyException::new_err("Missing 'content'"))?
            .extract()?;
        let created: u64 = concept.get_item("created")?
            .ok_or_else(|| PyException::new_err("Missing 'created'"))?
            .extract()?;
        let last_accessed: u64 = concept.get_item("last_accessed")?
            .ok_or_else(|| PyException::new_err("Missing 'last_accessed'"))?
            .extract()?;
        let access_count: u32 = concept.get_item("access_count")?
            .ok_or_else(|| PyException::new_err("Missing 'access_count'"))?
            .extract()?;
        let strength: f32 = concept.get_item("strength")?
            .ok_or_else(|| PyException::new_err("Missing 'strength'"))?
            .extract()?;
        let confidence: f32 = concept.get_item("confidence")?
            .ok_or_else(|| PyException::new_err("Missing 'confidence'"))?
            .extract()?;
        let source: Option<String> = concept.get_item("source")?
            .map(|o| o.extract()).transpose()?.flatten();
        let category: Option<String> = concept.get_item("category")?
            .map(|o| o.extract()).transpose()?.flatten();

        let concept_data = ConceptData {
            id: concept_id,
            content,
            created,
            last_accessed,
            access_count,
            strength,
            confidence,
            source,
            category,
        };

        // Parse associations
        let mut assoc_vec: Vec<AssociationData> = Vec::new();
        for item in associations.iter() {
            let a: &PyDict = item.downcast()?;
            let source: String = a.get_item("source_id")?
                .ok_or_else(|| PyException::new_err("Missing 'source_id'"))?
                .extract()?;
            let target: String = a.get_item("target_id")?
                .ok_or_else(|| PyException::new_err("Missing 'target_id'"))?
                .extract()?;
            let assoc_type_val: u8 = a.get_item("assoc_type")?
                .ok_or_else(|| PyException::new_err("Missing 'assoc_type'"))?
                .extract()?;
            let weight: f32 = a.get_item("weight")?
                .ok_or_else(|| PyException::new_err("Missing 'weight'"))?
                .extract()?;
            let conf: f32 = a.get_item("confidence")?
                .ok_or_else(|| PyException::new_err("Missing 'confidence'"))?
                .extract()?;
            let created: u64 = a.get_item("created")?
                .ok_or_else(|| PyException::new_err("Missing 'created'"))?
                .extract()?;
            let last_used: u64 = a.get_item("last_used")?
                .ok_or_else(|| PyException::new_err("Missing 'last_used'"))?
                .extract()?;

            assoc_vec.push(AssociationData {
                source_id: ConceptId::from_string(&source),
                target_id: ConceptId::from_string(&target),
                assoc_type: AssociationType::from_u8(assoc_type_val).unwrap_or(AssociationType::Semantic),
                weight,
                confidence: conf,
                created,
                last_used,
            });
        }

        let emb = embedding.map(|e| e.as_slice().unwrap().to_vec());
        self.store.lock().unwrap().learn_atomic(concept_data, emb, assoc_vec)
            .map_err(|e| PyException::new_err(format!("{}", e)))
    }

    /// Native pathfinding returning list of paths
    fn find_paths(&self, source_id: &str, target_id: &str, max_depth: usize, max_paths: usize, py: Python<'_>) -> PyResult<PyObject> {
        let source = ConceptId::from_string(source_id);
        let target = ConceptId::from_string(target_id);
        let paths = self.store.lock().unwrap().find_paths(source, target, max_depth, max_paths);
        let out: Vec<PyObject> = paths.into_iter().map(|p| {
            let dict = PyDict::new(py);
            let concepts: Vec<String> = p.concepts.into_iter().map(|id| id.to_hex()).collect();
            let edges: Vec<PyObject> = p.edges.into_iter().map(|(s,t,ty)| {
                let tuple = (s.to_hex(), t.to_hex(), ty as u8);
                tuple.into_py(py)
            }).collect();
            dict.set_item("concepts", PyList::new(py, concepts)).unwrap();
            dict.set_item("edges", PyList::new(py, edges)).unwrap();
            dict.set_item("confidence", p.confidence).unwrap();
            dict.into()
        }).collect();
        Ok(PyList::new(py, out).into())
    }

    fn get_all_associations(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = self.store.lock().unwrap().get_all_associations();
        let out: Vec<PyObject> = list.into_iter().map(|a| {
            let dict = PyDict::new(py);
            dict.set_item("source_id", a.source_id.to_hex()).unwrap();
            dict.set_item("target_id", a.target_id.to_hex()).unwrap();
            dict.set_item("assoc_type", a.assoc_type as u8).unwrap();
            dict.set_item("weight", a.weight).unwrap();
            dict.set_item("confidence", a.confidence).unwrap();
            dict.set_item("created", a.created).unwrap();
            dict.set_item("last_used", a.last_used).unwrap();
            dict.into()
        }).collect();
        Ok(PyList::new(py, out).into())
    }

    fn get_associations_from(&self, concept_id: &str, py: Python<'_>) -> PyResult<PyObject> {
        let id = ConceptId::from_string(concept_id);
        let list = self.store.lock().unwrap().get_associations_from(id);
        let out: Vec<PyObject> = list.into_iter().map(|a| {
            let dict = PyDict::new(py);
            dict.set_item("source_id", a.source_id.to_hex()).unwrap();
            dict.set_item("target_id", a.target_id.to_hex()).unwrap();
            dict.set_item("assoc_type", a.assoc_type as u8).unwrap();
            dict.set_item("weight", a.weight).unwrap();
            dict.set_item("confidence", a.confidence).unwrap();
            dict.set_item("created", a.created).unwrap();
            dict.set_item("last_used", a.last_used).unwrap();
            dict.into()
        }).collect();
        Ok(PyList::new(py, out).into())
    }

    // Graph queries
    fn get_neighbors(&self, concept_id: &str, py: Python<'_>) -> PyResult<PyObject> {
        let id = ConceptId::from_string(concept_id);
        let neighbors = self.store.lock().unwrap().get_neighbors(id);
        let neighbors_hex: Vec<String> = neighbors.into_iter().map(|id| id.to_hex()).collect();
        Ok(PyList::new(py, neighbors_hex).into())
    }

    fn search_by_text(&self, query: &str, py: Python<'_>) -> PyResult<PyObject> {
        let ids = self.store.lock().unwrap().search_by_text(query);
        let ids_hex: Vec<String> = ids.into_iter().map(|id| id.to_hex()).collect();
        Ok(PyList::new(py, ids_hex).into())
    }

    // Lifecycle
    fn flush(&self) -> PyResult<()> {
        self.store.lock().unwrap().flush()
            .map_err(|e| PyException::new_err(format!("{}", e)))
    }

    fn stats(&self, py: Python<'_>) -> PyResult<PyObject> {
        let stats = self.store.lock().unwrap().stats();
        let dict = PyDict::new(py);
        for (k, v) in stats.into_iter() {
            dict.set_item(k, v)?;
        }
        Ok(dict.into())
    }
}
