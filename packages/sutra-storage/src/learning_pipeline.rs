//! Unified learning pipeline: embedding + association extraction + storage
//! This orchestrates the complete learning flow inside the storage server.

use anyhow::Result;
use tracing::{info, warn, debug};

use crate::association_extractor::{AssociationExtractor, AssociationExtractorConfig, AssocKind};
use crate::embedding_client::EmbeddingClient;
use crate::concurrent_memory::ConcurrentMemory;
use crate::types::{ConceptId, AssociationType};

#[derive(Debug, Clone)]
pub struct LearnOptions {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}

impl Default for LearnOptions {
    fn default() -> Self {
        Self {
            generate_embedding: true,
            embedding_model: None,
            extract_associations: true,
            min_association_confidence: std::env::var("SUTRA_MIN_ASSOCIATION_CONFIDENCE").ok().and_then(|s| s.parse().ok()).unwrap_or(0.5),
            max_associations_per_concept: std::env::var("SUTRA_MAX_ASSOCIATIONS_PER_CONCEPT").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
            strength: 1.0,
            confidence: 1.0,
        }
    }
}

pub struct LearningPipeline {
    embedding_client: EmbeddingClient,
    assoc_extractor: AssociationExtractor,
}

impl LearningPipeline {
    pub fn new() -> Result<Self> {
        let embedding_client = EmbeddingClient::with_defaults()?;
        let assoc_extractor = AssociationExtractor::new(AssociationExtractorConfig::default())?;
        Ok(Self { embedding_client, assoc_extractor })
    }

    /// Learn a single concept end-to-end
    pub async fn learn_concept(
        &self,
        storage: &ConcurrentMemory,
        content: &str,
        options: &LearnOptions,
    ) -> Result<String> {
        info!("LearningPipeline: learn_concept (len={})", content.len());

        // Step 1: Embedding
        let embedding_opt = if options.generate_embedding {
            match self.embedding_client.generate(content, true).await {
                Ok(vec) => Some(vec),
                Err(e) => { warn!("Embedding failed, continuing without: {}", e); None }
            }
        } else { None };

        // Step 2: Generate ID
        let concept_id = self.generate_concept_id(content);
        let id = ConceptId::from_string(&concept_id);

        // Step 3: Store concept
        let sequence = storage.learn_concept(
            id,
            content.as_bytes().to_vec(),
            embedding_opt.clone(),
            options.strength,
            options.confidence,
        )?;
        debug!("Stored concept seq={}", sequence);

        // Step 4: Associations
        if options.extract_associations {
            let extracted = self.assoc_extractor.extract(content)?;
            let mut stored = 0usize;
            for assoc in extracted.into_iter().take(options.max_associations_per_concept) {
                // Map target term to concept id (deterministic)
                let target_id_hex = self.generate_concept_id(&assoc.target_term);
                let target_id = ConceptId::from_string(&target_id_hex);

                let assoc_type = match assoc.kind {
                    AssocKind::Semantic => AssociationType::Semantic,
                    AssocKind::Causal => AssociationType::Causal,
                    AssocKind::Temporal => AssociationType::Temporal,
                    AssocKind::Hierarchical => AssociationType::Hierarchical,
                    AssocKind::Compositional => AssociationType::Compositional,
                };

                if let Err(e) = storage.learn_association(id, target_id, assoc_type, assoc.confidence) {
                    warn!("Association store failed: {}", e);
                } else {
                    stored += 1;
                }
            }
            debug!("Stored {} associations", stored);
        }

        Ok(concept_id)
    }

    /// Learn concepts in batch with basic optimizations
    pub async fn learn_batch(
        &self,
        storage: &ConcurrentMemory,
        contents: &[String],
        options: &LearnOptions,
    ) -> Result<Vec<String>> {
        info!("LearningPipeline: learn_batch count={}", contents.len());

        // Batch embeddings first to reduce overhead
        let embeddings: Vec<Option<Vec<f32>>> = if options.generate_embedding {
            self.embedding_client.generate_batch(contents, true).await
        } else {
            vec![None; contents.len()]
        };

        let mut concept_ids = Vec::with_capacity(contents.len());
        for (i, (content, embedding_opt)) in contents.iter().zip(embeddings.into_iter()).enumerate() {
            if let Some(ref emb) = embedding_opt {
                info!("💡 Concept {}: embedding dimension = {}", i, emb.len());
            } else {
                warn!("⚠️  Concept {}: NO EMBEDDING", i);
            }
            // Generate ID
            let concept_id = self.generate_concept_id(content);
            let id = ConceptId::from_string(&concept_id);

            // Store concept with pre-computed embedding
            let sequence = storage.learn_concept(
                id,
                content.as_bytes().to_vec(),
                embedding_opt.clone(),
                options.strength,
                options.confidence,
            )?;
            debug!("Stored concept seq={}", sequence);

            // Extract and store associations
            if options.extract_associations {
                let extracted = self.assoc_extractor.extract(content)?;
                let mut stored = 0usize;
                for assoc in extracted.into_iter().take(options.max_associations_per_concept) {
                    let target_id_hex = self.generate_concept_id(&assoc.target_term);
                    let target_id = ConceptId::from_string(&target_id_hex);

                    let assoc_type = match assoc.kind {
                        AssocKind::Semantic => AssociationType::Semantic,
                        AssocKind::Causal => AssociationType::Causal,
                        AssocKind::Temporal => AssociationType::Temporal,
                        AssocKind::Hierarchical => AssociationType::Hierarchical,
                        AssocKind::Compositional => AssociationType::Compositional,
                    };

                    if let Err(e) = storage.learn_association(id, target_id, assoc_type, assoc.confidence) {
                        warn!("Association store failed: {}", e);
                    } else {
                        stored += 1;
                    }
                }
                debug!("Stored {} associations", stored);
            }

            concept_ids.push(concept_id);
        }
        Ok(concept_ids)
    }

    fn generate_concept_id(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
