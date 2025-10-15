# Sutra AI - API Reference

Complete reference for the Sutra AI reasoning engine and all AI capabilities.

## 🧠 ReasoningEngine

The main AI interface that orchestrates all reasoning components.

### Constructor

```python
ReasoningEngine(
    enable_caching=True,
    max_cache_size=1000,
    cache_ttl_seconds=None,          # Optional TTL (seconds) for query cache
    enable_central_links=True,       # Link central concept to extracted phrases
    central_link_confidence=0.6,     # Confidence for central links
    central_link_type=AssociationType.COMPOSITIONAL  # Link type
)
```

**Parameters:**
- `enable_caching` (bool): Enable query result caching for performance
- `max_cache_size` (int): Maximum number of cached query results

### Core Methods

#### ask(question, **kwargs) → ConsensusResult
Process natural language questions and return AI reasoning results.

```python
result = ai.ask("How do plants make energy?", num_reasoning_paths=5)
```

**Parameters:**
- `question` (str): Natural language question
- `num_reasoning_paths` (int, optional): Number of reasoning paths to explore (default: 5)
- `max_concepts` (int, optional): Maximum relevant concepts to consider (default: 10)

**Returns:**
`ConsensusResult` with:
- `primary_answer` (str): Best answer from consensus
- `confidence` (float): Overall confidence score (0.0-1.0)
- `consensus_strength` (float): Agreement strength across paths
- `supporting_paths` (List[ReasoningPath]): Reasoning paths that support the answer
- `alternative_answers` (List[Tuple[str, float]]): Alternative answers with confidence
- `reasoning_explanation` (str): Human-readable reasoning explanation

#### learn(content, source=None, **kwargs) → str
Learn new knowledge instantly without retraining.

```python
concept_id = ai.learn("CRISPR is a gene editing technology", source="research_paper")
```

**Parameters:**
- `content` (str): Knowledge content to learn
- `source` (str, optional): Source attribution
- `category` (str, optional): Knowledge category/domain

**Returns:**
- `str`: Unique concept ID for the learned knowledge

#### explain_reasoning(question, detailed=False) → dict
Get detailed explanation of AI reasoning process.

```python
explanation = ai.explain_reasoning("Why is water important?", detailed=True)
```

**Parameters:**
- `question` (str): Question to explain reasoning for
- `detailed` (bool): Include detailed reasoning paths

**Returns:**
Dictionary with:
- `question` (str): Original question
- `answer` (str): AI's answer
- `confidence` (float): Answer confidence
- `consensus_strength` (float): Path agreement strength
- `reasoning_explanation` (str): Detailed reasoning explanation
- `alternative_answers` (List): Alternative possibilities
- `reasoning_robustness` (dict): Robustness analysis metrics
- `detailed_paths` (List, optional): Step-by-step reasoning paths

### System Management

#### get_system_stats() → dict
Get comprehensive system statistics.

```python
stats = ai.get_system_stats()
```

**Returns:**
Dictionary with:
```python
{
    "system_info": {
        "total_concepts": int,
        "total_associations": int,
        "total_queries": int,
        "learning_events": int,
        "cache_hits": int,
        "cache_hit_rate": float,
        "cache_size": int
    },
    "learning_stats": {
        "difficult_concepts": int,
        "moderate_concepts": int,
        "easy_concepts": int,
        "average_strength": float
    },
    "association_stats": {
        "by_type": dict,
        "average_confidence": float
    }
}
```

#### optimize_performance() → dict
Run performance optimization routines.

```python
optimizations = ai.optimize_performance()
```

**Returns:**
Dictionary with optimization results:
- `concepts_strengthened` (int): Number of concepts strengthened
- `weak_associations_removed` (int): Number of weak associations pruned
- `cache_entries_pruned` (int): Number of cache entries cleaned

#### get_health_snapshot() → dict
Return a compact runtime health snapshot (totals, averages, cache stats).

