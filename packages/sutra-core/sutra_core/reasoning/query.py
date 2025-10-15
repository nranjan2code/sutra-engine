"""
Advanced query processing for natural language understanding.

Transforms natural language queries into structured reasoning tasks:
- Intent recognition and query classification
- Concept extraction and relevance scoring
- Query expansion and semantic enrichment
- Multi-step reasoning orchestration
"""

import logging
import time
from collections import defaultdict
from typing import Dict, List, Set, Tuple

from ..graph.concepts import Association, Concept
from ..learning.associations import AssociationExtractor
from ..utils.text import clean_text, extract_words
from .mppa import ConsensusResult, MultiPathAggregator
from .paths import PathFinder

logger = logging.getLogger(__name__)


class QueryProcessor:
    """
    Advanced natural language query processing for AI reasoning.

    Handles complex queries by:
    - Understanding user intent and query type
    - Finding relevant concepts and expanding context
    - Orchestrating multi-path reasoning
    - Generating explainable answers with confidence scores
    """

    def __init__(
        self,
        concepts: Dict[str, Concept],
        associations: Dict[Tuple[str, str], Association],
        concept_neighbors: Dict[str, Set[str]],
        association_extractor: AssociationExtractor,
        path_finder: PathFinder,
        mppa: MultiPathAggregator,
    ):
        """
        Initialize query processor.

        Args:
            concepts: All concepts in the system
            associations: All associations
            concept_neighbors: Neighbor mapping
            association_extractor: For concept finding
            path_finder: For reasoning path generation
            mppa: For consensus aggregation
        """
        self.concepts = concepts
        self.associations = associations
        self.concept_neighbors = concept_neighbors
        self.association_extractor = association_extractor
        self.path_finder = path_finder
        self.mppa = mppa

        # Query type patterns
        self.question_patterns = {
            "what": ["what is", "what are", "what does", "what causes"],
            "how": ["how does", "how do", "how can", "how to"],
            "why": ["why does", "why do", "why is", "why are"],
            "when": ["when does", "when do", "when is", "when are"],
            "where": ["where is", "where are", "where does"],
            "who": ["who is", "who are", "who does"],
        }

    def process_query(
        self, query: str, num_reasoning_paths: int = 5, max_concepts: int = 10
    ) -> ConsensusResult:
        """
        Process natural language query and generate AI reasoning response.

        Args:
            query: Natural language question
            num_reasoning_paths: Number of reasoning paths to explore
            max_concepts: Maximum relevant concepts to consider

        Returns:
            Consensus result with explainable reasoning
        """
        start_time = time.time()

        logger.debug(f"Processing query: {query}")

        # Step 1: Clean and normalize query
        cleaned_query = clean_text(query.lower())

        # Step 2: Classify query intent
        query_intent = self._classify_query_intent(cleaned_query)

        # Step 3: Extract and rank relevant concepts
        relevant_concepts = self._find_relevant_concepts(cleaned_query, max_concepts)

        if not relevant_concepts:
            return ConsensusResult(
                primary_answer="I don't have enough knowledge to answer this question.",
                confidence=0.0,
                consensus_strength=0.0,
                supporting_paths=[],
                alternative_answers=[],
                reasoning_explanation=f"No relevant concepts found for: {query}",
            )

        # Step 4: Expand context with related concepts
        expanded_concepts = self._expand_query_context(relevant_concepts, query_intent)

        # Step 5: Generate multiple reasoning paths
        reasoning_paths = self._generate_reasoning_paths(
            expanded_concepts, query_intent, num_reasoning_paths
        )

        # Step 6: Aggregate paths using MPPA
        consensus_result = self.mppa.aggregate_reasoning_paths(reasoning_paths, query)

        # Step 7: Enhance result with query-specific information
        enhanced_result = self._enhance_consensus_result(
            consensus_result, query, query_intent, start_time
        )

        processing_time = time.time() - start_time
        logger.debug(
            (
                f"Query processed in {processing_time:.3f}s with "
                f"{len(reasoning_paths)} paths"
            )
        )

        return enhanced_result

    def _classify_query_intent(self, query: str) -> Dict[str, any]:
        """Classify the intent and type of the query."""

        intent = {
            "type": "unknown",
            "confidence": 0.0,
            "focus": None,
            "seeking": "information",
        }

        # Check for question word patterns
        for question_type, patterns in self.question_patterns.items():
            for pattern in patterns:
                if pattern in query:
                    intent["type"] = question_type
                    intent["confidence"] = 0.8
                    break
            if intent["confidence"] > 0:
                break

        # Determine what the query is seeking
        if any(word in query for word in ["define", "definition", "meaning"]):
            intent["seeking"] = "definition"
        elif any(word in query for word in ["cause", "reason", "why"]):
            intent["seeking"] = "causation"
        elif any(word in query for word in ["process", "how", "steps"]):
            intent["seeking"] = "process"
        elif any(word in query for word in ["example", "instance"]):
            intent["seeking"] = "examples"
        elif any(word in query for word in ["compare", "difference", "versus"]):
            intent["seeking"] = "comparison"

        # Extract focus keywords (most important concepts)
        query_words = extract_words(query)
        important_words = [w for w in query_words if len(w) > 3][:3]
        intent["focus"] = important_words

        return intent

    def _find_relevant_concepts(
        self, query: str, max_concepts: int
    ) -> List[Tuple[str, float]]:
        """Find and rank concepts relevant to the query."""

        query_words = set(extract_words(query))
        concept_scores = defaultdict(float)

        # Score concepts by word overlap
        for concept_id, concept in self.concepts.items():
            concept_words = set(extract_words(concept.content.lower()))

            # Direct word matches
            matches = query_words & concept_words
            if matches:
                base_score = len(matches) / max(len(query_words), 1)

                # Boost by concept strength and confidence
                strength_boost = min(concept.strength / 5.0, 1.0)
                confidence_boost = concept.confidence

                final_score = base_score * strength_boost * confidence_boost
                concept_scores[concept_id] = final_score

        # Sort by relevance score
        ranked_concepts = sorted(
            concept_scores.items(), key=lambda x: x[1], reverse=True
        )

        return ranked_concepts[:max_concepts]

    def _expand_query_context(
        self, relevant_concepts: List[Tuple[str, float]], query_intent: Dict
    ) -> List[str]:
        """Expand query context with related concepts."""

        expanded = set()

        # Add primary relevant concepts
        for concept_id, score in relevant_concepts:
            expanded.add(concept_id)

        # Add strongly connected neighbors
        for concept_id, score in relevant_concepts[:3]:  # Top 3 concepts
            neighbors = self.concept_neighbors.get(concept_id, set())

            # Add high-confidence neighbors
            for neighbor_id in neighbors:
                if neighbor_id in self.concepts:
                    association = self.associations.get((concept_id, neighbor_id))
                    if association and association.confidence >= 0.6:
                        expanded.add(neighbor_id)

                    # Also check reverse direction
                    reverse_association = self.associations.get(
                        (neighbor_id, concept_id)
                    )
                    if reverse_association and reverse_association.confidence >= 0.6:
                        expanded.add(neighbor_id)

        # Limit expansion to prevent explosion
        expanded_list = list(expanded)[:15]

        logger.debug(
            f"Expanded {len(relevant_concepts)} concepts to {len(expanded_list)}"
        )
        return expanded_list

    def _generate_reasoning_paths(
        self, concepts: List[str], query_intent: Dict, num_paths: int
    ) -> List:
        """Generate multiple reasoning paths for the query."""

        if len(concepts) < 2:
            return []

        # Select start and target concepts based on query intent
        start_concepts = concepts[:3]  # Most relevant
        target_concepts = concepts[1:6] if len(concepts) > 3 else concepts[1:]

        # Generate diverse reasoning paths
        all_paths = []

        # Try different search strategies
        strategies = ["best_first", "breadth_first", "bidirectional"]
        paths_per_strategy = max(1, num_paths // len(strategies))

        for strategy in strategies:
            try:
                strategy_paths = self.path_finder.find_reasoning_paths(
                    start_concepts,
                    target_concepts,
                    num_paths=paths_per_strategy,
                    search_strategy=strategy,
                )
                all_paths.extend(strategy_paths)

                # Break if we have enough paths
                if len(all_paths) >= num_paths:
                    break

            except Exception as e:
                logger.warning(f"Strategy {strategy} failed: {e}")
                continue

        # Diversify and limit paths centrally via PathFinder
        return self.path_finder.select_diverse_paths(all_paths, num_paths)

    def _enhance_consensus_result(
        self,
        result: ConsensusResult,
        original_query: str,
        query_intent: Dict,
        start_time: float,
    ) -> ConsensusResult:
        """Enhance consensus result with query-specific information."""

        processing_time = time.time() - start_time

        # Enhance the explanation with query context
        enhanced_explanation = (
            f"Query type: {query_intent['type']} (seeking {query_intent['seeking']}). "
            + result.reasoning_explanation
            + f" Processing completed in {processing_time:.2f}s."
        )

        # Adjust confidence based on query complexity
        complexity_factor = self._assess_query_complexity(original_query, query_intent)
        adjusted_confidence = max(0.0, min(1.0, result.confidence * complexity_factor))

        return ConsensusResult(
            primary_answer=result.primary_answer,
            confidence=adjusted_confidence,
            consensus_strength=result.consensus_strength,
            supporting_paths=result.supporting_paths,
            alternative_answers=result.alternative_answers,
            reasoning_explanation=enhanced_explanation,
        )

    def _assess_query_complexity(self, query: str, query_intent: Dict) -> float:
        """Assess query complexity and adjust confidence accordingly."""

        base_factor = 1.0

        # Simple queries get confidence boost
        if (
            query_intent["type"] in ["what", "who"]
            and query_intent["seeking"] == "definition"
        ):
            base_factor = 1.1

        # Complex multi-part queries get penalty
        if len(query.split()) > 10:
            base_factor *= 0.95

        # Questions seeking causation are inherently harder
        if query_intent["seeking"] == "causation":
            base_factor *= 0.9

        # Comparison queries are complex
        if query_intent["seeking"] == "comparison":
            base_factor *= 0.85

        return base_factor

    def get_query_suggestions(
        self, partial_query: str, max_suggestions: int = 5
    ) -> List[str]:
        """Generate query suggestions based on partial input."""

        suggestions = []
        query_words = extract_words(partial_query.lower())

        if not query_words:
            return []

        # Find concepts matching query words
        matching_concepts = []
        for concept_id, concept in self.concepts.items():
            concept_words = extract_words(concept.content.lower())
            if any(word in concept_words for word in query_words):
                matching_concepts.append(concept)

        # Generate suggestions based on common query patterns
        for concept in matching_concepts[:3]:
            content_snippet = (
                concept.content[:30] + "..."
                if len(concept.content) > 30
                else concept.content
            )

            suggestions.extend(
                [
                    f"What is {content_snippet}?",
                    f"How does {content_snippet} work?",
                    f"Why is {content_snippet} important?",
                ]
            )

        # Remove duplicates and limit
        unique_suggestions = list(dict.fromkeys(suggestions))
        return unique_suggestions[:max_suggestions]
