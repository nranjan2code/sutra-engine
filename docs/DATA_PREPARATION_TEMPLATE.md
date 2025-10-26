# Sutra AI Data Preparation Template

## Executive Summary

The Sutra storage engine is a domain-specific knowledge graph that learns through **structured concept ingestion** with semantic understanding. Unlike traditional LLMs, Sutra builds a queryable graph of interconnected concepts with embeddings, associations, and semantic metadata.

This document provides the optimal data structure and preparation guidelines for training the Sutra storage engine to maximize reasoning quality and performance.

## Core Learning Algorithm

### What Happens When You "Learn"

When you submit data to Sutra's `/learn` endpoint, the storage engine performs:

1. **Concept ID Generation** - Deterministic hash of content (16-byte MD5)
2. **Semantic Embedding** - 768-dimensional vector via nomic-embed-text-v1.5
3. **Semantic Analysis** - Pattern-based classification (11 types, 15+ domains)
4. **Association Extraction** - Semantic relationship discovery via embeddings
5. **Graph Storage** - Atomic write with HNSW index update

### Data Flow
```
Raw Text → Embedding Generation → Semantic Analysis → Association Extraction → Graph Storage
         ↓                      ↓                   ↓                      ↓
    768-d vector          Type/Domain         Related Concepts      Knowledge Graph
```

## Optimal Data Structure

### 1. Single Concept Format (Basic Unit)

```json
{
  "text": "The mitochondria is the powerhouse of the cell, responsible for producing ATP through cellular respiration.",
  "metadata": {
    "source": "biology_textbook",
    "chapter": "cellular_biology",
    "confidence": 0.95
  }
}
```

**Key Principles:**
- **Self-contained**: Each concept should be complete and understandable in isolation
- **Factual**: Focus on declarative knowledge rather than procedural
- **Atomic**: One main idea per concept (can have supporting details)
- **Clean**: No formatting artifacts, HTML tags, or special characters

### 2. Batch Learning Format (Recommended for Large Datasets)

```json
{
  "concepts": [
    {
      "text": "Machine learning is a subset of artificial intelligence that enables systems to learn from data.",
      "id": "ml_definition_001",  // Optional: custom ID
      "tags": ["AI", "ML", "fundamentals"]
    },
    {
      "text": "Supervised learning requires labeled training data with input-output pairs.",
      "id": "ml_supervised_001",
      "tags": ["ML", "supervised_learning"]
    },
    {
      "text": "Neural networks are computational models inspired by biological neural networks.",
      "id": "ml_neural_001",
      "tags": ["ML", "neural_networks"]
    }
  ],
  "metadata": {
    "dataset": "ml_fundamentals",
    "version": "1.0",
    "date": "2024-01-26"
  }
}
```

### 3. Structured Knowledge Format (Best for Domain Expertise)

```json
{
  "domain": "medical",
  "concepts": [
    {
      "type": "definition",
      "term": "Hypertension",
      "text": "Hypertension is a chronic medical condition where blood pressure in the arteries is persistently elevated above 130/80 mmHg.",
      "relationships": [
        {"target": "cardiovascular disease", "type": "causes"},
        {"target": "stroke", "type": "risk_factor"},
        {"target": "ACE inhibitors", "type": "treated_by"}
      ]
    },
    {
      "type": "fact",
      "text": "Primary hypertension accounts for 90-95% of all cases and has no identifiable cause.",
      "confidence": 0.98,
      "source": "WHO Guidelines 2023"
    },
    {
      "type": "protocol",
      "text": "Initial hypertension treatment involves lifestyle modifications including sodium restriction, weight loss, and regular exercise.",
      "temporal": "first_line_treatment"
    }
  ]
}
```

## CSV Format for Bulk Ingestion

### Simple Facts CSV
```csv
text,source,confidence
"Python is a high-level programming language created by Guido van Rossum",tutorial,0.95
"Python uses dynamic typing and automatic memory management",documentation,0.98
"The Python Package Index (PyPI) hosts over 400000 packages",statistics,0.90
```

