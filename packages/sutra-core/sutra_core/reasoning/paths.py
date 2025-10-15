"""
Advanced path-finding algorithms for graph reasoning.

Implements sophisticated graph traversal beyond basic spreading activation:
- Multi-hop inference with confidence decay
- Bidirectional search for optimal paths
- Constraint-based path filtering
- Dynamic path pruning for efficiency
"""

import heapq
import logging
import time
from collections import deque
from dataclasses import dataclass
from typing import Deque, Dict, List, Optional, Set, Tuple

from ..graph.concepts import Association, Concept, ReasoningPath, ReasoningStep

logger = logging.getLogger(__name__)


@dataclass
class PathNode:
    """Node in reasoning path with accumulated confidence."""

    concept_id: str
    confidence: float
    depth: int
    path_history: List[str]
    total_score: float

    def __lt__(self, other: "PathNode") -> bool:
        """Enable heap comparison by total score."""
        return self.total_score < other.total_score


class PathFinder:
    """Advanced path-finding for multi-hop reasoning."""

    def __init__(
        self,
        concepts: Dict[str, Concept],
        associations: Dict[Tuple[str, str], Association],
        concept_neighbors: Dict[str, Set[str]],
        max_depth: int = 5,
        min_confidence: float = 0.1,
        confidence_decay: float = 0.95,
        use_harmonic_mean: bool = True,
    ):
        """
        Initialize path finder.

        Args:
            concepts: All concepts in the graph
            associations: All associations
            concept_neighbors: Adjacency list for fast traversal
            max_depth: Maximum reasoning depth
            min_confidence: Minimum confidence threshold
            confidence_decay: Decay factor per hop (ignored if use_harmonic_mean=True)
            use_harmonic_mean: Use harmonic mean for confidence (better for long paths)
        """
        self.concepts = concepts
        self.associations = associations
        self.concept_neighbors = concept_neighbors
        self.max_depth = max_depth
        self.min_confidence = min_confidence
        self.confidence_decay = confidence_decay
        self.use_harmonic_mean = use_harmonic_mean
        """
        Initialize path finder.

        Args:
            concepts: All concepts in the system
            associations: All associations between concepts
            concept_neighbors: Neighbor mapping for efficient traversal
        """
        self.concepts = concepts
        self.associations = associations
        self.concept_neighbors = concept_neighbors

        # Performance settings
        self.max_depth = 6
        self.min_confidence = 0.1
        self.confidence_decay = 0.85
        self.max_paths_per_search = 50

    def find_reasoning_paths(
        self,
        start_concepts: List[str],
        target_concepts: List[str],
        num_paths: int = 3,
        search_strategy: str = "best_first",
    ) -> List[ReasoningPath]:
        """
        Find multiple reasoning paths between concept sets.

        Args:
            start_concepts: Starting concept IDs
            target_concepts: Target concept IDs
            num_paths: Number of diverse paths to find
            search_strategy: 'best_first', 'breadth_first', or 'bidirectional'

        Returns:
            List of reasoning paths with confidence scores
        """
        logger.debug(
            f"Finding {num_paths} paths from {len(start_concepts)} "
            f"starts to {len(target_concepts)} targets"
        )

        all_paths = []

        # Try each start-target combination
        for start_id in start_concepts:
            for target_id in target_concepts:
                if start_id == target_id:
                    continue

                paths = self._find_paths_between_concepts(
                    start_id, target_id, search_strategy
                )
                all_paths.extend(paths)

        # Diversify and rank paths
        diverse_paths = self._diversify_paths(all_paths, num_paths)

        logger.debug(f"Found {len(diverse_paths)} diverse reasoning paths")
        return diverse_paths

    def _find_paths_between_concepts(
        self, start_id: str, target_id: str, strategy: str = "best_first"
    ) -> List[ReasoningPath]:
        """Find paths between two specific concepts."""

        if strategy == "best_first":
            return self._best_first_search(start_id, target_id)
        elif strategy == "breadth_first":
            return self._breadth_first_search(start_id, target_id)
        elif strategy == "bidirectional":
            return self._bidirectional_search(start_id, target_id)
        else:
            raise ValueError(f"Unknown search strategy: {strategy}")

    def _best_first_search(self, start_id: str, target_id: str) -> List[ReasoningPath]:
        """Best-first search with confidence-based scoring."""

        paths_found: List[ReasoningPath] = []
        # Priority queue: (-score, path_node)
        heap: List[Tuple[float, PathNode]] = [(-1.0, PathNode(start_id, 1.0, 0, [start_id], 1.0))]
        visited_states: Set[Tuple[str, Tuple[str, ...]]] = set()

        while heap and len(paths_found) < 3:
            neg_score, current = heapq.heappop(heap)

            # Skip if we've seen this state
            state_key = (current.concept_id, tuple(current.path_history[-3:]))
            if state_key in visited_states:
                continue
            visited_states.add(state_key)

            # Check if we reached target
            if current.concept_id == target_id:
                path = self._create_reasoning_path(current)
                paths_found.append(path)
                continue

            # Expand if within depth limit
            if current.depth < self.max_depth:
                neighbors = self.concept_neighbors.get(current.concept_id, set())

                for neighbor_id in neighbors:
                    if neighbor_id in current.path_history:
                        continue  # Avoid cycles

                    # Calculate transition confidence
                    association = self.associations.get(
                        (current.concept_id, neighbor_id)
                    )
                    if not association:
                        continue

                    # Mark edge as used
                    association.last_used = time.time()

                    # Calculate confidence with harmonic mean or multiplicative decay
                    new_confidence = self._propagate_confidence(
                        current.confidence, 
                        association.confidence,
                        current.depth + 1
                    )

                    if new_confidence < self.min_confidence:
                        continue

                    # Boost score if closer to target (heuristic)
                    target_boost = self._calculate_target_proximity(
                        neighbor_id, target_id
                    )

                    new_score = new_confidence * (1.0 + target_boost)

                    new_path = PathNode(
                        concept_id=neighbor_id,
                        confidence=new_confidence,
                        depth=current.depth + 1,
                        path_history=current.path_history + [neighbor_id],
                        total_score=new_score,
                    )

                    heapq.heappush(heap, (-new_score, new_path))

        return paths_found

    def _breadth_first_search(
        self, start_id: str, target_id: str
    ) -> List[ReasoningPath]:
        """Breadth-first search for shortest paths."""

        paths_found: List[ReasoningPath] = []
        queue: Deque[PathNode] = deque([PathNode(start_id, 1.0, 0, [start_id], 1.0)])
        visited: Dict[str, float] = {start_id: 1.0}  # concept_id -> best_confidence

        while queue and len(paths_found) < 3:
            current = queue.popleft()

            if current.concept_id == target_id:
                path = self._create_reasoning_path(current)
                paths_found.append(path)
                continue

            if current.depth < self.max_depth:
                neighbors = self.concept_neighbors.get(current.concept_id, set())

                for neighbor_id in neighbors:
                    if neighbor_id in current.path_history:
                        continue

                    association = self.associations.get(
                        (current.concept_id, neighbor_id)
                    )
                    if not association:
                        continue

                    # Mark edge as used
                    association.last_used = time.time()

                    new_confidence = (
                        current.confidence
                        * association.confidence
                        * self.confidence_decay
                    )

                    # Only proceed if better than previous visit
                    if (
                        neighbor_id in visited
                        and new_confidence <= visited[neighbor_id]
                    ):
                        continue

                    if new_confidence < self.min_confidence:
                        continue

                    visited[neighbor_id] = new_confidence

                    new_path = PathNode(
                        concept_id=neighbor_id,
                        confidence=new_confidence,
                        depth=current.depth + 1,
                        path_history=current.path_history + [neighbor_id],
                        total_score=new_confidence,
                    )

                    queue.append(new_path)

        return paths_found

    def _bidirectional_search(
        self, start_id: str, target_id: str
    ) -> List[ReasoningPath]:
        """Bidirectional search for optimal paths."""

        # Forward search from start
        forward_visited = {start_id: PathNode(start_id, 1.0, 0, [start_id], 1.0)}
        forward_queue = deque([forward_visited[start_id]])

        # Backward search from target
        backward_visited = {target_id: PathNode(target_id, 1.0, 0, [target_id], 1.0)}
        backward_queue = deque([backward_visited[target_id]])

        paths_found = []
        max_depth_each = self.max_depth // 2 + 1

        for depth in range(max_depth_each):
            # Expand forward frontier
            if forward_queue:
                self._expand_bidirectional_frontier(
                    forward_queue, forward_visited, depth, "forward"
                )

            # Expand backward frontier
            if backward_queue:
                self._expand_bidirectional_frontier(
                    backward_queue, backward_visited, depth, "backward"
                )

            # Check for intersections
            intersections = set(forward_visited.keys()) & set(backward_visited.keys())
            for meeting_point in intersections:
                if meeting_point not in [start_id, target_id]:
                    path = self._merge_bidirectional_paths(
                        forward_visited[meeting_point],
                        backward_visited[meeting_point],
                        start_id,
                        target_id,
                    )
                    if path:
                        paths_found.append(path)

        return paths_found[:3]  # Return top 3 paths

    def _expand_bidirectional_frontier(
        self, queue: Deque[PathNode], visited: Dict[str, PathNode], depth: int, direction: str
    ) -> None:
        """
        Expand one step in bidirectional search.
        
        BUGFIX: Previously filtered by depth incorrectly, dropping valid paths.
        Now processes entire queue for current depth level.
        """
        next_queue: Deque[PathNode] = deque()

        while queue:
            current = queue.popleft()

            # Process all nodes at current depth (BUGFIX: was filtering incorrectly)
            if current.depth != depth:
                # Re-add to queue if not at target depth yet
                next_queue.append(current)
                continue

            neighbors = self.concept_neighbors.get(current.concept_id, set())

            for neighbor_id in neighbors:
                if neighbor_id in current.path_history:
                    continue

                # Get association (consider direction)
                if direction == "forward":
                    association = self.associations.get(
                        (current.concept_id, neighbor_id)
                    )
                else:
                    association = self.associations.get(
                        (neighbor_id, current.concept_id)
                    )

                if not association:
                    continue

                # Mark edge as used
                association.last_used = time.time()

                # Use harmonic mean confidence propagation (OPTIMIZATION)
                new_confidence = self._propagate_confidence(
                    current.confidence,
                    association.confidence,
                    current.depth + 1
                )

                if new_confidence < self.min_confidence:
                    continue

                if (
                    neighbor_id not in visited
                    or new_confidence > visited[neighbor_id].confidence
                ):

                    new_path = PathNode(
                        concept_id=neighbor_id,
                        confidence=new_confidence,
                        depth=current.depth + 1,
                        path_history=current.path_history + [neighbor_id],
                        total_score=new_confidence,
                    )

                    visited[neighbor_id] = new_path
                    next_queue.append(new_path)

        # Replace queue with next level (BUGFIX: proper frontier expansion)
        queue.clear()
        queue.extend(next_queue)

    def _calculate_target_proximity(self, concept_id: str, target_id: str) -> float:
        """Heuristic: estimate proximity to target concept."""

        if concept_id == target_id:
            return 1.0

        # Simple heuristic based on direct connections
        if target_id in self.concept_neighbors.get(concept_id, set()):
            return 0.5

        # Check for common neighbors
        concept_neighbors = self.concept_neighbors.get(concept_id, set())
        target_neighbors = self.concept_neighbors.get(target_id, set())
        common = len(concept_neighbors & target_neighbors)

        if common > 0:
            return 0.2 * min(1.0, common / 3.0)

        return 0.0

    def _create_reasoning_path(self, path_node: PathNode) -> ReasoningPath:
        """Convert PathNode to ReasoningPath."""

        steps = []
        for i in range(len(path_node.path_history) - 1):
            source_id = path_node.path_history[i]
            target_id = path_node.path_history[i + 1]

            association = self.associations.get((source_id, target_id))

            step = ReasoningStep(
                source_concept=self.concepts[source_id].content[:50] + "...",
                relation=association.assoc_type.value if association else "related",
                target_concept=self.concepts[target_id].content[:50] + "...",
                confidence=association.confidence if association else 0.5,
                step_number=i + 1,
                source_id=source_id,
                target_id=target_id,
            )
            steps.append(step)

        return ReasoningPath(
            query="Path reasoning",
            answer=self.concepts[path_node.concept_id].content,
            steps=steps,
            confidence=path_node.confidence,
            total_time=0.0,  # Will be set by caller
        )

    def _merge_bidirectional_paths(
        self,
        forward_path: PathNode,
        backward_path: PathNode,
        start_id: str,
        target_id: str,
    ) -> Optional[ReasoningPath]:
        """Merge forward and backward paths at meeting point."""

        # Combine path histories
        forward_history = forward_path.path_history
        backward_history = backward_path.path_history[::-1]  # Reverse

        # Remove duplicate meeting point
        if forward_history[-1] == backward_history[0]:
            full_path = forward_history + backward_history[1:]
        else:
            full_path = forward_history + backward_history

        # Calculate combined confidence
        combined_confidence = forward_path.confidence * backward_path.confidence

        # Create merged path node
        merged_node = PathNode(
            concept_id=target_id,
            confidence=combined_confidence,
            depth=len(full_path) - 1,
            path_history=full_path,
            total_score=combined_confidence,
        )

        return self._create_reasoning_path(merged_node)

    def _diversify_paths(
        self, all_paths: List[ReasoningPath], num_paths: int
    ) -> List[ReasoningPath]:
        """Select diverse high-quality paths."""

        if not all_paths:
            return []

        # Sort by confidence
        sorted_paths = sorted(all_paths, key=lambda p: p.confidence, reverse=True)

        # Greedy diversification
        selected = [sorted_paths[0]] if sorted_paths else []

        for path in sorted_paths[1:]:
            if len(selected) >= num_paths:
                break

            # Check diversity vs existing paths
            is_diverse = True
            for selected_path in selected:
                overlap = self._calculate_path_overlap(path, selected_path)
                if overlap > 0.7:  # Too similar
                    is_diverse = False
                    break

            if is_diverse:
                selected.append(path)

        return selected

    def _propagate_confidence(
        self, current_conf: float, edge_conf: float, depth: int
    ) -> float:
        """
        Propagate confidence through reasoning path.
        
        OPTIMIZATION: Uses harmonic mean instead of pure multiplication to avoid
        confidence collapse on long paths. With multiplication, 10 hops at 0.85
        each gives 0.20 confidence. With harmonic mean, maintains ~0.60.
        
        Args:
            current_conf: Current path confidence
            edge_conf: Edge/association confidence
            depth: Current depth in path
            
        Returns:
            New confidence score
        """
        if self.use_harmonic_mean:
            # Harmonic mean: 2xy/(x+y) with gentle penalty for depth
            # Better for preserving confidence in multi-hop reasoning
            if current_conf + edge_conf == 0:
                return 0.0
            
            harmonic = (2 * current_conf * edge_conf) / (current_conf + edge_conf)
            
            # Gentle depth penalty (1% per hop) instead of aggressive decay
            depth_penalty = 0.99 ** depth
            
            return harmonic * depth_penalty
        else:
            # Original multiplicative decay (prone to collapse)
            return current_conf * edge_conf * self.confidence_decay

    def select_diverse_paths(
        self, all_paths: List[ReasoningPath], num_paths: int
    ) -> List[ReasoningPath]:
        """Public API to select a diverse subset of paths."""
        return self._diversify_paths(all_paths, num_paths)

    def _calculate_path_overlap(
        self, path1: ReasoningPath, path2: ReasoningPath
    ) -> float:
        """Calculate overlap between two reasoning paths.
        Uses stable concept IDs when available, otherwise falls back to labels."""

        def pairs(path: ReasoningPath) -> Set[Tuple[str, str]]:
            out: Set[Tuple[str, str]] = set()
            for s in path.steps:
                a = getattr(s, "source_id", None) or s.source_concept
                b = getattr(s, "target_id", None) or s.target_concept
                out.add((a, b))
            return out

        steps1 = pairs(path1)
        steps2 = pairs(path2)

        if not steps1 or not steps2:
            return 0.0

        intersection = len(steps1 & steps2)
        union = len(steps1 | steps2)

        return intersection / union if union > 0 else 0.0
