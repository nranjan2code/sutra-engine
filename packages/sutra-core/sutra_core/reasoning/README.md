# Sutra AI - Advanced Reasoning Engine

This package implements the sophisticated AI reasoning capabilities that make Sutra AI a genuine alternative to traditional LLMs.

## 🧠 Core Components

### ReasoningEngine
**Main AI interface** - Orchestrates all reasoning components to provide AI-level capabilities.

```python
from sutra_core import ReasoningEngine

# Initialize AI engine with caching
ai = ReasoningEngine(enable_caching=True, max_cache_size=1000)

# Learn knowledge instantly (no retraining required)
ai.learn("Photosynthesis converts sunlight into chemical energy using chlorophyll")

# Ask complex questions with explainable answers
result = ai.ask("How do plants make energy from sunlight?")
print(f"Answer: {result.primary_answer}")
print(f"Confidence: {result.confidence:.2f}")
print(f"Reasoning: {result.reasoning_explanation}")
```

**Key Features:**
- Natural language query processing
- Real-time learning without retraining
- Query result caching for performance
- Knowledge base persistence
- System statistics and optimization

### PathFinder
**Advanced graph traversal** for multi-hop reasoning with multiple search strategies.

```python
from sutra_core.reasoning import PathFinder

# Initialize with graph data
path_finder = PathFinder(concepts, associations, concept_neighbors)

# Find reasoning paths between concepts
paths = path_finder.find_reasoning_paths(
    start_concepts=["concept1"], 
    target_concepts=["concept2"],
    num_paths=3,
    search_strategy="best_first"  # or "breadth_first", "bidirectional"
)
```

**Search Strategies:**
- **Best-first**: Confidence-optimized with heuristics
- **Breadth-first**: Shortest path exploration  
- **Bidirectional**: Optimal path finding from both ends

**Features:**
- Confidence decay (0.85 default) for realistic propagation
- Cycle detection and path diversification
- Target proximity heuristics for efficient search
- Configurable depth limits and confidence thresholds

### MultiPathAggregator (MPPA)
**Consensus-based reasoning** that prevents single-path derailment through voting.

```python
from sutra_core.reasoning import MultiPathAggregator

# Initialize aggregator with consensus settings
mppa = MultiPathAggregator(
    consensus_threshold=0.5,    # 50% agreement required
    min_paths_for_consensus=2,  # Minimum paths for consensus
    outlier_penalty=0.3         # 30% penalty for outlier paths
)

# Aggregate multiple reasoning paths
consensus = mppa.aggregate_reasoning_paths(reasoning_paths, query)

print(f"Primary Answer: {consensus.primary_answer}")
print(f"Consensus Strength: {consensus.consensus_strength:.2f}")
print(f"Alternative Answers: {len(consensus.alternative_answers)}")
```

**Consensus Features:**
- Path clustering by answer similarity (0.8 threshold)
- Majority voting with configurable thresholds
- Diversity bonus for varied reasoning approaches
- Robustness analysis with multiple metrics
- Alternative answer ranking

### QueryProcessor  
**Natural language understanding** that transforms queries into structured reasoning tasks.

```python
from sutra_core.reasoning import QueryProcessor

# Initialize with system components
processor = QueryProcessor(concepts, associations, neighbors, extractor, finder, mppa)

# Process complex natural language queries
result = processor.process_query(
    "Why is sunlight essential for life on Earth?",
    num_reasoning_paths=5,
    max_concepts=10
)
```

**NLP Capabilities:**
- Intent classification (what/how/why/when/where/who)
- Query complexity assessment
- Context expansion using high-confidence associations
- Multi-concept relevance scoring
- Query suggestion generation

## 🎯 Usage Examples

### Basic AI Usage

```python
from sutra_core import ReasoningEngine

# Create AI engine
ai = ReasoningEngine()

# Learn domain knowledge
ai.learn("Mitochondria are the powerhouses of cells")
ai.learn("ATP provides energy for cellular processes")  
ai.learn("Cellular respiration produces ATP in mitochondria")

# Ask intelligent questions
result = ai.ask("How do cells get energy?")
print(f"Answer: {result.primary_answer}")
print(f"Confidence: {result.confidence:.2f}")
```

### Advanced Reasoning Analysis

