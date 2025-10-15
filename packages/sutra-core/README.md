# Sutra Core - Graph-Based Reasoning Engine

The foundational package for the Sutra AI system, providing graph-based reasoning with explainable AI capabilities.

## 🎉 Recent Updates (October 15, 2025)

### Phase 1: Type Safety & Validation ✅
- **95% type coverage** (up from 40%)
- Comprehensive input validation with DOS protection
- Zero critical mypy errors

### Phase 2: NLP Upgrade ✅
- **spaCy integration** (3.8.7 with en_core_web_sm)
- Production-grade text processing
- Lemmatization, entity extraction, negation detection
- Backward compatible with fallback

See `PHASE1_2_SUMMARY.md` for complete details.

---

## Features

- **Concept Management**: Adaptive strength learning for knowledge concepts
- **Association Networks**: Typed relationships between concepts (semantic, causal, temporal, hierarchical, compositional)  
- **Adaptive Learning**: Research-based adaptive focus learning (AdaKD-inspired)
- **Real-Time Integration**: Instant knowledge updates without retraining
- **Input Validation**: Comprehensive validation with DOS protection (NEW)
- **Modern NLP**: spaCy-based text processing with lemmatization and NER (NEW)

## Installation

```bash
pip install -e .
```

## Quick Start

### Basic Concepts

```python
from sutra_core import Concept, Association, AssociationType

# Create a concept
concept = Concept(
    id="photosynthesis", 
    content="process by which plants convert light energy to chemical energy"
)

# Access strengthens the concept
concept.access()
print(f"Concept strength: {concept.strength}")
```

### Adaptive Learning

```python
from sutra_core.learning import AdaptiveLearner, AssociationExtractor
from collections import defaultdict

# Set up learning system
concepts = {}
associations = {}
word_to_concepts = defaultdict(set)
concept_neighbors = defaultdict(set)

extractor = AssociationExtractor(
    concepts, word_to_concepts, concept_neighbors, associations
)
learner = AdaptiveLearner(concepts, extractor)

# Learn new knowledge with adaptive focus
concept_id = learner.learn_adaptive(
    "Photosynthesis converts sunlight into glucose and oxygen",
    source="biology_textbook"
)
```

### Text Processing

```python
from sutra_core.utils.nlp import TextProcessor

# NEW: Modern NLP with spaCy
processor = TextProcessor()

# Lemmatization
tokens = processor.extract_meaningful_tokens("The cats are running quickly")
print(tokens)  # ['cat', 'run', 'quickly']

# Entity extraction
entities = processor.extract_entities("Apple Inc. is based in Cupertino, California")
print(entities)  # [('Apple Inc.', 'ORG'), ('Cupertino', 'GPE'), ('California', 'GPE')]

# Negation detection
has_negation = processor.detect_negation("The sun is not a planet")
print(has_negation)  # True

# Subject-verb-object triples
triples = processor.extract_subject_verb_object("Cats chase mice")
print(triples)  # [('cats', 'chase', 'mice', False)]

# Backward compatible
from sutra_core.utils import extract_words, get_association_patterns

# Extract meaningful words (with fallback if spaCy unavailable)
words = extract_words("Photosynthesis is a crucial biological process")
print(words)  # ['photosynthesis', 'crucial', 'biological', 'process']

# Get association patterns
patterns = get_association_patterns()
for pattern, assoc_type in patterns:
    print(f"{assoc_type}: {pattern}")
```

### Input Validation

```python
from sutra_core import Validator

# NEW: Comprehensive validation
try:
    Validator.validate_content("Some content")  # ✅ Valid
    Validator.validate_query("What is X?")      # ✅ Valid
    Validator.validate_confidence(0.95)         # ✅ Valid
    Validator.validate_depth(5)                 # ✅ Valid
    
    # DOS protection
    huge_content = "x" * 100000
    Validator.validate_content(huge_content)    # ❌ Raises ValidationError
    
    # Auto-clamping
    confidence = Validator.validate_confidence(1.5)
    print(confidence)  # 1.0 (clamped)
    
except Exception as e:
    print(f"Validation error: {e}")
```

## Architecture

### Core Components

- **`graph/`**: Core data structures (Concept, Association, ReasoningPath)
- **`learning/`**: Adaptive learning algorithms and association extraction
- **`utils/`**: Text processing and utility functions

### Key Classes

#### Concept
Represents a knowledge unit with adaptive strength learning:
- Strength increases with access (max 10.0)
- Tracks usage statistics and metadata
- Serializable for persistence

#### Association  
Weighted connections between concepts:
- Multiple types: semantic, causal, temporal, hierarchical, compositional
- Confidence scoring for uncertainty handling
- Strengthens through use

#### AdaptiveLearner
Implements research-based adaptive focus:
- Difficult concepts (strength < 4.0): Strong reinforcement (1.15×)
- Easy concepts (strength > 7.0): Minimal reinforcement (1.01×)
- Dynamic extraction depth based on difficulty

## Research Foundation

Based on cutting-edge research:

- **Adaptive Focus Learning**: "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2025)
- **Loss-Driven Token Focusing**: More compute for difficult concepts
- **Dynamic Reinforcement**: Strength-based learning strategies

## Development

```bash
# Run tests
python -m pytest tests/

# Format code  
black sutra_core/
isort sutra_core/

# Type checking
mypy sutra_core/
```

## Performance

- **Memory**: ~0.1KB per concept
- **Learning**: ~1000 concepts/second
- **Access**: O(1) concept lookup via word indexing
- **Associations**: O(branches^depth) graph traversal

## Configuration (New)

- ReasoningEngine caching:
  - enable_caching (bool): enable query caching
  - max_cache_size (int): max cached queries
  - cache_ttl_seconds (float|None): optional TTL for cached entries (LRU+TTL)
- Learning links from central concept to extracted phrases:
  - enable_central_links (bool): default True
  - central_link_confidence (float): default 0.6
  - central_link_type (AssociationType): default COMPOSITIONAL

## Persistence Notes (Updated)

- Associations are serialized with ":" as the source/target separator. The loader accepts both ":" and "|" for backward compatibility.
- Index rebuild now uses the same tokenization as learning (utils.extract_words) to keep word-to-concepts consistent after load.

## License

MIT License - see LICENSE file for details.
