//! Unified learning pipeline: embedding + association extraction + storage
//! This orchestrates the complete learning flow inside the storage server.

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::embedding_client::HttpEmbeddingClient;
use crate::embedding_provider::EmbeddingProvider;
use crate::semantic::{SemanticAnalyzer, SemanticMetadata};
use crate::semantic_extractor::SemanticExtractor;
use crate::storage_trait::LearningStorage;
use crate::types::ConceptId;

#[derive(Debug, Clone)]
pub struct LearnOptions {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub analyze_semantics: bool, // 🔥 NEW: Enable semantic analysis
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
            analyze_semantics: true, // 🔥 NEW: Enabled by default
            min_association_confidence: std::env::var("SUTRA_MIN_ASSOCIATION_CONFIDENCE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.5),
            max_associations_per_concept: std::env::var("SUTRA_MAX_ASSOCIATIONS_PER_CONCEPT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            strength: 1.0,
            confidence: 1.0,
        }
    }
}

pub struct LearningPipeline {
    embedding_client: Arc<dyn EmbeddingProvider>,
    semantic_extractor: SemanticExtractor,
    semantic_analyzer: SemanticAnalyzer, // 🔥 NEW: Semantic understanding
}

impl LearningPipeline {
    pub async fn new() -> Result<Self> {
        let embedding_client = Arc::new(HttpEmbeddingClient::with_defaults()?);
        Self::new_with_provider(embedding_client).await
    }

    pub async fn new_with_provider(embedding_client: Arc<dyn EmbeddingProvider>) -> Result<Self> {
        // Initialize semantic extractor (async - pre-computes relation embeddings)
        let semantic_extractor = SemanticExtractor::new(embedding_client.clone()).await?;

        // Initialize semantic analyzer (deterministic, no async)
        let semantic_analyzer = SemanticAnalyzer::new();

        Ok(Self {
            embedding_client,
            semantic_extractor,
            semantic_analyzer,
        })
    }

    /// Analyze semantic metadata for content
    pub fn analyze_semantic(&self, content: &str) -> SemanticMetadata {
        self.semantic_analyzer.analyze(content)
    }

