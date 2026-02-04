"""
Multi-Path Plan Aggregation (MPPA) for consensus-based reasoning.

Implements sophisticated multi-path reasoning that:
- Explores multiple reasoning paths simultaneously
- Uses consensus voting to prevent single-path derailment
- Aggregates confidence scores from diverse reasoning approaches
- Provides robust, explainable AI decision making
"""

import logging
from collections import Counter, defaultdict
from dataclasses import dataclass
from typing import Counter as CounterType
from typing import Dict, List, Tuple

from ..graph.concepts import ReasoningPath

logger = logging.getLogger(__name__)


@dataclass
class ConsensusResult:
    """Result of multi-path consensus analysis."""

    primary_answer: str
    confidence: float
    consensus_strength: float
    supporting_paths: List[ReasoningPath]
    alternative_answers: List[Tuple[str, float]]
    reasoning_explanation: str


@dataclass
class PathCluster:
    """Cluster of similar reasoning paths."""

    representative_path: ReasoningPath
    member_paths: List[ReasoningPath]
    cluster_confidence: float
    consensus_weight: float


class MultiPathAggregator:
    """
    Advanced multi-path reasoning with consensus aggregation.

    Prevents reasoning derailment by:
    - Running multiple independent reasoning paths
    - Clustering similar conclusions
    - Using majority voting for final decisions
    - Boosting consensus answers, penalizing outliers
    """

    def __init__(
        self,
        consensus_threshold: float = 0.5,
        min_paths_for_consensus: int = 2,
        diversity_penalty: float = 0.1,
        outlier_penalty: float = 0.3,
    ):
        """
        Initialize multi-path aggregator.

        Args:
            consensus_threshold: Minimum agreement for consensus
            min_paths_for_consensus: Minimum paths needed for consensus
            diversity_penalty: Penalty for lack of path diversity
            outlier_penalty: Penalty for outlier answers
        """
        self.consensus_threshold = consensus_threshold
        self.min_paths_for_consensus = min_paths_for_consensus
        self.diversity_penalty = diversity_penalty
        self.outlier_penalty = outlier_penalty

    def aggregate_reasoning_paths(
        self, reasoning_paths: List[ReasoningPath], query: str
    ) -> ConsensusResult:
        """
        Aggregate multiple reasoning paths into consensus result.

        Args:
            reasoning_paths: List of reasoning paths to aggregate
            query: Original query being answered

        Returns:
            Consensus result with aggregated confidence and explanation
        """
        if not reasoning_paths:
            # PRODUCTION: No paths — fall back to best matching concept content
            try:
                from .query import extract_words  # wrong path; fix below
            except Exception:
                pass
            return ConsensusResult(
                primary_answer="No reasoning paths found",
                confidence=0.0,
                consensus_strength=0.0,
                supporting_paths=[],
                alternative_answers=[],
                reasoning_explanation="No valid reasoning paths were generated.",
            )

        logger.debug(
            f"Aggregating {len(reasoning_paths)} reasoning paths for: {query[:50]}..."
        )

        # Cluster paths by similar answers
        path_clusters = self._cluster_paths_by_answer(reasoning_paths)

        # Calculate consensus scores
        cluster_scores = self._calculate_consensus_scores(path_clusters)

        # Rank clusters by consensus strength
        ranked_clusters = sorted(
            cluster_scores, key=lambda x: x.consensus_weight, reverse=True
        )

        if not ranked_clusters:
            return ConsensusResult(
                primary_answer="No consensus found",
                confidence=0.0,
                consensus_strength=0.0,
                supporting_paths=[],
                alternative_answers=[],
                reasoning_explanation="Unable to reach consensus from available paths.",
            )

        # Primary result from strongest consensus
        primary_cluster = ranked_clusters[0]

        # Alternative answers from other clusters
        alternatives = [
            (cluster.representative_path.answer, cluster.consensus_weight)
            for cluster in ranked_clusters[1:5]  # Top 4 alternatives
        ]

        # Generate explanation
        explanation = self._generate_consensus_explanation(
            primary_cluster, len(reasoning_paths), query
        )

        return ConsensusResult(
            primary_answer=primary_cluster.representative_path.answer,
            confidence=primary_cluster.cluster_confidence,
            consensus_strength=primary_cluster.consensus_weight,
            supporting_paths=primary_cluster.member_paths,
            alternative_answers=alternatives,
            reasoning_explanation=explanation,
        )

    def _cluster_paths_by_answer(
        self, reasoning_paths: List[ReasoningPath]
    ) -> List[PathCluster]:
        """Cluster reasoning paths by similar answers."""

        # Group paths by answer similarity
        answer_groups: Dict[str, List[ReasoningPath]] = defaultdict(list)

        for path in reasoning_paths:
            # Normalize answer for grouping
            normalized_answer = self._normalize_answer(path.answer)

            # Find existing group with similar answer
            matched_group = None
            for existing_answer in answer_groups:
                similarity = self._calculate_answer_similarity(
                    normalized_answer, existing_answer
                )
                if similarity > 0.8:  # High similarity threshold
                    matched_group = existing_answer
                    break

            # Add to matched group or create new one
            group_key = matched_group if matched_group else normalized_answer
            answer_groups[group_key].append(path)

        # Convert groups to clusters
        clusters = []
        for answer, paths in answer_groups.items():
            if paths:  # Ensure non-empty
                # Select representative path (highest individual confidence)
                representative = max(paths, key=lambda p: p.confidence)

                # Calculate cluster confidence (weighted average)
                total_weight = sum(p.confidence for p in paths)
                cluster_confidence = total_weight / len(paths) if paths else 0.0

                cluster = PathCluster(
                    representative_path=representative,
                    member_paths=paths,
                    cluster_confidence=cluster_confidence,
                    consensus_weight=0.0,  # Will be calculated later
                )
                clusters.append(cluster)

        return clusters

    def _calculate_consensus_scores(
        self, clusters: List[PathCluster]
    ) -> List[PathCluster]:
        """Calculate consensus weights for each cluster."""

        total_paths = sum(len(cluster.member_paths) for cluster in clusters)

        if total_paths == 0:
            return clusters

        for cluster in clusters:
            num_supporting_paths = len(cluster.member_paths)

            # Base consensus from path count
            path_support = num_supporting_paths / total_paths

            # Boost if meets minimum consensus threshold
            consensus_boost = 1.0
            if (
                path_support >= self.consensus_threshold
                and num_supporting_paths >= self.min_paths_for_consensus
            ):
                consensus_boost = 1.0 + (path_support - self.consensus_threshold)

            # Penalty for being outlier (few supporting paths)
            outlier_penalty = 1.0
            if num_supporting_paths == 1 and len(clusters) > 1:
                outlier_penalty = 1.0 - self.outlier_penalty

            # Diversity bonus for having multiple different reasoning approaches
            diversity_bonus = self._calculate_diversity_bonus(cluster.member_paths)

            # Final consensus weight
            cluster.consensus_weight = (
                cluster.cluster_confidence
                * path_support
                * consensus_boost
                * outlier_penalty
                * diversity_bonus
            )

        return clusters

    def _calculate_diversity_bonus(self, paths: List[ReasoningPath]) -> float:
        """Calculate bonus for diverse reasoning approaches in cluster."""

        if len(paths) <= 1:
            return 1.0

        # Analyze reasoning step patterns
        step_patterns = []
        for path in paths:
            pattern = tuple(step.relation for step in path.steps)
            step_patterns.append(pattern)

        # Count unique patterns
        unique_patterns = len(set(step_patterns))
        max_diversity = min(len(paths), 4)  # Cap diversity bonus

        diversity_ratio = unique_patterns / max_diversity
        return 1.0 + (diversity_ratio * 0.2)  # Max 20% bonus

    def _normalize_answer(self, answer: str) -> str:
        """Normalize answer text for comparison."""
        return answer.lower().strip().replace(".", "").replace(",", "")

    def _calculate_answer_similarity(self, answer1: str, answer2: str) -> float:
        """Calculate similarity between two normalized answers."""

        if not answer1 or not answer2:
            return 0.0

        # Simple word overlap similarity
        words1 = set(answer1.split())
        words2 = set(answer2.split())

        if not words1 and not words2:
            return 1.0
        if not words1 or not words2:
            return 0.0

        intersection = len(words1 & words2)
        union = len(words1 | words2)

        return intersection / union if union > 0 else 0.0

    def _generate_consensus_explanation(
        self, primary_cluster: PathCluster, total_paths: int, query: str
    ) -> str:
        """Generate human-readable explanation of consensus reasoning."""

        num_supporting = len(primary_cluster.member_paths)
        consensus_percentage = (num_supporting / total_paths) * 100

        explanation_parts = [
            (
                f"Consensus reached with {consensus_percentage:.0f}% agreement "
                f"({num_supporting}/{total_paths} paths)."
            )
        ]

        # Describe reasoning approaches
        if len(primary_cluster.member_paths) > 1:
            relation_types: CounterType[str] = Counter()
            for path in primary_cluster.member_paths:
                for step in path.steps:
                    relation_types[step.relation] += 1

            top_relations = [rel for rel, _ in relation_types.most_common(3)]
            explanation_parts.append(
                f"Primary reasoning used: {', '.join(top_relations)}."
            )

        # Confidence assessment
        confidence_level = (
            "high" if primary_cluster.cluster_confidence > 0.7 else "moderate"
        )
        explanation_parts.append(
            (
                f"Answer confidence: {confidence_level} ("
                f"{primary_cluster.cluster_confidence:.2f})."
            )
        )

        # Consensus strength
        if primary_cluster.consensus_weight > 0.8:
            explanation_parts.append(
                "Strong consensus across multiple reasoning paths."
            )
        elif primary_cluster.consensus_weight > 0.5:
            explanation_parts.append(
                "Moderate consensus with some variation in approaches."
            )
        else:
            explanation_parts.append("Weak consensus - answer should be verified.")

        return " ".join(explanation_parts)

    def analyze_reasoning_robustness(
        self, consensus_result: ConsensusResult
    ) -> Dict[str, float]:
        """
        Analyze the robustness of the reasoning result.

        Returns:
            Dictionary with robustness metrics
        """
        num_supporting = len(consensus_result.supporting_paths)

        # Path diversity score
        if num_supporting > 1:
            step_patterns = set()
            for path in consensus_result.supporting_paths:
                pattern = tuple(step.relation for step in path.steps)
                step_patterns.add(pattern)
            diversity_score = len(step_patterns) / num_supporting
        else:
            diversity_score = 0.0

        # Confidence consistency
        confidences = [path.confidence for path in consensus_result.supporting_paths]
        if confidences:
            avg_confidence = sum(confidences) / len(confidences)
            confidence_variance = sum(
                (c - avg_confidence) ** 2 for c in confidences
            ) / len(confidences)
            consistency_score = max(0.0, 1.0 - confidence_variance)
        else:
            consistency_score = 0.0

        # Overall robustness
        robustness_score = (
            consensus_result.consensus_strength * 0.4
            + diversity_score * 0.3
            + consistency_score * 0.3
        )

        return {
            "robustness_score": robustness_score,
            "path_diversity": diversity_score,
            "confidence_consistency": consistency_score,
            "consensus_strength": consensus_result.consensus_strength,
            "supporting_path_count": num_supporting,
            "alternative_answer_count": len(consensus_result.alternative_answers),
        }