### Knowledge Graph CSV
```csv
subject,predicate,object,confidence,domain
"Python","is_a","programming language",1.0,"technology"
"Python","created_by","Guido van Rossum",1.0,"technology"
"Python","has_feature","dynamic typing",0.95,"technology"
"Django","written_in","Python",1.0,"technology"
"Flask","is_a","web framework",1.0,"technology"
```

### Q&A Pairs CSV (Transforms to Facts)
```csv
question,answer,domain,confidence
"What is photosynthesis?","Photosynthesis is the process by which plants convert light energy into chemical energy",biology,0.95
"Who invented the telephone?","Alexander Graham Bell invented the telephone in 1876",history,0.98
"What is the capital of France?","Paris is the capital of France",geography,1.0
```

## Data Quality Guidelines

### ✅ GOOD Concepts

1. **Factual Statements**
   ```
   "The Great Wall of China is approximately 21,196 kilometers long."
   ```

2. **Definitions**
   ```
   "Photosynthesis is the process by which plants convert carbon dioxide and water into glucose using sunlight."
   ```

3. **Causal Relationships**
   ```
   "Deforestation leads to increased carbon dioxide levels in the atmosphere, contributing to global warming."
   ```

4. **Temporal Sequences**
   ```
   "The Renaissance period in Europe lasted from the 14th to the 17th century, followed by the Age of Enlightenment."
   ```

5. **Hierarchical Knowledge**
   ```
   "Mammals are a class of vertebrates characterized by warm blood, hair, and milk production for their young."
   ```

### ❌ POOR Concepts

1. **Vague Statements**
   ```
   "Some things are important to consider sometimes."  // No specific information
   ```

2. **Opinions Without Facts**
   ```
   "This is the best approach."  // Subjective without justification
   ```

3. **Incomplete Fragments**
   ```
   "The process involves..."  // Trailing off without completion
   ```

4. **Redundant Duplicates**
   ```
   "Paris is the capital of France."
   "The capital of France is Paris."  // Same information
   ```

5. **Formatting Noise**
   ```
   "<p>Click here for more info</p>"  // HTML artifacts
   "Page 42 | Chapter 3 |"  // Navigation elements
   ```

## Pre-processing Pipeline

### Step 1: Data Cleaning
```python
import re
import json

def clean_concept_text(text):
    """Clean and normalize concept text."""
    # Remove HTML tags
    text = re.sub(r'<[^>]+>', '', text)
    
    # Remove multiple whitespaces
    text = ' '.join(text.split())
    
    # Remove special characters but keep punctuation
    text = re.sub(r'[^\w\s\.\,\!\?\-\(\)]', '', text)
    
    # Ensure sentence ends with period
    if text and text[-1] not in '.!?':
        text += '.'
    
    return text.strip()
```

### Step 2: Concept Extraction
```python
def extract_concepts_from_document(document, min_length=20, max_length=500):
    """Extract atomic concepts from larger document."""
    concepts = []
    
    # Split by sentences
    sentences = document.split('.')
    
    current_concept = ""
    for sentence in sentences:
        sentence = sentence.strip()
        if not sentence:
            continue
            
        # Check if adding sentence keeps concept atomic
        if len(current_concept) + len(sentence) < max_length:
            current_concept += sentence + ". "
        else:
            # Save current concept
            if len(current_concept) >= min_length:
                concepts.append({
                    "text": clean_concept_text(current_concept)
                })
            current_concept = sentence + ". "
    
    # Don't forget last concept
    if len(current_concept) >= min_length:
        concepts.append({
            "text": clean_concept_text(current_concept)
        })
    
    return concepts
```

### Step 3: Enrichment
```python
def enrich_concepts_with_metadata(concepts, source_metadata):
    """Add metadata for better organization."""
    enriched = []
    
    for i, concept in enumerate(concepts):
        enriched.append({
            "text": concept["text"],
            "metadata": {
                "source": source_metadata.get("source", "unknown"),
                "index": i,
                "timestamp": datetime.utcnow().isoformat(),
                "word_count": len(concept["text"].split()),
                "char_count": len(concept["text"])
            }
        })
    
    return enriched
```