```python
snapshot = ai.get_health_snapshot()
```

**Returns:**
Dictionary with:
- `totals`: `concepts`, `associations`
- `concepts`: `avg_strength`, `weak_pct`, `strong_pct`
- `associations`: `avg_confidence`, `weak_count`, `stale_count`
- `graph`: `avg_degree`
- `cache`: `enabled`, `size`, `hit_rate`

#### decay_and_prune(...) → dict
Decay inactive concepts and prune stale/low-confidence associations.

```python
pruned = ai.decay_and_prune(
    concept_decay_after_days=14,
    concept_remove_after_days=90,
    min_strength_to_keep=1.0,
    association_remove_after_days=90,
    min_association_confidence_to_keep=0.2,
    daily_decay_rate=0.995,
)
```

Parameters:
- `concept_decay_after_days` (int): Inactivity threshold to start decaying strength
- `concept_remove_after_days` (int): Inactivity threshold for potential concept removal
- `min_strength_to_keep` (float): Minimum strength to avoid removal
- `association_remove_after_days` (int): Inactivity threshold for associations
- `min_association_confidence_to_keep` (float): Prune stale associations below this
- `daily_decay_rate` (float): Multiplicative daily decay factor for concept strength

Returns counts for: `concepts_decayed`, `concepts_removed`, `associations_removed`

#### save_knowledge_base(filepath) → bool
Save knowledge base to file.

```python
success = ai.save_knowledge_base("my_knowledge.json")
```

#### load_knowledge_base(filepath) → bool  
Load knowledge base from file.

```python
success = ai.load_knowledge_base("my_knowledge.json")
```

### Search and Discovery

#### search_concepts(query, limit=10) → List[dict]
Search for concepts matching a query.

```python
results = ai.search_concepts("photosynthesis", limit=5)
```

**Returns:**
List of concept information dictionaries with:
- `id`, `content`, `strength`, `confidence`
- `access_count`, `created`, `last_accessed` 
- `source`, `category`, `neighbor_count`, `neighbors`
- `relevance_score` (float): Match score for the query

#### get_concept_info(concept_id) → dict
Get detailed information about a specific concept.

```python
info = ai.get_concept_info("concept_id_here")
```

## 🔍 PathFinder

Advanced graph traversal for multi-hop reasoning.

Note: Traversal refreshes association `last_used` when edges are expanded, enabling time-based pruning policies.

### Constructor

```python
PathFinder(concepts, associations, concept_neighbors)
```

### find_reasoning_paths(start_concepts, target_concepts, num_paths=3, search_strategy="best_first") → List[ReasoningPath]

Find multiple reasoning paths between concept sets.

**Parameters:**
- `start_concepts` (List[str]): Starting concept IDs
- `target_concepts` (List[str]): Target concept IDs
- `num_paths` (int): Number of diverse paths to find
- `search_strategy` (str): "best_first", "breadth_first", or "bidirectional"

**Search Strategies:**
- **best_first**: Confidence-optimized with target proximity heuristics
- **breadth_first**: Shortest path exploration with cycle detection  
- **bidirectional**: Optimal path finding from both ends

### Configuration

```python
path_finder.max_depth = 6              # Maximum reasoning depth
path_finder.min_confidence = 0.1       # Minimum path confidence
path_finder.confidence_decay = 0.85    # Confidence decay per step
```

## 🤝 MultiPathAggregator (MPPA)

Consensus-based reasoning that prevents single-path derailment.

### Constructor

```python
MultiPathAggregator(
    consensus_threshold=0.5,
    min_paths_for_consensus=2,
    diversity_penalty=0.1,
    outlier_penalty=0.3
)
```

**Parameters:**
- `consensus_threshold` (float): Minimum agreement for consensus (0.0-1.0)
- `min_paths_for_consensus` (int): Minimum paths needed for consensus
- `diversity_penalty` (float): Penalty for lack of path diversity
- `outlier_penalty` (float): Penalty for outlier answers (singleton paths)

