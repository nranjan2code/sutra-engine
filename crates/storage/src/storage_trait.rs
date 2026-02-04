use crate::semantic::SemanticMetadata;
use crate::types::{AssociationType, ConceptId};
/// Storage trait for unified learning pipeline
///
/// Both ConcurrentMemory and ShardedStorage implement this trait,
/// allowing the learning pipeline to work with either storage backend.
use anyhow::Result;

/// Common storage operations needed by learning pipeline
pub trait LearningStorage {
    /// Store a concept with optional embedding and semantic metadata
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    ) -> Result<u64>;

    /// Store a concept with semantic metadata (🔥 NEW)
    fn learn_concept_with_semantic(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        _semantic: SemanticMetadata,
    ) -> Result<u64> {
        // Default implementation: ignore semantic for backward compat
        // Implementations should override this
        self.learn_concept(
            id,
            content,
            vector,
            strength,
            confidence,
            std::collections::HashMap::new(),
        )
    }

    /// Create an association between concepts
    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64>;

    /// Search for concepts similar to query vector
    fn vector_search(&self, vector: &[f32], k: usize, ef_search: usize) -> Vec<(ConceptId, f32)>;
}

// Implement for ConcurrentMemory
impl LearningStorage for crate::concurrent_memory::ConcurrentMemory {
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    ) -> Result<u64> {
        self.learn_concept(id, content, vector, strength, confidence, attributes)
            .map_err(|e| anyhow::anyhow!("WriteLog error: {:?}", e))
    }

    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64> {
        self.learn_association(source, target, assoc_type, confidence)
            .map_err(|e| anyhow::anyhow!("WriteLog error: {:?}", e))
    }

    fn learn_concept_with_semantic(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        semantic: SemanticMetadata,
    ) -> Result<u64> {
        self.learn_concept_with_semantic(id, content, vector, strength, confidence, semantic)
            .map_err(|e| anyhow::anyhow!("WriteLog error: {:?}", e))
    }

    fn vector_search(&self, vector: &[f32], k: usize, ef_search: usize) -> Vec<(ConceptId, f32)> {
        // Disambiguate call to inherent method to avoid recursion
        crate::concurrent_memory::ConcurrentMemory::vector_search(self, vector, k, ef_search)
    }
}

// Implement for ShardedStorage
impl LearningStorage for crate::sharded_storage::ShardedStorage {
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    ) -> Result<u64> {
        self.learn_concept(id, content, vector, strength, confidence, attributes)
    }

    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64> {
        self.learn_association(source, target, assoc_type, confidence)
    }

    fn learn_concept_with_semantic(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        semantic: SemanticMetadata,
    ) -> Result<u64> {
        crate::sharded_storage::ShardedStorage::learn_concept_with_semantic(
            self,
            id,
            content,
            vector,
            strength,
            confidence,
            semantic,
        )
    }

    fn vector_search(&self, vector: &[f32], k: usize, _ef_search: usize) -> Vec<(ConceptId, f32)> {
        // ShardedStorage uses semantic_search as its inherent vector search implementation
        self.semantic_search(vector.to_vec(), k)
    }
}

// Blanket impl for Arc<T> where T: LearningStorage
impl<T: LearningStorage> LearningStorage for std::sync::Arc<T> {
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    ) -> Result<u64> {
        (**self).learn_concept(id, content, vector, strength, confidence, attributes)
    }

    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64> {
        (**self).learn_association(source, target, assoc_type, confidence)
    }

    fn vector_search(&self, vector: &[f32], k: usize, ef_search: usize) -> Vec<(ConceptId, f32)> {
        (**self).vector_search(vector, k, ef_search)
    }
}