### Step 4: Validation
```python
def validate_concept_batch(concepts):
    """Validate concepts meet quality standards."""
    valid = []
    rejected = []
    
    for concept in concepts:
        text = concept.get("text", "")
        
        # Check minimum length
        if len(text) < 10:
            rejected.append({"concept": concept, "reason": "too_short"})
            continue
        
        # Check for actual content
        if len(set(text.split())) < 3:  # Less than 3 unique words
            rejected.append({"concept": concept, "reason": "low_diversity"})
            continue
            
        # Check for noise patterns
        if any(pattern in text.lower() for pattern in ['click here', 'page', 'chapter', 'fig.', 'table']):
            rejected.append({"concept": concept, "reason": "formatting_noise"})
            continue
        
        valid.append(concept)
    
    print(f"Validated: {len(valid)} accepted, {len(rejected)} rejected")
    return valid, rejected
```

## Domain-Specific Templates

### Medical/Healthcare
```json
{
  "domain": "medical",
  "concept_template": {
    "disease": {
      "text": "[Disease] is a [type] condition characterized by [symptoms]. It is caused by [etiology] and treated with [treatment].",
      "example": "Type 2 diabetes is a metabolic disorder characterized by high blood sugar, insulin resistance, and relative insulin deficiency. It is caused by genetic factors and lifestyle choices and treated with metformin, lifestyle changes, and insulin therapy."
    },
    "drug": {
      "text": "[Drug] is a [class] medication that works by [mechanism]. It is used to treat [conditions] with common side effects including [side_effects].",
      "example": "Metformin is a biguanide medication that works by decreasing glucose production in the liver. It is used to treat type 2 diabetes with common side effects including nausea and diarrhea."
    }
  }
}
```

### Legal/Regulatory
```json
{
  "domain": "legal",
  "concept_template": {
    "regulation": {
      "text": "[Regulation] enacted in [year] by [authority] requires [requirements] and applies to [scope].",
      "example": "GDPR enacted in 2018 by the European Union requires explicit consent for data processing and applies to all organizations processing EU residents' data."
    },
    "precedent": {
      "text": "In [case_name] ([year]), the [court] ruled that [ruling], establishing the principle that [principle].",
      "example": "In Brown v. Board of Education (1954), the Supreme Court ruled that racial segregation in schools is unconstitutional, establishing the principle that separate but equal is inherently unequal."
    }
  }
}
```

### Technical/Engineering
```json
{
  "domain": "technical",
  "concept_template": {
    "algorithm": {
      "text": "[Algorithm] is a [type] algorithm with time complexity [time] and space complexity [space]. It is used for [purpose] and works by [mechanism].",
      "example": "QuickSort is a divide-and-conquer algorithm with average time complexity O(n log n) and space complexity O(log n). It is used for sorting arrays and works by selecting a pivot and partitioning elements."
    },
    "pattern": {
      "text": "The [pattern_name] pattern solves [problem] by [solution]. It consists of [components] and is commonly used in [context].",
      "example": "The Observer pattern solves the problem of notifying multiple objects about state changes by defining a one-to-many dependency. It consists of Subject and Observer interfaces and is commonly used in event-driven systems."
    }
  }
}
```

## Ingestion Best Practices

### 1. Batch Size Optimization
- **Optimal batch size**: 100-500 concepts per request
- **Memory considerations**: Each concept with embedding uses ~3KB
- **Network efficiency**: Batch embeddings are generated together

### 2. Deduplication
```python
def deduplicate_concepts(concepts):
    """Remove duplicate concepts based on content hash."""
    seen = set()
    unique = []
    
    for concept in concepts:
        content_hash = hashlib.md5(concept["text"].encode()).hexdigest()
        if content_hash not in seen:
            seen.add(content_hash)
            unique.append(concept)
    
    return unique
```

### 3. Progressive Enhancement
```python
# Stage 1: Learn core definitions
core_concepts = load_definitions()
api.learn_batch(core_concepts)

# Stage 2: Add relationships
relationships = extract_relationships()
api.learn_batch(relationships)

# Stage 3: Add examples and edge cases
examples = load_examples()
api.learn_batch(examples)
```

