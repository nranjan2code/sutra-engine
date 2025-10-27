# Learning Pipeline

The Sutra Learning Pipeline is the core system that transforms raw content into structured knowledge. This unified approach handles embedding generation, association extraction, and concept storage in a single atomic operation, ensuring consistency and optimal performance.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Unified Learning Pipeline                    │
├─────────────────────────────────────────────────────────────────┤
│  Input: Raw Content                                             │
│  ├─ Text documents, procedures, facts                          │
│  ├─ Medical protocols, legal cases                             │
│  └─ Any domain-specific knowledge                              │
├─────────────────────────────────────────────────────────────────┤
│  Stage 1: Content Analysis                                     │
│  ├─ Text normalization and cleaning                           │
│  ├─ Content validation and quality checks                     │
│  └─ Duplicate detection                                       │
├─────────────────────────────────────────────────────────────────┤
│  Stage 2: Embedding Generation                                │
│  ├─ Semantic vector creation (768 dimensions)                 │
│  ├─ Model selection (granite, sentence-transformers)          │
│  └─ Batch processing for efficiency                          │
├─────────────────────────────────────────────────────────────────┤
│  Stage 3: Association Extraction                              │
│  ├─ Find related existing concepts                           │
│  ├─ Calculate semantic similarities                          │
│  ├─ Extract named entities and relationships                 │
│  └─ Apply confidence thresholds                             │
├─────────────────────────────────────────────────────────────────┤
│  Stage 4: Atomic Storage                                      │
│  ├─ Generate deterministic concept ID                        │
│  ├─ Store concept + embedding + associations                 │
│  ├─ Update HNSW vector index                                │
│  └─ Write-Ahead Log for durability                          │
└─────────────────────────────────────────────────────────────────┘
```

## Learning Options

### Core Configuration

```python
# Default learning options (automatically applied)
default_options = {
    # Embedding generation
    "generate_embedding": True,
    "embedding_model": "granite-embedding:30m",  # Default model
    
    # Association extraction  
    "extract_associations": True,
    "min_association_confidence": 0.5,          # Quality threshold
    "max_associations_per_concept": 10,         # Prevent over-linking
    
    # Concept properties
    "strength": 1.0,                            # Importance (0.0-1.0)
    "confidence": 1.0,                          # Reliability (0.0-1.0)
    
    # Processing options
    "batch_embeddings": True,                   # Efficient batch processing
    "async_associations": True,                 # Background association extraction
}
```

### Advanced Configuration

```python
# High-quality medical knowledge
medical_options = {
    "embedding_model": "granite-embedding:30m",
    "extract_associations": True,
    "min_association_confidence": 0.75,         # Higher threshold
    "max_associations_per_concept": 15,         # More connections
    "strength": 0.95,                          # High importance
    "confidence": 0.90,                        # High reliability
}

# Fast bulk loading
bulk_options = {
    "embedding_model": "sentence-transformers/all-MiniLM-L6-v2",  # Faster model
    "extract_associations": False,              # Skip associations for speed
    "min_association_confidence": 0.8,
    "max_associations_per_concept": 5,
    "strength": 0.8,
    "confidence": 0.8,
    "batch_size": 100,                         # Large batches
}