### aggregate_reasoning_paths(reasoning_paths, query) → ConsensusResult

Aggregate multiple reasoning paths into consensus result.

**Algorithm:**
1. **Path Clustering**: Group paths by answer similarity (0.8 threshold)
2. **Consensus Scoring**: Calculate support and confidence for each cluster
3. **Majority Voting**: Select cluster with highest consensus weight
4. **Robustness Analysis**: Analyze path diversity and confidence consistency

### analyze_reasoning_robustness(consensus_result) → dict

Analyze robustness of reasoning results.

**Returns:**
- `robustness_score` (float): Overall robustness (0.0-1.0)
- `path_diversity` (float): Diversity of reasoning approaches
- `confidence_consistency` (float): Consistency of confidence scores
- `consensus_strength` (float): Agreement strength across paths
- `supporting_path_count` (int): Number of supporting paths
- `alternative_answer_count` (int): Number of alternative answers

## 🗣️ QueryProcessor

Natural language understanding for complex queries.

### Constructor

```python
QueryProcessor(concepts, associations, concept_neighbors, association_extractor, path_finder, mppa)
```

### process_query(query, num_reasoning_paths=5, max_concepts=10) → ConsensusResult

Process natural language query through full reasoning pipeline.

**Processing Steps:**
1. **Query Normalization**: Clean and normalize input text
2. **Intent Classification**: Classify query type and complexity
3. **Concept Extraction**: Find relevant concepts and rank by relevance
4. **Context Expansion**: Expand with related high-confidence concepts
5. **Path Generation**: Generate multiple reasoning paths with different strategies
6. **Consensus Aggregation**: Use MPPA to build consensus answer
7. **Result Enhancement**: Add explanations and confidence adjustments

### Query Intent Classification

The system recognizes these query patterns:

| Intent Type | Patterns | Example |
|-------------|----------|---------|
| **what** | "what is", "what are", "what does" | "What is photosynthesis?" |
| **how** | "how does", "how do", "how can" | "How do plants make energy?" |
| **why** | "why does", "why is", "why are" | "Why is sunlight important?" |
| **when** | "when does", "when is" | "When does photosynthesis occur?" |
| **where** | "where is", "where does" | "Where does photosynthesis happen?" |
| **who** | "who is", "who does" | "Who discovered photosynthesis?" |

### Query Complexity Assessment

- **Simple queries** (definitions): +10% confidence boost
- **Complex multi-part queries** (>10 words): -5% confidence  
- **Causal reasoning** (seeking causes): -10% confidence  
- **Comparison queries**: -15% confidence

Notes:
- Final confidence is clamped to [0, 1].
- Context expansion uses associations with confidence >= 0.6.

### get_query_suggestions(partial_query, max_suggestions=5) → List[str]

Generate query suggestions based on partial input.

```python
suggestions = processor.get_query_suggestions("photosyn", max_suggestions=3)
# Returns: ["What is photosynthesis?", "How does photosynthesis work?", ...]
```

## 📊 Data Structures

### ConsensusResult

Result of multi-path consensus analysis.

```python
@dataclass
class ConsensusResult:
    primary_answer: str                    # Best consensus answer
    confidence: float                      # Overall confidence (0.0-1.0)
    consensus_strength: float              # Agreement strength
    supporting_paths: List[ReasoningPath]  # Supporting reasoning paths
    alternative_answers: List[Tuple[str, float]]  # Alternative answers with confidence
    reasoning_explanation: str             # Human-readable explanation
```

### ReasoningPath

Complete reasoning chain from query to answer.

```python
@dataclass  
class ReasoningPath:
    query: str                    # Original query
    answer: str                   # Final answer
    steps: List[ReasoningStep]    # Reasoning steps
    confidence: float             # Path confidence
    total_time: float            # Processing time
```

### ReasoningStep