    /// Learn a single concept end-to-end
    pub async fn learn_concept<S: LearningStorage>(
        &self,
        storage: &S,
        content: &str,
        options: &LearnOptions,
    ) -> Result<String> {
        info!("LearningPipeline: learn_concept (len={})", content.len());

        // Step 1: Embedding
        let embedding_opt = if options.generate_embedding {
            match self.embedding_client.generate(content, true).await {
                Ok(vec) => Some(vec),
                Err(e) => {
                    warn!("Embedding failed, continuing without: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Step 2: Generate ID
        let concept_id = self.generate_concept_id(content);
        let id = ConceptId::from_string(&concept_id);

        // Step 3: Analyze semantics (🔥 NEW)
        let semantic = if options.analyze_semantics {
            Some(self.semantic_analyzer.analyze(content))
        } else {
            None
        };

        // Step 4: Store concept with semantic metadata
        let sequence = if let Some(semantic_meta) = semantic {
            info!(
                "💡 Semantic: type={}, domain={:?}, confidence={:.2}",
                semantic_meta.semantic_type,
                semantic_meta.domain_context,
                semantic_meta.classification_confidence
            );
            storage.learn_concept_with_semantic(
                id,
                content.as_bytes().to_vec(),
                embedding_opt.clone(),
                options.strength,
                options.confidence,
                semantic_meta,
            )?
        } else {
            storage.learn_concept(
                id,
                content.as_bytes().to_vec(),
                embedding_opt.clone(),
                options.strength,
                options.confidence,
                std::collections::HashMap::new(),
            )?
        };
        debug!("Stored concept seq={}", sequence);

        // Step 4: Semantic associations (modern approach!)
        if options.extract_associations {
            let extracted = self.semantic_extractor.extract(content).await?;
            let mut stored = 0usize;

            for assoc in extracted
                .into_iter()
                .take(options.max_associations_per_concept)
            {
                // Only store if confidence meets threshold
                if assoc.confidence < options.min_association_confidence {
                    continue;
                }

                // Map target term to concept id (deterministic)
                let target_id_hex = self.generate_concept_id(&assoc.target);
                let target_id = ConceptId::from_string(&target_id_hex);

                if let Err(e) =
                    storage.learn_association(id, target_id, assoc.assoc_type, assoc.confidence)
                {
                    warn!("Association store failed: {}", e);
                } else {
                    stored += 1;
                }
            }

            debug!("Stored {} semantic associations", stored);
        }

        Ok(concept_id)
    }

    /// Learn concepts in batch with basic optimizations
    pub async fn learn_batch<S: LearningStorage>(
        &self,
        storage: &S,
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
        for (i, (content, embedding_opt)) in contents.iter().zip(embeddings.into_iter()).enumerate()
        {
            if let Some(ref emb) = embedding_opt {
                info!("💡 Concept {}: embedding dimension = {}", i, emb.len());
            } else {
                warn!("⚠️  Concept {}: NO EMBEDDING", i);
            }
            // Generate ID
            let concept_id = self.generate_concept_id(content);
            let id = ConceptId::from_string(&concept_id);

            // Analyze semantics (🔥 NEW)
            let semantic = if options.analyze_semantics {
                Some(self.semantic_analyzer.analyze(content))
            } else {
                None
            };

            // Store concept with pre-computed embedding and semantic metadata
            let sequence = if let Some(semantic_meta) = semantic {
                if i < 3 {
                    // Log first 3 for visibility
                    info!(
                        "💡 Batch[{}] Semantic: type={}, domain={:?}",
                        i, semantic_meta.semantic_type, semantic_meta.domain_context
                    );
                }
                storage.learn_concept_with_semantic(
                    id,
                    content.as_bytes().to_vec(),
                    embedding_opt.clone(),
                    options.strength,
                    options.confidence,
                    semantic_meta,
                )?
            } else {
                storage.learn_concept(
                    id,
                    content.as_bytes().to_vec(),
                    embedding_opt.clone(),
                    options.strength,
                    options.confidence,
                    std::collections::HashMap::new(),
                )?
            };
            debug!("Stored concept seq={}", sequence);

            // Extract and store semantic associations
            if options.extract_associations {
                let extracted = self.semantic_extractor.extract(content).await?;
                let mut stored = 0usize;

                for assoc in extracted
                    .into_iter()
                    .take(options.max_associations_per_concept)
                {
                    // Only store if confidence meets threshold
                    if assoc.confidence < options.min_association_confidence {
                        continue;
                    }

                    let target_id_hex = self.generate_concept_id(&assoc.target);
                    let target_id = ConceptId::from_string(&target_id_hex);

                    if let Err(e) =
                        storage.learn_association(id, target_id, assoc.assoc_type, assoc.confidence)
                    {
                        warn!("Association store failed: {}", e);
                    } else {
                        stored += 1;
                    }
                }

                debug!("Stored {} semantic associations", stored);
            }

            concept_ids.push(concept_id);
        }
        Ok(concept_ids)
    }

    fn generate_concept_id(&self, content: &str) -> String {
        let digest = md5::compute(content);
        format!("{:x}", digest)
    }

    /// Search concepts by text using semantic similarity
    pub async fn search<S: LearningStorage>(
        &self,
        storage: &S,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(ConceptId, f32)>> {
        info!("LearningPipeline: search for '{}' (limit={})", query, limit);

        // Step 1: Generate embedding for search query
        let query_vector = self.embedding_client.generate(query, false).await?;

        // Step 2: Perform vector search in storage
        // ef_search is hardcoded to 128 for a good balance of speed and accuracy
        let results = storage.vector_search(&query_vector, limit, 128);

        Ok(results)
    }
}