# Regulatory compliance (maximum auditability)
compliance_options = {
    "embedding_model": "granite-embedding:30m",
    "extract_associations": True,
    "min_association_confidence": 0.9,          # Very high threshold
    "max_associations_per_concept": 8,          # Conservative linking
    "strength": 1.0,
    "confidence": 0.95,
    "audit_mode": True,                        # Full provenance tracking
    "require_citations": True,                 # Enforce source attribution
}
```

## Embedding Generation

### Model Selection

Sutra supports multiple embedding models optimized for different use cases:

| Model | Dimensions | Speed | Quality | Best For |
|-------|------------|-------|---------|----------|
| `granite-embedding:30m` | 768 | Fast | High | General knowledge |
| `sentence-transformers/all-MiniLM-L6-v2` | 384 | Very Fast | Good | Bulk loading |
| `sentence-transformers/all-mpnet-base-v2` | 768 | Medium | Very High | Medical/Legal |
| `text-embedding-ada-002` | 1536 | Slow | Excellent | Premium quality |

### Batch Embedding Optimization

```rust
// Rust implementation (server-side)
impl EmbeddingService {
    pub async fn generate_embeddings_batch(
        &self,
        contents: &[String],
        model: &str,
    ) -> Result<Vec<Option<Vec<f32>>>> {
        // Batch size optimization based on model and hardware
        let batch_size = self.optimal_batch_size(model);
        let mut results = Vec::with_capacity(contents.len());
        
        for chunk in contents.chunks(batch_size) {
            let embeddings = self.model_pool
                .get_model(model)?
                .encode_batch(chunk)
                .await?;
            
            results.extend(embeddings);
        }
        
        Ok(results)
    }
    
    fn optimal_batch_size(&self, model: &str) -> usize {
        match model {
            "granite-embedding:30m" => 64,
            "sentence-transformers/all-MiniLM-L6-v2" => 128,
            "text-embedding-ada-002" => 16,
            _ => 32,
        }
    }
}
```

### Custom Embedding Integration

```python
# Using external embedding service
import openai
from sutra_storage_client import StorageClient

class CustomEmbeddingClient:
    def __init__(self, storage_client: StorageClient):
        self.storage = storage_client
        self.openai_client = openai.Client()
    
    def learn_with_openai_embeddings(self, content: str) -> str:
        """Learn concept with OpenAI embeddings."""
        # Generate embedding using OpenAI
        response = self.openai_client.embeddings.create(
            model="text-embedding-ada-002",
            input=content
        )
        embedding = response.data[0].embedding
        
        # Store with pre-computed embedding
        concept_id = self.storage.learn_concept(
            concept_id=self._generate_concept_id(content),
            content=content,
            embedding=embedding,
            strength=0.95,
            confidence=0.90
        )
        
        return concept_id
    
    def _generate_concept_id(self, content: str) -> str:
        import hashlib
        return hashlib.md5(content.encode()).hexdigest()

# Usage
custom_client = CustomEmbeddingClient(client)
concept_id = custom_client.learn_with_openai_embeddings(
    "Advanced cardiac surgery requires specialized training and certification."
)
```

## Association Extraction

### Semantic Association Discovery

The pipeline automatically discovers relationships between concepts:

```rust
// Rust implementation of semantic association extraction
impl SemanticExtractor {
    pub async fn extract_associations(
        &self,
        content: &str,
        existing_concepts: &ConceptIndex,
        options: &LearnOptions,
    ) -> Result<Vec<SemanticAssociation>> {
        let mut associations = Vec::new();
        
        // Step 1: Generate embedding for new content
        let query_embedding = self.embedding_service
            .generate_embedding(content, &options.embedding_model)
            .await?;
        
        // Step 2: Vector search for similar concepts
        let similar_concepts = existing_concepts
            .vector_search(&query_embedding, 50, 100)?; // k=50, ef=100
        
        // Step 3: Filter by confidence threshold
        for (concept_id, similarity) in similar_concepts {
            if similarity >= options.min_association_confidence {
                let assoc_type = self.classify_relationship(
                    content, 
                    &existing_concepts.get_content(concept_id)?
                );
                
                associations.push(SemanticAssociation {
                    target_concept: concept_id,
                    confidence: similarity,
                    assoc_type,
                    reasoning: self.generate_reasoning(content, concept_id),
                });
                
                if associations.len() >= options.max_associations_per_concept {
                    break;
                }
            }
        }
        
        Ok(associations)
    }
    