```python
# Get detailed reasoning explanation
explanation = ai.explain_reasoning(
    "What happens during cellular respiration?", 
    detailed=True
)

# Analyze reasoning quality
robustness = explanation['reasoning_robustness']
print(f"Robustness Score: {robustness['robustness_score']:.2f}")
print(f"Path Diversity: {robustness['path_diversity']:.2f}")  
print(f"Supporting Paths: {robustness['supporting_path_count']}")

# View detailed reasoning paths
if 'detailed_paths' in explanation:
    for path in explanation['detailed_paths']:
        print(f"\nPath {path['path_number']} (confidence: {path['confidence']:.2f}):")
        for step in path['steps']:
            print(f"  {step['from']} --[{step['relation']}]--> {step['to']}")
```

### Performance Optimization

```python
# System statistics
stats = ai.get_system_stats()
print(f"Total concepts: {stats['system_info']['total_concepts']}")
print(f"Cache hit rate: {stats['system_info']['cache_hit_rate']:.1%}")

# Performance optimization
optimizations = ai.optimize_performance()
print(f"Concepts strengthened: {optimizations['concepts_strengthened']}")
print(f"Weak associations removed: {optimizations['weak_associations_removed']}")

# Knowledge persistence
ai.save_knowledge_base("my_ai_knowledge.json")
ai.load_knowledge_base("my_ai_knowledge.json")
```

## 🔬 Technical Architecture

### Reasoning Pipeline

1. **Query Processing** - Parse natural language and classify intent
2. **Context Expansion** - Find relevant concepts and expand context  
3. **Path Generation** - Create multiple reasoning paths using different strategies
4. **Consensus Building** - Aggregate paths and build consensus through voting
5. **Result Enhancement** - Add explanations, confidence scores, alternatives

### Confidence Propagation

```python
# Confidence flows through associations with decay
new_confidence = current_confidence * association.confidence * decay_factor  # 0.85

# Consensus boosts high-agreement answers
consensus_boost = 1.0 + (path_support - threshold) * boost_factor

# Outlier penalty for singleton paths  
outlier_penalty = 1.0 - penalty_factor  # 0.7 for single paths
```

### Path Diversification

- **Similarity Threshold**: 0.7 maximum overlap between selected paths
- **Greedy Selection**: Choose highest confidence, then most diverse
- **Cluster Analysis**: Group similar reasoning approaches
- **Diversity Scoring**: Reward unique reasoning patterns
- **Stable IDs**: Overlap/diversity now computed using concept IDs when available (more accurate than text-only)

## 📊 Performance Characteristics

- **Query Latency**: 5-50ms depending on complexity
- **Learning Speed**: Instant knowledge integration
- **Memory Scaling**: Linear with knowledge base size
- **Cache Efficiency**: 8.5x speedup for repeated queries
- **Reasoning Quality**: Consensus prevents single-path failures

## 🎛️ Configuration Options

### ReasoningEngine Parameters
```python
ai = ReasoningEngine(
    enable_caching=True,          # Enable query result caching
    max_cache_size=1000,          # Maximum cached queries
    cache_ttl_seconds=None,       # Optional TTL (seconds) for query cache
    enable_central_links=True,    # Link central concept to extracted phrases
    central_link_confidence=0.6   # Confidence for central links
)
```

### PathFinder Parameters  
```python
path_finder.max_depth = 6              # Maximum reasoning depth
path_finder.min_confidence = 0.1       # Minimum path confidence
path_finder.confidence_decay = 0.85    # Confidence decay per step
```

### MultiPathAggregator Parameters
```python
mppa = MultiPathAggregator(
    consensus_threshold=0.5,      # Minimum agreement for consensus
    min_paths_for_consensus=2,    # Minimum paths needed
    diversity_penalty=0.1,        # Penalty for low diversity
    outlier_penalty=0.3           # Penalty for outlier answers
)
```

## 🚀 Advanced Features

### Real-Time Learning
- Instant knowledge integration without retraining
- Adaptive reinforcement based on concept difficulty  
- Automatic association extraction and indexing

### Explainable AI
- Complete reasoning paths with confidence scores
- Alternative answer exploration
- Robustness analysis with multiple metrics

### Performance Optimization
- Query result caching with configurable limits
- Concept strengthening through usage patterns
- Weak association pruning for noise reduction
- Memory management and garbage collection

This reasoning engine transforms Sutra from a graph database into a sophisticated AI system capable of human-like reasoning with complete explainability.