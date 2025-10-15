# Sutra AI - Core Algorithms

**Version**: 1.0  
**Last Updated**: October 15, 2025

---

## Table of Contents

1. [Algorithm Overview](#algorithm-overview)
2. [Temporal Dynamics Algorithms](#temporal-dynamics-algorithms)
3. [Graph Traversal Algorithms](#graph-traversal-algorithms)
4. [Multi-Path Aggregation](#multi-path-aggregation)
5. [Query Planning Algorithms](#query-planning-algorithms)
6. [Contradiction Detection](#contradiction-detection)
7. [Learning Algorithms](#learning-algorithms)
8. [Optimization Algorithms](#optimization-algorithms)

---

## Algorithm Overview

> **Related Documentation**:  
> - [DESIGN.md](DESIGN.md) - Design rationale behind these algorithms  
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System components using these algorithms  
> - [CONTRIBUTING.md](CONTRIBUTING.md) - Testing and development guidelines

Sutra AI employs several classes of algorithms:

| Category | Purpose | Complexity | Key Feature |
|----------|---------|------------|-------------|
| Temporal | Concept evolution | O(1) per access | Adaptive strengthening |
| Traversal | Path finding | O(b^d) worst case | Multiple strategies |
| Aggregation | Consensus | O(n log n) | Clustering + voting |
| Planning | Query decomposition | O(n²) dependencies | Topological sort |
| Detection | Contradiction finding | O(n) per concept | Semantic analysis |
| Learning | Knowledge integration | O(m) associations | Adaptive depth |

---

## Temporal Dynamics Algorithms

### 1. Concept Strengthening

**Algorithm**: Exponential growth with cap

**Pseudocode**:
```
function strengthen_concept(concept):
    concept.access_count++
    concept.last_accessed = current_time()
    
    # Exponential growth with cap
    growth_rate = 1.02  # 2% per access
    cap = 10.0
    
    concept.strength = min(cap, concept.strength * growth_rate)
    
    return concept
```

**Mathematical Model**:
```
strength(n) = min(10.0, 1.0 × 1.02^n)

where n = number of accesses
```

**Convergence**:
- Approaches 10.0 asymptotically
- 95% of max reached at ~150 accesses
- Stable and predictable growth

**Why This Works**:
- Exponential captures frequency effects
- Cap prevents numerical instability
- Simple to implement and understand

### 2. Temporal Decay

**Algorithm**: Exponential decay based on inactivity

**Pseudocode**:
```
function decay_concepts(concepts, decay_rate=0.995):
    now = current_time()
    seconds_per_day = 86400
    min_strength = 0.1
    
    for concept in concepts:
        # Calculate inactivity
        inactive_days = (now - concept.last_accessed) / seconds_per_day
        
        if inactive_days >= threshold:
            # Apply exponential decay
            decay_steps = int(inactive_days - threshold) + 1
            concept.strength = max(
                min_strength,
                concept.strength * (decay_rate ^ decay_steps)
            )
```

**Mathematical Model**:
```
strength(t) = max(0.1, strength(0) × decay_rate^days)

where:
- t = days since last access
- decay_rate = 0.995 (default)
- min = 0.1 (floor)
```

**Decay Curve**:
```
Days    Retention
0       100%
30      86%
60      74%
90      64%
180     41%
365     17%
```

**Properties**:
- Mimics biological forgetting curves
- Reversible (re-access restores)
- Configurable per use case
- Never completely removes (floor at 0.1)

### 3. Adaptive Reinforcement

> See [DESIGN.md#decision-2-continuous-learning-architecture](DESIGN.md#decision-2-continuous-learning-architecture) for design rationale.

**Algorithm**: Difficulty-based learning rate

**Pseudocode**:
```
function adaptive_reinforce(concept):
    # Assess difficulty
    if concept.strength < 4.0:
        # Difficult concept - strong boost
        multiplier = 1.15
        depth = 2  # Deep extraction
    elif concept.strength > 7.0:
        # Easy concept - minimal boost
        multiplier = 1.01
        depth = 1  # Normal extraction
    else:
        # Moderate - standard boost
        multiplier = 1.02
        depth = 1
    
    concept.strength = min(10.0, concept.strength * multiplier)
    return depth  # For association extraction
```

**Inspiration**: AdaKD (Adaptive Knowledge Distillation)

**Key Innovation**: Use strength as proxy for difficulty (no loss function needed)

**Effect**:
- Struggling concepts get more attention
- Established concepts get less
- Self-balancing over time

---

## Graph Traversal Algorithms

> See [ARCHITECTURE.md#path-finder](ARCHITECTURE.md#path-finder) for architectural context.

### 1. Best-First Search

**Purpose**: Find high-confidence paths quickly

**Algorithm**:
```
function best_first_search(start, target, max_depth=6):
    # Priority queue: (negative_score, path_node)
    heap = [(1.0, PathNode(start, confidence=1.0, depth=0, path=[start]))]
    visited = set()
    paths = []
    
    while heap and len(paths) < k:
        neg_score, current = heap.pop()
        
        # Skip if visited (state = concept_id + recent history)
        state = (current.id, tuple(current.path[-3:]))
        if state in visited:
            continue
        visited.add(state)
        
        # Found target?
        if current.id == target:
            paths.append(construct_path(current))
            continue
        
        # Expand neighbors if within depth limit
        if current.depth < max_depth:
            for neighbor, association in get_neighbors(current.id):
                if neighbor in current.path:
                    continue  # Avoid cycles
                
                # Calculate new confidence
                new_conf = (current.confidence * 
                           association.confidence * 
                           decay_factor)
                
                # Heuristic boost if closer to target
                proximity = estimate_distance(neighbor, target)
                score = new_conf * (1.0 + proximity_bonus(proximity))
                
                new_node = extend_path(current, neighbor, new_conf)
                heap.push((-score, new_node))
    
    return paths
```

**Heuristic Function**:
```
proximity_bonus(neighbor, target):
    if neighbor == target:
        return 1.0
    if target in direct_neighbors(neighbor):
        return 0.5
    common = count_common_neighbors(neighbor, target)
    return 0.2 * min(1.0, common / 3)
```

**Properties**:
- Greedy but effective
- Fast for most queries
- Uses domain knowledge (neighbor overlap)
- Not guaranteed optimal

### 2. Breadth-First Search

**Purpose**: Find shortest paths

**Algorithm**:
```
function breadth_first_search(start, target, max_depth=6):
    queue = [PathNode(start, confidence=1.0, depth=0, path=[start])]
    visited = {start: 1.0}  # concept -> best confidence seen
    paths = []
    
    while queue and len(paths) < k:
        current = queue.pop_front()
        
        if current.id == target:
            paths.append(construct_path(current))
            continue
        
        if current.depth < max_depth:
            for neighbor, association in get_neighbors(current.id):
                if neighbor in current.path:
                    continue
                
                new_conf = (current.confidence * 
                           association.confidence * 
                           decay_factor)
                
                # Only visit if better than before
                if new_conf > visited.get(neighbor, 0):
                    visited[neighbor] = new_conf
                    new_node = extend_path(current, neighbor, new_conf)
                    queue.append(new_node)
    
    return paths
```

**Properties**:
- Explores level-by-level
- Guaranteed shortest path length
- Slower than best-first
- Good for relationship queries

### 3. Bidirectional Search

**Purpose**: Optimal path finding

**Algorithm**:
```
function bidirectional_search(start, target, max_depth=6):
    # Search from both ends
    forward = {start: PathNode(start, 1.0, 0, [start])}
    backward = {target: PathNode(target, 1.0, 0, [target])}
    
    forward_queue = [forward[start]]
    backward_queue = [backward[target]]
    
    paths = []
    half_depth = max_depth // 2 + 1
    
    for depth in range(half_depth):
        # Expand forward frontier
        expand_frontier(forward_queue, forward, depth, "forward")
        
        # Expand backward frontier
        expand_frontier(backward_queue, backward, depth, "backward")
        
        # Check for intersections
        meeting_points = set(forward.keys()) & set(backward.keys())
        
        for meet in meeting_points:
            if meet not in [start, target]:
                # Found connection!
                path = merge_paths(
                    forward[meet],
                    backward[meet],
                    start,
                    target
                )
                paths.append(path)
    
    return top_k_paths(paths, k=3)
```

**Merge Strategy**:
```
function merge_paths(forward_path, backward_path, start, target):
    # Combine paths at meeting point
    full_path = forward_path.path + reverse(backward_path.path[1:])
    
    # Combined confidence
    confidence = forward_path.confidence * backward_path.confidence
    
    return Path(start, target, full_path, confidence)
```

**Properties**:
- Optimal for known endpoints
- Faster than unidirectional for distant concepts
- More complex implementation
- Good for "how is A related to B?" queries

---

## Multi-Path Aggregation

### MPPA (Multi-Path Plan Aggregation)

**Purpose**: Consensus-based answer selection from multiple reasoning paths

**High-Level Algorithm**:
```
function aggregate_paths(paths, query):
    if paths is empty:
        return no_answer_result()
    
    # Step 1: Cluster by answer similarity
    clusters = cluster_by_answer(paths, similarity_threshold=0.8)
    
    # Step 2: Score each cluster
    scored_clusters = []
    for cluster in clusters:
        score = calculate_consensus_score(cluster, len(paths))
        scored_clusters.append((cluster, score))
    
    # Step 3: Rank by score
    ranked = sort_descending(scored_clusters, by=score)
    
    # Step 4: Extract result
    primary = ranked[0]
    alternatives = ranked[1:5]
    
    return ConsensusResult(
        primary_answer=primary.representative.answer,
        confidence=primary.cluster_confidence,
        consensus_strength=primary.consensus_weight,
        supporting_paths=primary.paths,
        alternatives=alternatives
    )
```

**Answer Clustering**:
```
function cluster_by_answer(paths, threshold=0.8):
    clusters = []
    
    for path in paths:
        normalized_answer = normalize(path.answer)
        
        # Find matching cluster
        matched_cluster = None
        for cluster in clusters:
            similarity = answer_similarity(
                normalized_answer,
                cluster.representative_answer
            )
            if similarity > threshold:
                matched_cluster = cluster
                break
        
        if matched_cluster:
            matched_cluster.add_path(path)
        else:
            clusters.append(new_cluster(path))
    
    return clusters
```

**Consensus Scoring**:
```
function calculate_consensus_score(cluster, total_paths):
    num_supporting = len(cluster.paths)
    
    # Base: fraction of paths supporting this answer
    path_support = num_supporting / total_paths
    
    # Cluster confidence: weighted average of path confidences
    cluster_conf = sum(p.confidence for p in cluster.paths) / num_supporting
    
    # Consensus bonus if above threshold
    consensus_boost = 1.0
    if path_support >= 0.5 and num_supporting >= 2:
        consensus_boost = 1.0 + (path_support - 0.5)
    
    # Outlier penalty for singleton paths
    outlier_penalty = 1.0
    if num_supporting == 1 and len(clusters) > 1:
        outlier_penalty = 0.7  # 30% penalty
    
    # Diversity bonus for varied reasoning
    diversity = calculate_diversity(cluster.paths)
    diversity_bonus = 1.0 + (diversity * 0.2)  # Up to 20% bonus
    
    # Final score
    return (cluster_conf * 
            path_support * 
            consensus_boost * 
            outlier_penalty * 
            diversity_bonus)
```

**Diversity Calculation**:
```
function calculate_diversity(paths):
    # Extract reasoning patterns (sequence of relation types)
    patterns = [tuple(step.relation for step in path.steps) 
                for path in paths]
    
    unique_patterns = len(set(patterns))
    max_diversity = min(len(paths), 4)  # Cap at 4
    
    return unique_patterns / max_diversity
```

**Properties**:
- Robust to single-path errors
- Rewards agreement
- Penalizes outliers
- Values diverse approaches
- O(n log n) complexity

---

## Query Planning Algorithms

### Query Decomposition

**Purpose**: Break complex queries into manageable steps

**Algorithm**:
```
function plan_query(query, max_steps=10):
    # Step 1: Classify query type
    intent = classify_intent(query)
    
    # Step 2: Pattern-based decomposition
    sub_questions = decompose_by_patterns(query, intent)
    
    if sub_questions is empty:
        # Simple query - no decomposition needed
        return single_step_plan(query, intent)
    
    # Step 3: Build steps
    steps = []
    for i, (sub_query, sub_type) in enumerate(sub_questions):
        step = QueryStep(
            id=i,
            query=sub_query,
            type=sub_type,
            dependencies=[]  # Will be set next
        )
        steps.append(step)
    
    # Step 4: Analyze dependencies
    steps = analyze_dependencies(steps)
    
    # Step 5: Create execution order (topological sort)
    order = topological_sort(steps)
    
    # Step 6: Estimate complexity
    complexity = estimate_complexity(steps)
    
    return QueryPlan(query, steps, order, complexity)
```

**Pattern-Based Decomposition**:
```
decomposition_patterns = [
    # Conjunctions
    (r"(.+?) and (.+)", "conjunction"),
    
    # Causal chains
    (r"why (.+)", "causal"),
    (r"what causes (.+)", "causal"),
    
    # Comparisons
    (r"compare (.+?) (?:and|to|with) (.+)", "comparative"),
    (r"difference between (.+?) and (.+)", "comparative"),
    
    # Temporal
    (r"what happens after (.+)", "temporal"),
    (r"what happens before (.+)", "temporal"),
    
    # Hypothetical
    (r"what if (.+)", "hypothetical"),
    (r"what would happen if (.+)", "hypothetical"),
]

function decompose_by_patterns(query):
    for pattern, type in decomposition_patterns:
        if match = regex_match(pattern, query):
            return generate_sub_questions(match, type)
    
    return None  # No decomposition
```

**Dependency Analysis**:
```
function analyze_dependencies(steps):
    # Simple strategy: sequential dependencies
    # (More sophisticated analysis possible)
    
    for i in range(1, len(steps)):
        # Each step depends on previous
        steps[i].dependencies = [i - 1]
    
    return steps
```

**Topological Sort**:
```
function topological_sort(steps):
    # Build in-degree map
    in_degree = {step.id: len(step.dependencies) for step in steps}
    
    # Queue of steps with no dependencies
    queue = [id for id, degree in in_degree if degree == 0]
    order = []
    
    while queue:
        current_id = queue.pop(0)
        order.append(current_id)
        
        # Reduce in-degree of dependents
        for step in steps:
            if current_id in step.dependencies:
                in_degree[step.id] -= 1
                if in_degree[step.id] == 0:
                    queue.append(step.id)
    
    # Check for cycles
    if len(order) != len(steps):
        # Cycle detected - fall back to sequential
        return [step.id for step in steps]
    
    return order
```

**Properties**:
- Handles complex multi-part queries
- Detects dependencies automatically
- Optimal execution ordering
- O(n²) worst case for dependencies

---

## Contradiction Detection

### Semantic Conflict Detection

**Purpose**: Identify contradictory concepts

**Algorithm**:
```
function detect_contradictions(new_concept, existing_concepts):
    contradictions = []
    
    for existing in recent_concepts(existing_concepts, limit=100):
        if existing.id == new_concept.id:
            continue
        
        # Check multiple conflict types
        conflict = check_conflicts(new_concept, existing)
        
        if conflict:
            contradictions.append(conflict)
    
    return contradictions
```

**Conflict Type Checks**:
```
function check_conflicts(concept1, concept2):
    content1 = normalize(concept1.content)
    content2 = normalize(concept2.content)
    
    # Type 1: Direct negation
    if has_direct_negation(content1, content2):
        return Contradiction(
            concept1.id, concept2.id,
            type=DIRECT,
            confidence=0.9
        )
    
    # Type 2: Semantic opposites
    if has_semantic_opposite(content1, content2):
        return Contradiction(
            concept1.id, concept2.id,
            type=SEMANTIC,
            confidence=0.7
        )
    
    # Type 3: Quantitative conflict
    if has_quantitative_conflict(content1, content2):
        return Contradiction(
            concept1.id, concept2.id,
            type=QUANTITATIVE,
            confidence=0.8
        )
    
    return None
```

**Direct Negation Detection**:
```
negation_keywords = {
    "not", "no", "never", "none", "nothing",
    "cannot", "can't", "won't", "don't", "isn't"
}

function has_direct_negation(content1, content2):
    words1 = tokenize(content1)
    words2 = tokenize(content2)
    
    # Check if one has negation, other doesn't
    has_neg1 = any(w in negation_keywords for w in words1)
    has_neg2 = any(w in negation_keywords for w in words2)
    
    if has_neg1 != has_neg2:
        # Remove negations and compare similarity
        clean1 = words1 - negation_keywords
        clean2 = words2 - negation_keywords
        
        overlap = len(clean1 & clean2)
        total = len(clean1 | clean2)
        
        if total > 0 and overlap / total > 0.6:
            return True
    
    return False
```

**Semantic Opposite Detection**:
```
opposite_pairs = {
    ("hot", "cold"), ("big", "small"), ("fast", "slow"),
    ("high", "low"), ("alive", "dead"), ("true", "false"),
    # ... more pairs
}

# Build lookup map
opposites_map = {}
for (a, b) in opposite_pairs:
    opposites_map[a] = b
    opposites_map[b] = a

function has_semantic_opposite(content1, content2):
    words1 = tokenize(content1)
    words2 = tokenize(content2)
    
    # Check for any opposite pairs
    for word1 in words1:
        if word1 in opposites_map:
            opposite = opposites_map[word1]
            if opposite in words2:
                return True
    
    return False
```

**Properties**:
- Three complementary detection methods
- Confidence scores reflect reliability
- Fast O(n) per concept check
- Extensible with more patterns

---

## Learning Algorithms

### Adaptive Association Extraction

**Purpose**: Extract relationships based on concept difficulty

**Algorithm**:
```
function learn_adaptive(content, source):
    # Step 1: Create or update concept
    concept_id = hash(content)
    
    if concept_id exists:
        concept = get_concept(concept_id)
        old_strength = concept.strength
        
        # Standard strengthening
        concept.access()
        
        # Apply adaptive reinforcement
        apply_adaptive_boost(concept)
    else:
        concept = create_concept(content, source)
        store_concept(concept)
    
    # Step 2: Determine extraction depth based on difficulty
    depth = get_extraction_depth(concept)
    
    # Step 3: Extract associations
    associations = extract_associations(content, concept_id, depth)
    
    return concept_id
```

**Extraction Depth Determination**:
```
function get_extraction_depth(concept):
    if concept.strength < 4.0:
        # Difficult - needs deep extraction
        return 2
    else:
        # Easy/moderate - standard extraction
        return 1
```

**Two-Stage Extraction**:
```
function extract_associations(content, concept_id, depth):
    count = 0
    
    # Stage 1: Pattern-based extraction (always)
    count += pattern_extraction(content, concept_id)
    
    # Stage 2: Co-occurrence (only if depth > 1)
    if depth > 1:
        count += cooccurrence_extraction(content, concept_id)
    
    return count
```

**Pattern Extraction**:
```
association_patterns = [
    (r"(.+?) causes (.+)", CAUSAL, 0.8),
    (r"(.+?) is (?:a|an) (.+)", HIERARCHICAL, 0.9),
    (r"(.+?) contains (.+)", COMPOSITIONAL, 0.9),
    (r"(.+?) similar to (.+)", SEMANTIC, 0.7),
    (r"(.+?) before (.+)", TEMPORAL, 0.8),
]

function pattern_extraction(content, central_id):
    count = 0
    
    for (pattern, type, confidence) in association_patterns:
        matches = find_all_matches(pattern, content)
        
        for (source_text, target_text) in matches:
            source_id = find_or_create_concept(source_text)
            target_id = find_or_create_concept(target_text)
            
            # Create association between extracted concepts
            create_association(source_id, target_id, type, confidence)
            count++
            
            # Optional: Link to central concept
            if enable_central_links:
                create_association(central_id, source_id, COMPOSITIONAL, 0.6)
                create_association(central_id, target_id, COMPOSITIONAL, 0.6)
    
    return count
```

**Properties**:
- Adaptive depth saves computation
- Pattern matching is precise
- Co-occurrence adds coverage
- Configurable trade-offs

---

## Optimization Algorithms

### LRU Cache with TTL

**Purpose**: Speed up repeated queries

**Algorithm**:
```
class LRUCache:
    def __init__(self, max_size=1000, ttl_seconds=None):
        self.cache = OrderedDict()  # Maintains insertion order
        self.max_size = max_size
        self.ttl = ttl_seconds
    
    function get(key):
        if key not in cache:
            return None
        
        value, timestamp = cache[key]
        
        # Check TTL if enabled
        if ttl and (current_time() - timestamp) > ttl:
            del cache[key]
            return None
        
        # Move to end (most recently used)
        cache.move_to_end(key)
        return value
    
    function put(key, value):
        # Evict oldest if at capacity
        while len(cache) >= max_size:
            oldest_key = next(iter(cache))  # First key
            del cache[oldest_key]
        
        # Insert with timestamp
        cache[key] = (value, current_time())
        cache.move_to_end(key)
```

**Properties**:
- O(1) get and put operations
- Optional TTL for freshness
- Size-bounded memory use
- OrderedDict provides LRU ordering

### Index Maintenance

**Purpose**: Fast concept lookup

**Algorithm**:
```
function maintain_indices(concepts, associations):
    # Index 1: Concept ID -> Offset
    concept_map = {}
    for concept in concepts:
        concept_map[concept.id] = concept
    
    # Index 2: Adjacency list
    adjacency = defaultdict(set)
    for (source, target), assoc in associations:
        adjacency[source].add(target)
        adjacency[target].add(source)  # Bidirectional
    
    # Index 3: Word -> Concepts
    word_index = defaultdict(set)
    for concept in concepts:
        words = extract_words(concept.content)
        for word in words:
            word_index[word.lower()].add(concept.id)
    
    return concept_map, adjacency, word_index
```

**Update Strategy**:
- Incremental updates during learning
- Full rebuild on load
- Background async rebuild option

---

## Algorithm Complexity Summary

| Algorithm | Time | Space | Notes |
|-----------|------|-------|-------|
| Concept Access | O(1) | O(1) | Hash lookup |
| Decay All | O(n) | O(1) | Linear scan |
| Best-First Search | O(b^d log b^d) | O(b^d) | Priority queue |
| BFS | O(b^d) | O(b^d) | Queue-based |
| Bidirectional | O(b^(d/2)) | O(b^(d/2)) | Exponential savings |
| MPPA | O(n log n) | O(n) | Sorting clusters |
| Query Planning | O(n²) | O(n) | Dependency analysis |
| Contradiction Detection | O(m) | O(1) | Per concept, m = recent concepts |
| Pattern Extraction | O(p × l) | O(k) | p=patterns, l=length, k=matches |
| LRU Cache | O(1) | O(c) | c=cache size |
| Index Rebuild | O(n × w) | O(n × w) | n=concepts, w=avg words |

**Key**:
- n = number of concepts
- m = number of concepts to check
- b = average branching factor
- d = search depth
- p = number of patterns
- l = content length
- w = average words per concept
- k = number of extracted associations
- c = cache size

---

##Conclusion

These algorithms work together to create a system that:
1. **Evolves over time** (temporal dynamics)
2. **Reasons robustly** (multi-path consensus)
3. **Handles complexity** (query planning)
4. **Detects conflicts** (contradiction resolution)
5. **Learns efficiently** (adaptive extraction)
6. **Performs well** (caching and indexing)

The key insight is that **simple algorithms composed well** can create sophisticated emergent behavior without the black-box complexity of neural networks.

---

**Related Documents**:
- ARCHITECTURE.md - System structure
- DESIGN.md - Design decisions
- Implementation files in `packages/sutra-core/sutra_core/`