    fn classify_relationship(&self, content1: &str, content2: &str) -> AssociationType {
        // NLP-based relationship classification
        if self.is_causal_relationship(content1, content2) {
            AssociationType::Causal
        } else if self.is_hierarchical_relationship(content1, content2) {
            AssociationType::Hierarchical
        } else if self.is_temporal_relationship(content1, content2) {
            AssociationType::Temporal
        } else {
            AssociationType::Semantic  // Default
        }
    }
}
```

### Association Types and Patterns

#### Medical Knowledge Associations

```python
# Medical concept learning with specialized associations
medical_facts = [
    "Diabetes mellitus is characterized by elevated blood glucose levels.",
    "Insulin resistance is the primary pathophysiology in type 2 diabetes.",
    "Metformin is the first-line treatment for type 2 diabetes.",
    "HbA1c testing should be performed every 3-6 months in diabetic patients.",
    "Diabetic retinopathy is a serious complication of poorly controlled diabetes."
]

# Learn concepts with medical association extraction
concept_ids = client.learn_batch_v2(
    contents=medical_facts,
    options={
        "extract_associations": True,
        "min_association_confidence": 0.8,  # High medical accuracy
        "embedding_model": "granite-embedding:30m"
    }
)

# Expected associations created:
# diabetes -> insulin_resistance (causal)
# diabetes -> metformin (treatment/hierarchical)  
# diabetes -> hba1c_testing (temporal/procedural)
# diabetes -> diabetic_retinopathy (causal/complication)
```

#### Legal Precedent Associations

```python
# Legal case learning with precedent relationships
legal_cases = [
    "Brown v. Board established that separate educational facilities are inherently unequal.",
    "Miranda v. Arizona requires law enforcement to inform suspects of their rights.",
    "Roe v. Wade established constitutional right to abortion under privacy doctrine.",
    "Citizens United v. FEC allowed unlimited corporate spending in elections."
]

concept_ids = client.learn_batch_v2(
    contents=legal_cases,
    options={
        "extract_associations": True,
        "min_association_confidence": 0.85,  # High legal precision
        "max_associations_per_concept": 12   # More connections for precedent
    }
)

# Expected associations:
# brown_v_board -> constitutional_law (hierarchical)
# miranda_v_arizona -> criminal_procedure (hierarchical)
# All cases -> supreme_court_cases (hierarchical)
# Related cases by legal doctrine (semantic)
```

### Manual Association Creation

```python
# Create specific relationships not captured automatically
def create_medical_protocol_hierarchy(client: StorageClient):
    """Create hierarchical medical protocol structure."""
    
    # Learn protocol categories
    emergency_protocols_id = client.learn_concept_v2(
        "Emergency Medical Protocols - Life-threatening conditions requiring immediate intervention"
    )
    
    cardiac_protocols_id = client.learn_concept_v2(
        "Cardiac Emergency Protocols - Heart-related emergency procedures"
    )
    
    # Learn specific protocols
    cpr_protocol_id = client.learn_concept_v2(
        "CPR Protocol: Check responsiveness, call 911, begin chest compressions at 100-120 per minute"
    )
    
    # Create hierarchical associations
    client.learn_association(
        source_id=cardiac_protocols_id,
        target_id=emergency_protocols_id,
        assoc_type=AssociationType.HIERARCHICAL,  # Part of larger category
        confidence=1.0
    )
    
    client.learn_association(
        source_id=cpr_protocol_id,
        target_id=cardiac_protocols_id,
        assoc_type=AssociationType.HIERARCHICAL,  # Specific protocol
        confidence=1.0
    )
    
    # Create procedural associations
    defibrillation_id = client.learn_concept_v2(
        "Defibrillation: Deliver electrical shock for ventricular fibrillation or ventricular tachycardia"
    )
    
    client.learn_association(
        source_id=cpr_protocol_id,
        target_id=defibrillation_id,
        assoc_type=AssociationType.TEMPORAL,  # Sequence relationship
        confidence=0.9
    )

