/// Storage trait for unified learning pipeline
///
/// Both ConcurrentMemory and ShardedStorage implement this trait,
/// allowing the learning pipeline to work with either storage backend.

use anyhow::Result;
use crate::types::{ConceptId, AssociationType};

/// Common storage operations needed by learning pipeline
pub trait LearningStorage {
    /// Store a concept with optional embedding
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
    ) -> Result<u64>;
    
    /// Create an association between concepts
    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64>;
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
    ) -> Result<u64> {
        self.learn_concept(id, content, vector, strength, confidence)
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
    ) -> Result<u64> {
        self.learn_concept(id, content, vector, strength, confidence)
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
    ) -> Result<u64> {
        (**self).learn_concept(id, content, vector, strength, confidence)
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
}
