# sutra-core

Core graph reasoning engine. Production-ready.

## Features

- Concepts (nodes) with adaptive strength (1.0-10.0)
- Associations (edges) with types, confidence, and last_used timestamps
- Advanced reasoning engine (ReasoningEngine) with QueryProcessor, PathFinder, MPPA
- Adaptive learning (strength boost depends on difficulty)
- Maintenance APIs: health snapshot and decay/prune
- Text utilities (tokenization, normalization)
- Custom exception hierarchy

## Public APIs

### Core Types

- `sutra_core.Concept`
- `sutra_core.Association`
- `sutra_core.AssociationType`
- `sutra_core.learning.AdaptiveLearner`
- `sutra_core.learning.AssociationExtractor`
- `sutra_core.utils` (extract_words, clean_text, calculate_word_overlap)
- `sutra_core.exceptions` (SutraError and subclasses)

## Usage

### Maintenance APIs

```python
from sutra_core import ReasoningEngine

ai = ReasoningEngine()
health = ai.get_health_snapshot()
pruned = ai.decay_and_prune(
    concept_decay_after_days=14,
    concept_remove_after_days=90,
    min_strength_to_keep=1.0,
    association_remove_after_days=90,
    min_association_confidence_to_keep=0.2,
    daily_decay_rate=0.995,
)
```

### Create and Access Concept

```python
from sutra_core import Concept

c = Concept(id="photosynthesis", content="Photosynthesis converts sunlight")
print(c.strength)   # 1.0
c.access()
print(c.strength)   # 1.02 (access boost)
```

### Learn Knowledge (Adaptive)

```python
from collections import defaultdict
from sutra_core.learning import AdaptiveLearner, AssociationExtractor

concepts = {}
associations = {}
word_to_concepts = defaultdict(set)
concept_neighbors = defaultdict(set)

extractor = AssociationExtractor(
    concepts, word_to_concepts, concept_neighbors, associations
)
learner = AdaptiveLearner(concepts, extractor)

concept_id = learner.learn_adaptive(
    "Enzymes are proteins that speed up reactions",
    source="biology_textbook"
)
print(concept_id)
print(len(associations))
```

### Association Types

```python
from sutra_core import AssociationType

print(list(AssociationType))
# semantic, causal, temporal, hierarchical, compositional
```

## Exceptions

- `SutraError` (base)
- `ConceptError`
- `AssociationError`
- `LearningError`
- `ValidationError`
- `StorageError`
- `ConfigurationError`

## Tests

- Location: `packages/sutra-core/tests/`
- Run:

```bash
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests -v
```

## Notes

- Maintain a `visited` set during traversal to avoid cycles
- Strength max is 10.0 to prevent runaway growth
- Associations strengthen both weight and confidence and update `last_used`
- Traversal refreshes association `last_used`
- Context expansion uses associations with confidence >= 0.6
- Final query confidence is clamped to [0, 1]
- Co-occurrence extraction has a cap (default 200 links)
- Concept/phrase IDs use 16 hex chars
- Decay factor during propagation: 0.9 per hop