create_medical_protocol_hierarchy(client)
```

## Quality Control and Validation

### Content Quality Assessment

```python
class LearningQualityController:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.quality_metrics = {}
    
    def assess_content_quality(self, content: str) -> Dict[str, float]:
        """Assess content quality across multiple dimensions."""
        scores = {}
        
        # Length appropriateness
        word_count = len(content.split())
        scores['length'] = self._score_length(word_count)
        
        # Information density
        scores['density'] = self._score_information_density(content)
        
        # Domain relevance (customizable)
        scores['domain_relevance'] = self._score_domain_relevance(content)
        
        # Factual clarity
        scores['clarity'] = self._score_clarity(content)
        
        # Overall quality (weighted average)
        weights = {'length': 0.2, 'density': 0.3, 'domain_relevance': 0.3, 'clarity': 0.2}
        scores['overall'] = sum(scores[k] * weights[k] for k in weights)
        
        return scores
    
    def learn_with_quality_control(self, content: str, min_quality: float = 0.7) -> Dict:
        """Learn concept only if it meets quality standards."""
        quality_scores = self.assess_content_quality(content)
        
        result = {
            'content': content,
            'quality_scores': quality_scores,
            'concept_id': None,
            'learned': False,
            'reason': None
        }
        
        if quality_scores['overall'] >= min_quality:
            # Adjust confidence based on quality
            confidence = min(quality_scores['overall'], 1.0)
            
            concept_id = self.client.learn_concept_v2(
                content=content,
                options={
                    'confidence': confidence,
                    'strength': quality_scores['overall'],
                    'extract_associations': quality_scores['overall'] > 0.8
                }
            )
            
            result.update({
                'concept_id': concept_id,
                'learned': True,
                'confidence_used': confidence
            })
        else:
            result['reason'] = f"Quality score {quality_scores['overall']:.2f} below threshold {min_quality}"
        
        return result
    
    def _score_length(self, word_count: int) -> float:
        """Score based on optimal length (10-100 words)."""
        if 10 <= word_count <= 100:
            return 1.0
        elif word_count < 10:
            return word_count / 10.0
        else:
            return max(0.3, 100.0 / word_count)
    
    def _score_information_density(self, content: str) -> float:
        """Score based on information content vs filler words."""
        words = content.lower().split()
        filler_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by'}
        
        if not words:
            return 0.0
        
        content_words = len([w for w in words if w not in filler_words])
        density = content_words / len(words)
        return min(density * 1.5, 1.0)  # Scale and cap at 1.0
    
    def _score_domain_relevance(self, content: str) -> float:
        """Score based on domain-specific terminology (customize as needed)."""
        # Medical terms example
        medical_terms = {
            'patient', 'diagnosis', 'treatment', 'therapy', 'clinical', 'medical',
            'disease', 'condition', 'symptom', 'procedure', 'protocol', 'guideline'
        }
        
        content_lower = content.lower()
        relevant_terms = sum(1 for term in medical_terms if term in content_lower)
        
        # Score based on relevant term density
        words = len(content.split())
        if words == 0:
            return 0.0
        
        relevance_ratio = relevant_terms / (words / 10)  # Normalize by content length
        return min(relevance_ratio, 1.0)
    
    def _score_clarity(self, content: str) -> float:
        """Score based on sentence structure and clarity indicators."""
        sentences = content.split('.')
        if not sentences:
            return 0.0
        
        # Average sentence length (optimal: 15-25 words)
        avg_sentence_length = sum(len(s.split()) for s in sentences) / len(sentences)
        length_score = 1.0 - abs(avg_sentence_length - 20) / 20
        length_score = max(0.3, min(1.0, length_score))
        
        # Presence of specific facts/numbers
        has_specifics = any(char.isdigit() for char in content) or '%' in content
        specifics_score = 0.8 if has_specifics else 0.6
        
        return (length_score + specifics_score) / 2

# Usage
quality_controller = LearningQualityController(client)

# Test content quality
test_content = """
Hypertension affects approximately 45% of adults in the United States. 
Blood pressure readings of 130/80 mmHg or higher indicate stage 1 hypertension.
Treatment typically includes lifestyle modifications and antihypertensive medications.
"""

