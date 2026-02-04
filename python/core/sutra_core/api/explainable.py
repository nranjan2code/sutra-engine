"""
Explainable AI API Layer

This module provides transparent access to the reasoning system's knowledge
and decision-making process, distinguishing it from black-box LLMs.

Key Features:
- Knowledge introspection (what does the system know?)
- Association exploration (how are concepts connected?)
- Reasoning transparency (why did it give this answer?)
- Confidence explanation (what drives confidence scores?)
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class ConceptInfo:
    """Detailed information about a concept."""

    id: str
    content: str
    confidence: float
    strength: float
    access_count: int
    created: float
    last_accessed: float
    source: Optional[str]
    category: Optional[str]
    neighbor_count: int
    neighbors: List[str]  # Concept IDs


@dataclass
class AssociationInfo:
    """Detailed information about an association."""

    source_id: str
    target_id: str
    source_content: str
    target_content: str
    association_type: str
    confidence: float
    weight: float
    created: float
    last_used: float


@dataclass
class ReasoningPathInfo:
    """Detailed information about a reasoning path."""

    path_id: int
    confidence: float
    steps: List[Dict[str, Any]]
    start_concept: str
    end_concept: str
    total_hops: int


@dataclass
class KnowledgeStats:
    """Statistics about the knowledge base."""

    total_concepts: int
    total_associations: int
    categories: Dict[str, int]
    sources: Dict[str, int]
    avg_confidence: float
    avg_concept_strength: float
    most_connected_concepts: List[Tuple[str, int]]


class ExplainableAI:
    """
    API for transparent AI reasoning and knowledge introspection.

    This class exposes the internal state and reasoning process of the
    AI system, enabling full explainability and trust.
    """

    def __init__(self, reasoning_engine):
        """Initialize with a ReasoningEngine instance."""
        self.engine = reasoning_engine

    # =========================================================================
    # KNOWLEDGE INTROSPECTION - "What does the system know?"
    # =========================================================================

    def list_all_concepts(
        self,
        limit: Optional[int] = None,
        category: Optional[str] = None,
        min_confidence: float = 0.0,
        sort_by: str = "strength",  # strength, confidence, access_count, created
    ) -> List[ConceptInfo]:
        """
        List all concepts in the knowledge base.

        Args:
            limit: Maximum number to return
            category: Filter by category
            min_confidence: Minimum confidence threshold
            sort_by: Sort criterion

        Returns:
            List of concept information
        """
        concepts = []

        for concept_id, concept in self.engine.concepts.items():
            # Apply filters
            if category and concept.category != category:
                continue
            if concept.confidence < min_confidence:
                continue

            neighbors = list(self.engine.concept_neighbors.get(concept_id, set()))

            concepts.append(
                ConceptInfo(
                    id=concept_id,
                    content=concept.content,
                    confidence=concept.confidence,
                    strength=concept.strength,
                    access_count=concept.access_count,
                    created=concept.created,
                    last_accessed=concept.last_accessed,
                    source=concept.source,
                    category=concept.category,
                    neighbor_count=len(neighbors),
                    neighbors=neighbors[:10],  # Limit for display
                )
            )

        # Sort
        if sort_by == "strength":
            concepts.sort(key=lambda c: c.strength, reverse=True)
        elif sort_by == "confidence":
            concepts.sort(key=lambda c: c.confidence, reverse=True)
        elif sort_by == "access_count":
            concepts.sort(key=lambda c: c.access_count, reverse=True)
        elif sort_by == "created":
            concepts.sort(key=lambda c: c.created, reverse=True)

        if limit:
            concepts = concepts[:limit]

        return concepts

    def get_concept_details(self, concept_id: str) -> Optional[ConceptInfo]:
        """Get detailed information about a specific concept."""
        if concept_id not in self.engine.concepts:
            return None

        concept = self.engine.concepts[concept_id]
        neighbors = list(self.engine.concept_neighbors.get(concept_id, set()))

        return ConceptInfo(
            id=concept_id,
            content=concept.content,
            confidence=concept.confidence,
            strength=concept.strength,
            access_count=concept.access_count,
            created=concept.created,
            last_accessed=concept.last_accessed,
            source=concept.source,
            category=concept.category,
            neighbor_count=len(neighbors),
            neighbors=neighbors,
        )

    def search_knowledge(
        self,
        query: str,
        limit: int = 10,
        method: str = "semantic",  # semantic or keyword
    ) -> List[ConceptInfo]:
        """
        Search the knowledge base.

        Args:
            query: Search query
            limit: Maximum results
            method: Search method (semantic uses embeddings, keyword uses text match)

        Returns:
            Ranked list of matching concepts
        """
        results = self.engine.search_concepts(query, limit=limit)

        concepts = []
        for result in results:
            concept_id = result["id"]
            concept = self.engine.concepts.get(concept_id)
            if not concept:
                continue

            neighbors = list(self.engine.concept_neighbors.get(concept_id, set()))

            concepts.append(
                ConceptInfo(
                    id=concept_id,
                    content=concept.content,
                    confidence=concept.confidence,
                    strength=concept.strength,
                    access_count=concept.access_count,
                    created=concept.created,
                    last_accessed=concept.last_accessed,
                    source=concept.source,
                    category=concept.category,
                    neighbor_count=len(neighbors),
                    neighbors=neighbors[:10],
                )
            )

        return concepts

    def get_knowledge_stats(self) -> KnowledgeStats:
        """Get comprehensive statistics about the knowledge base."""
        stats = self.engine.get_system_stats()
        system_info = stats.get("system_info", {})

        # Collect categories and sources
        categories = {}
        sources = {}
        total_confidence = 0.0
        total_strength = 0.0

        for concept in self.engine.concepts.values():
            if concept.category:
                categories[concept.category] = categories.get(concept.category, 0) + 1
            if concept.source:
                sources[concept.source] = sources.get(concept.source, 0) + 1
            total_confidence += concept.confidence
            total_strength += concept.strength

        num_concepts = len(self.engine.concepts)

        # Find most connected concepts
        connectivity = [
            (cid, len(neighbors))
            for cid, neighbors in self.engine.concept_neighbors.items()
        ]
        connectivity.sort(key=lambda x: x[1], reverse=True)

        return KnowledgeStats(
            total_concepts=num_concepts,
            total_associations=len(self.engine.associations),
            categories=categories,
            sources=sources,
            avg_confidence=total_confidence / num_concepts if num_concepts > 0 else 0,
            avg_concept_strength=(
                total_strength / num_concepts if num_concepts > 0 else 0
            ),
            most_connected_concepts=connectivity[:10],
        )

    # =========================================================================
    # ASSOCIATION EXPLORATION - "How are concepts connected?"
    # =========================================================================

    def get_associations_for_concept(
        self, concept_id: str, direction: str = "both"  # outgoing, incoming, both
    ) -> List[AssociationInfo]:
        """
        Get all associations for a concept.

        Args:
            concept_id: Concept to explore
            direction: Which associations to retrieve

        Returns:
            List of associations
        """
        associations = []

        for (src, tgt), assoc in self.engine.associations.items():
            include = False

            if direction == "outgoing" and src == concept_id:
                include = True
            elif direction == "incoming" and tgt == concept_id:
                include = True
            elif direction == "both" and (src == concept_id or tgt == concept_id):
                include = True

            if include:
                src_concept = self.engine.concepts.get(src)
                tgt_concept = self.engine.concepts.get(tgt)

                if src_concept and tgt_concept:
                    associations.append(
                        AssociationInfo(
                            source_id=src,
                            target_id=tgt,
                            source_content=src_concept.content,
                            target_content=tgt_concept.content,
                            association_type=assoc.assoc_type.value,
                            confidence=assoc.confidence,
                            weight=assoc.weight,
                            created=assoc.created,
                            last_used=assoc.last_used,
                        )
                    )

        return associations

    def explore_concept_neighborhood(
        self, concept_id: str, depth: int = 1, min_confidence: float = 0.5
    ) -> Dict[str, Any]:
        """
        Explore the neighborhood around a concept.

        Args:
            concept_id: Starting concept
            depth: How many hops to explore
            min_confidence: Minimum association confidence

        Returns:
            Graph structure with concepts and associations
        """
        visited = set()
        current_level = {concept_id}
        graph = {"center": concept_id, "concepts": {}, "associations": []}

        for level in range(depth):
            next_level = set()

            for cid in current_level:
                if cid in visited:
                    continue
                visited.add(cid)

                # Add concept
                concept = self.engine.concepts.get(cid)
                if concept:
                    graph["concepts"][cid] = {
                        "id": cid,
                        "content": concept.content,
                        "confidence": concept.confidence,
                        "level": level,
                    }

                # Add associations
                for (src, tgt), assoc in self.engine.associations.items():
                    if assoc.confidence < min_confidence:
                        continue

                    if src == cid:
                        next_level.add(tgt)
                        graph["associations"].append(
                            {
                                "source": src,
                                "target": tgt,
                                "type": assoc.assoc_type.value,
                                "confidence": assoc.confidence,
                            }
                        )
                    elif tgt == cid:
                        next_level.add(src)
                        graph["associations"].append(
                            {
                                "source": src,
                                "target": tgt,
                                "type": assoc.assoc_type.value,
                                "confidence": assoc.confidence,
                            }
                        )

            current_level = next_level

        return graph

    # =========================================================================
    # REASONING TRANSPARENCY - "Why did it give this answer?"
    # =========================================================================

    def explain_answer(
        self, question: str, answer_result: Optional[Any] = None
    ) -> Dict[str, Any]:
        """
        Explain how the system arrived at an answer.

        Args:
            question: The question asked
            answer_result: Optional pre-computed answer result

        Returns:
            Detailed explanation of reasoning process
        """
        if answer_result is None:
            answer_result = self.engine.ask(question, num_reasoning_paths=5)

        explanation = {
            "question": question,
            "answer": answer_result.primary_answer,
            "confidence": answer_result.confidence,
            "consensus_strength": answer_result.consensus_strength,
            "reasoning_steps": [],
            "concepts_used": [],
            "associations_used": [],
            "alternative_paths": [],
        }

        # Extract reasoning paths
        if hasattr(answer_result, "supporting_paths"):
            for i, path in enumerate(answer_result.supporting_paths):
                path_info = {
                    "path_number": i + 1,
                    "confidence": path.confidence,
                    "steps": [],
                }

                for step in path.steps:
                    step_info = {
                        "from": step.source_concept,
                        "to": step.target_concept,
                        "relation": step.relation,
                        "confidence": step.confidence,
                    }
                    path_info["steps"].append(step_info)

                    # Track concepts and associations used
                    if step.source_concept not in explanation["concepts_used"]:
                        explanation["concepts_used"].append(step.source_concept)
                    if step.target_concept not in explanation["concepts_used"]:
                        explanation["concepts_used"].append(step.target_concept)

                    explanation["associations_used"].append(
                        {
                            "from": step.source_concept,
                            "to": step.target_concept,
                            "type": step.relation,
                        }
                    )

                explanation["reasoning_steps"].append(path_info)

        # Alternative answers
        if hasattr(answer_result, "alternative_answers"):
            explanation["alternatives"] = answer_result.alternative_answers

        return explanation

    def trace_reasoning_path(
        self, start_concept_id: str, end_concept_id: str, num_paths: int = 3
    ) -> List[ReasoningPathInfo]:
        """
        Find and explain reasoning paths between two concepts.

        Args:
            start_concept_id: Starting concept
            end_concept_id: Target concept
            num_paths: Number of paths to find

        Returns:
            List of reasoning paths with details
        """
        paths = self.engine.path_finder.find_reasoning_paths(
            [start_concept_id], [end_concept_id], num_paths=num_paths
        )

        path_infos = []
        for i, path in enumerate(paths):
            steps = []
            for step in path.steps:
                src_concept = self.engine.concepts.get(step.source_concept)
                tgt_concept = self.engine.concepts.get(step.target_concept)

                steps.append(
                    {
                        "source_id": step.source_concept,
                        "target_id": step.target_concept,
                        "source_content": (
                            src_concept.content if src_concept else "Unknown"
                        ),
                        "target_content": (
                            tgt_concept.content if tgt_concept else "Unknown"
                        ),
                        "relation": step.relation,
                        "confidence": step.confidence,
                    }
                )

            path_infos.append(
                ReasoningPathInfo(
                    path_id=i,
                    confidence=path.confidence,
                    steps=steps,
                    start_concept=start_concept_id,
                    end_concept=end_concept_id,
                    total_hops=len(steps),
                )
            )

        return path_infos

    def explain_confidence(
        self, question: str, answer_result: Optional[Any] = None
    ) -> Dict[str, Any]:
        """
        Explain what drives the confidence score.

        Args:
            question: The question
            answer_result: Optional pre-computed result

        Returns:
            Breakdown of confidence factors
        """
        if answer_result is None:
            answer_result = self.engine.ask(question)

        # Analyze confidence factors
        path_confidences = []
        if hasattr(answer_result, "supporting_paths"):
            path_confidences = [p.confidence for p in answer_result.supporting_paths]

        return {
            "overall_confidence": answer_result.confidence,
            "consensus_strength": answer_result.consensus_strength,
            "num_supporting_paths": len(path_confidences),
            "path_confidences": path_confidences,
            "avg_path_confidence": (
                sum(path_confidences) / len(path_confidences) if path_confidences else 0
            ),
            "confidence_factors": {
                "path_agreement": answer_result.consensus_strength,
                "path_quality": max(path_confidences) if path_confidences else 0,
                "path_diversity": (
                    len(set(path_confidences)) / len(path_confidences)
                    if len(path_confidences) > 1
                    else 1
                ),
            },
        }

    # =========================================================================
    # INTERACTIVE EXPLORATION - "Let me navigate the knowledge"
    # =========================================================================

    def suggest_related_questions(
        self, concept_id_or_question: str, num_suggestions: int = 5
    ) -> List[str]:
        """
        Suggest related questions based on a concept or question.

        Args:
            concept_id_or_question: Concept ID or question text
            num_suggestions: Number of suggestions

        Returns:
            List of suggested questions
        """
        # Determine if input is concept ID or question
        if concept_id_or_question in self.engine.concepts:
            concept = self.engine.concepts[concept_id_or_question]
            base_content = concept.content
        else:
            # Search for relevant concepts
            results = self.engine.search_concepts(concept_id_or_question, limit=3)
            if not results:
                return []
            base_content = results[0]["content"]

        # Generate question templates
        suggestions = []
        templates = [
            f"What is {base_content[:50]}?",
            f"How does {base_content[:50]} work?",
            f"Why is {base_content[:50]} important?",
            f"What are examples of {base_content[:50]}?",
            f"How is {base_content[:50]} related to other concepts?",
        ]

        return templates[:num_suggestions]

    def compare_concepts(self, concept_id1: str, concept_id2: str) -> Dict[str, Any]:
        """
        Compare two concepts and explain their relationship.

        Args:
            concept_id1: First concept
            concept_id2: Second concept

        Returns:
            Comparison analysis
        """
        concept1 = self.engine.concepts.get(concept_id1)
        concept2 = self.engine.concepts.get(concept_id2)

        if not concept1 or not concept2:
            return {"error": "One or both concepts not found"}

        # Find direct association
        direct_assoc = self.engine.associations.get((concept_id1, concept_id2))
        reverse_assoc = self.engine.associations.get((concept_id2, concept_id1))

        # Find common neighbors
        neighbors1 = self.engine.concept_neighbors.get(concept_id1, set())
        neighbors2 = self.engine.concept_neighbors.get(concept_id2, set())
        common = neighbors1 & neighbors2

        # Find paths between them
        paths = self.trace_reasoning_path(concept_id1, concept_id2, num_paths=3)

        return {
            "concept1": {
                "id": concept_id1,
                "content": concept1.content,
                "confidence": concept1.confidence,
            },
            "concept2": {
                "id": concept_id2,
                "content": concept2.content,
                "confidence": concept2.confidence,
            },
            "direct_connection": {
                "exists": direct_assoc is not None or reverse_assoc is not None,
                "type": (
                    direct_assoc.assoc_type.value
                    if direct_assoc
                    else (reverse_assoc.assoc_type.value if reverse_assoc else None)
                ),
                "confidence": (
                    direct_assoc.confidence
                    if direct_assoc
                    else (reverse_assoc.confidence if reverse_assoc else 0)
                ),
            },
            "common_neighbors": len(common),
            "reasoning_paths": len(paths),
            "shortest_path_length": (
                min([p.total_hops for p in paths]) if paths else None
            ),
        }