Single step in explainable reasoning chain (includes stable IDs).

```python
@dataclass
class ReasoningStep:
    source_concept: str    # Starting concept (label)
    relation: str          # Relationship type
    target_concept: str    # Target concept (label)
    confidence: float      # Step confidence
    step_number: int       # Step position
    source_id: Optional[str] = None  # Stable concept ID
    target_id: Optional[str] = None  # Stable concept ID
```

### Concept

Knowledge unit with adaptive strength learning.

```python
@dataclass
class Concept:
    id: str              # Unique identifier
    content: str         # Knowledge content
    strength: float      # Adaptive strength (1.0-10.0)
    confidence: float    # Content confidence
    access_count: int    # Usage frequency
    created: float       # Creation timestamp
    last_accessed: float # Last access time
    source: str          # Source attribution
    category: str        # Knowledge category
```

### Association

Weighted connection between concepts.

```python
@dataclass
class Association:
    source_id: str              # Source concept ID
    target_id: str              # Target concept ID
    assoc_type: AssociationType # Relationship type
    weight: float               # Connection strength
    confidence: float           # Relationship confidence
    created: float              # Creation timestamp
    last_used: float            # Last traversal/strengthen timestamp
```

### AssociationType

Types of relationships between concepts.

```python
class AssociationType(Enum):
    SEMANTIC = "semantic"        # Meaning-based connections
    CAUSAL = "causal"           # Cause and effect relationships
    TEMPORAL = "temporal"        # Time-based sequences
    HIERARCHICAL = "hierarchical" # Parent-child relationships
    COMPOSITIONAL = "compositional" # Part-whole relationships
```

## 🎛️ Configuration & Tuning

### Performance Tuning

```python
# Query caching for repeated questions (LRU + optional TTL)
ai = ReasoningEngine(enable_caching=True, max_cache_size=1000, cache_ttl_seconds=300)

# Path finding parameters
ai.path_finder.max_depth = 6              # Max reasoning depth
ai.path_finder.min_confidence = 0.1       # Min path confidence  
ai.path_finder.confidence_decay = 0.85    # Confidence decay factor

# Consensus parameters
ai.mppa.consensus_threshold = 0.5          # 50% agreement needed
ai.mppa.outlier_penalty = 0.3              # 30% penalty for outliers
```

### Memory Management

```python
# Optimize system performance
optimizations = ai.optimize_performance()

# Prune weak associations (< 0.1 confidence)
# Strengthen frequently accessed concepts (> 10 accesses)  
# Clean cache when full
```

### Learning Configuration

```python
# Adaptive learning thresholds
AdaptiveLearner.DIFFICULT_THRESHOLD = 4.0  # Concepts needing reinforcement
AdaptiveLearner.EASY_THRESHOLD = 7.0       # Well-established concepts

# Reinforcement multipliers  
AdaptiveLearner.DIFFICULT_MULTIPLIER = 1.15  # Strong reinforcement
AdaptiveLearner.EASY_MULTIPLIER = 1.01       # Minimal reinforcement
```

## 🚨 Error Handling

### Exception Hierarchy

```python
SutraError (base)
├── ConceptError          # Concept-related errors
├── AssociationError      # Association-related errors  
├── LearningError         # Learning process errors
├── ValidationError       # Data validation errors
├── StorageError         # Persistence errors
└── ConfigurationError    # Configuration errors
```

### Error Handling Examples

```python
from sutra_core import ReasoningEngine, LearningError, ConceptError

ai = ReasoningEngine()

try:
    ai.learn("Invalid concept content")
except LearningError as e:
    print(f"Learning failed: {e}")
    
try:
    result = ai.ask("Complex question")
    if result.confidence < 0.5:
        print("Warning: Low confidence answer")
except Exception as e:
    print(f"Query failed: {e}")
```

This API provides complete control over Sutra AI's advanced reasoning capabilities while maintaining simplicity for basic use cases.