result = quality_controller.learn_with_quality_control(test_content, min_quality=0.75)
print(f"Quality assessment: {result['quality_scores']}")
print(f"Learned: {result['learned']}")
if result['learned']:
    print(f"Concept ID: {result['concept_id']}")
```

### Association Quality Monitoring

```python
def analyze_association_quality(client: StorageClient, concept_id: str) -> Dict:
    """Analyze the quality of associations for a concept."""
    analysis = {
        'concept_id': concept_id,
        'neighbor_count': 0,
        'strong_associations': 0,
        'weak_associations': 0,
        'type_distribution': {},
        'avg_confidence': 0.0,
        'recommendations': []
    }
    
    try:
        # Get concept neighbors
        neighbor_ids = client.get_neighbors(concept_id)
        analysis['neighbor_count'] = len(neighbor_ids)
        
        if not neighbor_ids:
            analysis['recommendations'].append("No associations found - consider manual relationship creation")
            return analysis
        
        # Analyze association strengths (if API supports it)
        confidences = []
        for neighbor_id in neighbor_ids:
            try:
                # Note: get_association may not be available in all client versions
                assoc = client.get_association(concept_id, neighbor_id)
                if assoc:
                    confidence = assoc['confidence']
                    confidences.append(confidence)
                    
                    if confidence >= 0.8:
                        analysis['strong_associations'] += 1
                    elif confidence < 0.6:
                        analysis['weak_associations'] += 1
                    
                    # Track association types
                    assoc_type = assoc.get('type', 'unknown')
                    analysis['type_distribution'][assoc_type] = analysis['type_distribution'].get(assoc_type, 0) + 1
            
            except (AttributeError, Exception):
                # Fallback: estimate quality from vector similarity
                pass
        
        if confidences:
            analysis['avg_confidence'] = sum(confidences) / len(confidences)
        
        # Generate recommendations
        if analysis['weak_associations'] > analysis['strong_associations']:
            analysis['recommendations'].append("Many weak associations - consider raising min_association_confidence")
        
        if analysis['neighbor_count'] > 15:
            analysis['recommendations'].append("Many associations - consider reducing max_associations_per_concept")
        
        if analysis['neighbor_count'] < 3:
            analysis['recommendations'].append("Few associations - consider lowering min_association_confidence")
        
    except Exception as e:
        analysis['error'] = str(e)
    
    return analysis

# Usage
association_analysis = analyze_association_quality(client, concept_id)
print(f"Association analysis for {concept_id}:")
print(f"- Neighbors: {association_analysis['neighbor_count']}")
print(f"- Strong associations: {association_analysis['strong_associations']}")
print(f"- Average confidence: {association_analysis['avg_confidence']:.3f}")
for rec in association_analysis['recommendations']:
    print(f"- Recommendation: {rec}")
```

## Performance Optimization

### Batch Processing Strategy

```python
class OptimizedLearningPipeline:
    def __init__(self, storage_client: StorageClient, batch_size: int = 50):
        self.client = storage_client
        self.batch_size = batch_size
        self.pending_concepts = []
        self.statistics = {
            'processed': 0,
            'batches': 0,
            'errors': 0,
            'avg_batch_time': 0.0
        }
    
    def add_content(self, content: str, priority: int = 1):
        """Add content to processing queue."""
        self.pending_concepts.append({
            'content': content,
            'priority': priority,
            'timestamp': time.time()
        })
    
    def process_pending(self, options: dict = None) -> List[str]:
        """Process all pending concepts in optimized batches."""
        if not self.pending_concepts:
            return []
        
        # Sort by priority (higher first)
        self.pending_concepts.sort(key=lambda x: x['priority'], reverse=True)
        
        all_concept_ids = []
        batch_start_time = time.time()
        
        # Process in batches
        for i in range(0, len(self.pending_concepts), self.batch_size):
            batch = self.pending_concepts[i:i + self.batch_size]
            contents = [item['content'] for item in batch]
            
            try:
                concept_ids = self.client.learn_batch_v2(contents, options)
                all_concept_ids.extend(concept_ids)
                self.statistics['processed'] += len(concept_ids)
                self.statistics['batches'] += 1
                
                print(f"Processed batch {self.statistics['batches']}: {len(concept_ids)} concepts")
                
            except Exception as e:
                print(f"Batch processing error: {e}")
                self.statistics['errors'] += 1
                
                # Fallback: process individually
                for item in batch:
                    try:
                        concept_id = self.client.learn_concept_v2(item['content'], options)
                        all_concept_ids.append(concept_id)
                        self.statistics['processed'] += 1
                    except Exception as individual_error:
                        print(f"Individual concept error: {individual_error}")
                        self.statistics['errors'] += 1
        
        # Update timing statistics
        total_time = time.time() - batch_start_time
        if self.statistics['batches'] > 0:
            self.statistics['avg_batch_time'] = total_time / self.statistics['batches']
        
        # Clear pending queue
        self.pending_concepts.clear()
        
        return all_concept_ids
    
    def get_statistics(self) -> Dict:
        """Get processing statistics."""
        return self.statistics.copy()