### 4. Quality Metrics
Monitor these metrics during ingestion:
- **Association density**: Aim for 2-5 associations per concept
- **Embedding coverage**: Should be 100% (no missing embeddings)
- **Semantic diversity**: Mix of all 5 association types
- **Confidence distribution**: Most concepts > 0.7 confidence

## API Usage Examples

### Python Client
```python
import requests
import json

# Single concept
def learn_concept(text, confidence=0.95):
    response = requests.post(
        "http://localhost:8001/sutra/learn",
        json={"text": text, "confidence": confidence}
    )
    return response.json()

# Batch learning
def learn_batch(concepts):
    response = requests.post(
        "http://localhost:8001/sutra/learn/batch",
        json={"concepts": concepts}
    )
    return response.json()

# With metadata
def learn_with_metadata(text, metadata):
    response = requests.post(
        "http://localhost:8001/sutra/learn",
        json={
            "text": text,
            "metadata": metadata,
            "options": {
                "generate_embedding": True,
                "extract_associations": True,
                "analyze_semantics": True
            }
        }
    )
    return response.json()
```

### Bulk CSV Ingestion
```bash
# Using the bulk ingester service
curl -X POST http://localhost:8005/ingest/csv \
  -H "Content-Type: multipart/form-data" \
  -F "file=@knowledge.csv" \
  -F "config={\"format\":\"simple_facts\",\"has_header\":true}"
```

## Performance Expectations

| Dataset Size | Ingestion Time | Memory Usage | Storage Size |
|-------------|----------------|--------------|--------------|
| 1K concepts | ~2 seconds | 50MB | 5MB |
| 10K concepts | ~20 seconds | 200MB | 50MB |
| 100K concepts | ~3 minutes | 1GB | 500MB |
| 1M concepts | ~30 minutes | 8GB | 5GB |
| 10M concepts | ~5 hours | 64GB | 50GB |

**Throughput**: 
- Learning: ~50,000 concepts/sec (storage write)
- Embedding: ~100 concepts/sec (with HA service)
- Overall: ~500-1000 concepts/sec (end-to-end)

## Troubleshooting

### Common Issues

1. **"Same answer for all queries"**
   - Cause: No embeddings generated
   - Fix: Ensure SUTRA_EMBEDDING_SERVICE_URL is configured

2. **"Association extraction failing"**
   - Cause: Text too short or no entities
   - Fix: Ensure concepts have proper nouns or clear relationships

3. **"Slow ingestion"**
   - Cause: Small batch sizes
   - Fix: Batch 100-500 concepts together

4. **"High memory usage"**
   - Cause: Too many concepts in single batch
   - Fix: Stream data in smaller batches

## Conclusion

The Sutra storage engine thrives on well-structured, atomic concepts with clear relationships. Focus on:

1. **Quality over quantity** - Clean, factual, self-contained concepts
2. **Atomic knowledge** - One main idea per concept
3. **Rich relationships** - Explicit connections between concepts
4. **Domain consistency** - Keep related concepts together
5. **Progressive building** - Start with core, add details gradually

Following this template will ensure optimal knowledge graph construction and superior reasoning performance.

## Appendix: Sample Datasets

### Download Ready-to-Use Datasets

```bash
# Simple facts (1000 concepts)
curl -O https://sutra.ai/datasets/general-knowledge-1k.json

# Medical knowledge (5000 concepts)  
curl -O https://sutra.ai/datasets/medical-5k.json

# Legal precedents (2000 concepts)
curl -O https://sutra.ai/datasets/legal-2k.json

# Technical concepts (10000 concepts)
curl -O https://sutra.ai/datasets/technical-10k.json
```

### Dataset Generators

```python
# Generate synthetic training data
from sutra_datasets import DatasetGenerator

generator = DatasetGenerator(domain="medical")
concepts = generator.generate(
    count=1000,
    complexity="medium",
    include_associations=True
)

# Save for ingestion
with open("medical_training.json", "w") as f:
    json.dump({"concepts": concepts}, f, indent=2)
```

---

**Document Version**: 1.0  
**Last Updated**: 2024-01-26  
**Maintainer**: Sutra AI Team  
**License**: MIT