# Usage for large dataset processing
pipeline = OptimizedLearningPipeline(client, batch_size=75)

# Add large number of medical protocols
medical_database = [
    # ... hundreds or thousands of medical facts
    "Aspirin 81mg daily reduces cardiovascular risk in high-risk patients.",
    "Blood glucose targets for diabetics: fasting 80-130 mg/dL, post-meal <180 mg/dL.",
    # ... more content
]

# Queue all content
for content in medical_database:
    pipeline.add_content(content, priority=2)

# Process efficiently
concept_ids = pipeline.process_pending(options={
    'embedding_model': 'granite-embedding:30m',
    'extract_associations': True,
    'min_association_confidence': 0.8
})

stats = pipeline.get_statistics()
print(f"Processing complete: {stats}")
```

### Memory-Efficient Learning

```python
def learn_large_dataset_efficiently(client: StorageClient, 
                                  content_generator,
                                  batch_size: int = 100) -> Dict:
    """Learn large datasets without loading everything into memory."""
    stats = {'total': 0, 'successful': 0, 'errors': 0, 'batches': 0}
    
    batch = []
    
    for content in content_generator:
        batch.append(content)
        stats['total'] += 1
        
        if len(batch) >= batch_size:
            # Process batch
            try:
                concept_ids = client.learn_batch_v2(batch)
                stats['successful'] += len(concept_ids)
                stats['batches'] += 1
                
                if stats['batches'] % 10 == 0:
                    print(f"Processed {stats['batches']} batches, {stats['successful']} concepts")
                
            except Exception as e:
                print(f"Batch error: {e}")
                stats['errors'] += len(batch)
            
            batch.clear()  # Free memory
    
    # Process remaining items
    if batch:
        try:
            concept_ids = client.learn_batch_v2(batch)
            stats['successful'] += len(concept_ids)
            stats['batches'] += 1
        except Exception as e:
            print(f"Final batch error: {e}")
            stats['errors'] += len(batch)
    
    return stats

# Usage with generator for memory efficiency
def medical_content_generator():
    """Generator that yields content without loading all into memory."""
    # Could read from file, database, API, etc.
    import csv
    with open('medical_protocols.csv', 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            yield f"{row['title']}: {row['description']}"

# Process efficiently
results = learn_large_dataset_efficiently(
    client, 
    medical_content_generator(), 
    batch_size=80
)
print(f"Processed {results['successful']} of {results['total']} items in {results['batches']} batches")
```

## Next Steps

- [**Multi-Tenancy**](./06-multi-tenancy.md) - Organization isolation strategies
- [**Vector Search**](./07-vector-search.md) - Advanced semantic search
- [**Performance Guide**](./08-performance.md) - System optimization

---

*The unified learning pipeline provides the intelligence layer that transforms raw content into structured, interconnected knowledge ready for AI reasoning.*