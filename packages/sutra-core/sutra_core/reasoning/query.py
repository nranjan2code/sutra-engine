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
from typing import Any, Dict, List, Optional, Set, Tuple

from ..graph.concepts import Association, Concept, ReasoningPath
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
        storage,
        association_extractor: AssociationExtractor,
        path_finder: PathFinder,
        mppa: MultiPathAggregator,
        embedding_processor: Optional[Any] = None,
        nlp_processor: Optional[Any] = None,
    ):
        """
        Initialize query processor.

        Args:
            storage: RustStorageAdapter with native HNSW vector search
            association_extractor: For concept finding
            path_finder: For reasoning path generation
            mppa: For consensus aggregation
            embedding_processor: Optional batch embedding processor
            nlp_processor: Optional NLP processor for embeddings
        """
        self.storage = storage
        self.association_extractor = association_extractor
        self.path_finder = path_finder
        self.mppa = mppa

        # Performance components for semantic search
        self.embedding_processor = embedding_processor
        self.nlp_processor = nlp_processor

        # Query type patterns (order matters: check specific phrases first)
        self.question_patterns = {
            "what": ["what is", "what are", "what does", "what causes", "what"],
            "how": ["how does", "how do", "how can", "how to", "how"],
            "why": ["why does", "why do", "why is", "why are", "why"],
            "when": ["when does", "when do", "when is", "when are", "when"],
            "where": ["where is", "where are", "where does", "where"],
            "who": [
                "who is",
                "who are",
                "who does",
                "who created",
                "who made",
                "who invented",
                "who",
            ],
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
            expanded_concepts, query_intent, num_reasoning_paths, cleaned_query
        )

        # PRODUCTION: Check if all paths are trivial self-loops (no real associations)
        # This happens when knowledge graph has no associations yet
        all_self_loops = True
        if reasoning_paths:
            for path in reasoning_paths:
                if len(path.steps) > 1 or (
                    len(path.steps) == 1
                    and path.steps[0].source_concept != path.steps[0].target_concept
                ):
                    all_self_loops = False
                    break

        # If no paths or only trivial self-loops, use vector search result directly
        if not reasoning_paths or all_self_loops:
            logger.debug(
                f"No meaningful reasoning paths ({len(reasoning_paths)} trivial paths), using best vector search result"
            )
            best_concept = self.storage.get_concept(relevant_concepts[0][0])
            if best_concept:
                # Extract targeted answer from concept content
                # Pass similarity score for intelligent extraction
                similarity_score = relevant_concepts[0][1]
                answer = self._extract_targeted_answer(
                    best_concept.content,
                    cleaned_query,
                    query_intent,
                    similarity_score=similarity_score,
                )
                return ConsensusResult(
                    primary_answer=answer,
                    confidence=best_concept.confidence,  # Full confidence for vector search
                    consensus_strength=0.0,  # No consensus from single concept
                    supporting_paths=[],
                    alternative_answers=[],
                    reasoning_explanation=f"Direct semantic match (similarity: {relevant_concepts[0][1]:.2f})",
                )

        # Step 6: Aggregate paths using MPPA (only when we have meaningful paths)
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

    def _classify_query_intent(self, query: str) -> Dict[str, Any]:
        """Classify the intent and type of the query."""

        intent: Dict[str, Any] = {
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
            if float(intent["confidence"]) > 0:
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
        """Find and rank concepts relevant to the query.

        PRODUCTION: Vector search is the ONLY path. No fallbacks, no hacks.
        Embeddings naturally handle query variations, no expansion needed.
        Uses native HNSW in storage for O(log N) semantic search.
        """
        return self._find_concepts_semantic(query, max_concepts)

    def _find_concepts_semantic(
        self, query: str, max_concepts: int
    ) -> List[Tuple[str, float]]:
        """Find concepts using semantic vector search (O(log N)).

        PRODUCTION: Pure vector search, no fallbacks, no hacks.
        Trust embeddings to handle semantic similarity correctly.
        """
        import numpy as np

        # Generate query embedding
        query_embedding = None

        # Try embedding processor first (faster, batched)
        # Use 'Retrieval-query' prompt for queries (not documents)
        if self.embedding_processor:
            try:
                query_embedding = self.embedding_processor.encode_single(
                    query, prompt_name="Retrieval-query"
                )
                logger.debug(
                    f"Query embedding via EmbeddingGemma: dim={len(query_embedding) if query_embedding is not None else 0}"
                )
            except Exception as e:
                logger.warning(f"Batch embedding failed: {e}")

        # PRODUCTION FALLBACK: Use configured NLP processor (OllamaNLPProcessor)
        if query_embedding is None and self.nlp_processor:
            try:
                query_embedding = self.nlp_processor.get_embedding(query)
                logger.debug(
                    f"Query embedding via configured NLP processor (nomic-embed-text): dim={len(query_embedding) if query_embedding is not None else 0}"
                )
            except Exception as e:
                logger.warning(f"NLP embedding failed: {e}")

        # No embedding = hard fail (embeddings are mandatory)
        if query_embedding is None:
            raise RuntimeError(
                "No embedding processor available. "
                "Vector search requires embedding_processor or nlp_processor."
            )

        # Ensure numpy array
        if not isinstance(query_embedding, np.ndarray):
            query_embedding = np.array(query_embedding, dtype=np.float32)

        # Search using native HNSW in storage (O(log N))
        vector_results = self.storage.vector_search(
            query_embedding,
            k=max_concepts,
        )

        # Score with concept metadata (strength, confidence)
        # NO lexical boost, NO word matching merge - trust embeddings
        relevant_concepts = []
        for concept_id, similarity in vector_results:
            concept = self.storage.get_concept(concept_id)
            if concept:
                # Weight by concept quality metrics
                strength_boost = min(concept.strength / 5.0, 1.0)
                confidence_boost = concept.confidence
                final_score = similarity * strength_boost * confidence_boost
                relevant_concepts.append((concept_id, final_score))
                logger.debug(
                    f"  {concept_id[:8]}: sim={similarity:.3f}, strength={concept.strength:.2f}, "
                    f"conf={concept.confidence:.2f}, final={final_score:.3f} | {concept.content[:60]}"
                )

        # Sort and return
        relevant_concepts.sort(key=lambda x: x[1], reverse=True)
        logger.debug(f"Vector search found {len(relevant_concepts)} concepts")
        return relevant_concepts

    def _expand_query_context(
        self, relevant_concepts: List[Tuple[str, float]], query_intent: Dict[str, Any]
    ) -> List[str]:
        """Expand query context with related concepts."""

        expanded = set()

        # Add primary relevant concepts
        for concept_id, score in relevant_concepts:
            expanded.add(concept_id)

        # Add strongly connected neighbors
        for concept_id, score in relevant_concepts[:3]:  # Top 3 concepts
            neighbors = self.storage.get_neighbors(concept_id)
            if not neighbors:
                continue

            # Add high-confidence neighbors
            for neighbor_id in neighbors:
                association = self.storage.get_association(concept_id, neighbor_id)
                if association and association.confidence >= 0.6:
                    expanded.add(neighbor_id)

                # Also check reverse direction
                reverse_association = self.storage.get_association(
                    neighbor_id, concept_id
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
        self,
        concepts: List[str],
        query_intent: Dict[str, Any],
        num_paths: int,
        query: str = "",
    ) -> List[Any]:
        """Generate multiple reasoning paths for the query using storage graph."""

        if len(concepts) < 2:
            return []

        # Select start and target concepts based on query intent
        # IMPORTANT: Include the most relevant concept (concepts[0]) in targets!
        start_concepts = concepts[:3]
        target_concepts = concepts[:5] if len(concepts) > 3 else concepts

        try:
            # Pass full query to find_paths for intelligent answer extraction
            paths = self.storage.find_paths(
                start_concepts,
                target_concepts,
                max_depth=5,
                num_paths=num_paths,
                query=query,
            )
            return paths
        except Exception as e:
            logger.warning(f"Storage-backed pathfinding failed: {e}")
            return []

    def _enhance_consensus_result(
        self,
        result: ConsensusResult,
        original_query: str,
        query_intent: Dict[str, Any],
        start_time: float,
    ) -> ConsensusResult:
        """Enhance consensus result with query-specific information.

        PRODUCTION: Extract semantically targeted answers based on question type.
        """

        processing_time = time.time() - start_time

        # PRODUCTION: Extract targeted answer based on query type
        # For MPPA consensus, use conservative similarity (lower confidence in extraction)
        enhanced_answer = self._extract_targeted_answer(
            result.primary_answer,
            original_query,
            query_intent,
            similarity_score=0.5,  # Conservative for aggregated results
        )
        if enhanced_answer != result.primary_answer:
            logger.debug(
                f"Answer extraction: '{result.primary_answer}' → '{enhanced_answer}'"
            )

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
            primary_answer=enhanced_answer,  # Use enhanced answer
            confidence=adjusted_confidence,
            consensus_strength=result.consensus_strength,
            supporting_paths=result.supporting_paths,
            alternative_answers=result.alternative_answers,
            reasoning_explanation=enhanced_explanation,
        )

    def _extract_targeted_answer(
        self,
        answer: str,
        query: str,
        query_intent: Dict[str, Any],
        similarity_score: float = 1.0,
    ) -> str:
        """PRODUCTION-GRADE hybrid answer extraction.

        Strategy:
        1. High similarity? Trust embeddings, return full content
        2. Try intelligent extraction for common patterns
        3. If extraction incomplete/generic, return full content

        This handles all human query variations without brittleness.

        Examples:
        - "Who created Python?" → "Guido van Rossum" (NER extraction)
        - "What is the largest ocean?" → Full content (extraction would lose "Pacific")
        - "When did WWII end?" → "1945" (date extraction)
        """
        import re

        qtype = query_intent.get("type", "unknown")
        logger.debug(
            f"Hybrid extraction: qtype={qtype}, similarity={similarity_score:.2f}, answer='{answer[:60]}'..."
        )

        # STRATEGY 1: High similarity = trust embeddings, return full answer
        # The embeddings already solved the hard problem!
        if similarity_score >= 0.7:
            logger.debug("High similarity - returning full content")
            return answer

        # STRATEGY 2: Intelligent extraction for common patterns
        extracted = None

        # WHO questions: Extract person/entity names using spaCy NER
        if qtype == "who" and self.nlp_processor:
            try:
                doc = self.nlp_processor.nlp(answer)
                persons = [ent.text for ent in doc.ents if ent.label_ == "PERSON"]
                orgs = [ent.text for ent in doc.ents if ent.label_ == "ORG"]
                entities = persons + orgs
                if entities:
                    extracted = entities[0]
                    logger.debug(f"NER extraction: {extracted}")
            except Exception as e:
                logger.debug(f"NER failed: {e}")

        # WHEN questions: Extract dates/years using spaCy or regex
        elif qtype == "when":
            if self.nlp_processor:
                try:
                    doc = self.nlp_processor.nlp(answer)
                    dates = [ent.text for ent in doc.ents if ent.label_ == "DATE"]
                    if dates:
                        extracted = dates[0]
                        logger.debug(f"Date extraction: {extracted}")
                except Exception:
                    pass

            # Fallback: simple year/date extraction
            if not extracted:
                date_pattern = r"\b(1[0-9]{3}|20[0-2][0-9])\b"
                years = re.findall(date_pattern, answer)
                if years:
                    extracted = years[0]

        # WHERE questions: Extract locations using spaCy NER
        elif qtype == "where" and self.nlp_processor:
            try:
                doc = self.nlp_processor.nlp(answer)
                locations = [
                    ent.text for ent in doc.ents if ent.label_ in ["GPE", "LOC"]
                ]
                if locations:
                    extracted = locations[0]
                    logger.debug(f"Location extraction: {extracted}")
            except Exception:
                pass

        # WHAT questions: Careful! Don't lose subject names
        # "The Pacific Ocean is..." should NOT become just "the largest ocean"
        elif qtype == "what":
            # Check if answer starts with a proper noun/subject
            # If yes, keep it. If no, extract definition.
            if self.nlp_processor:
                try:
                    doc = self.nlp_processor.nlp(answer)
                    # If first token is proper noun, it's likely the subject we want
                    if doc and doc[0].pos_ == "PROPN":
                        logger.debug("Proper noun detected - returning full content")
                        return answer
                except Exception:
                    pass

            # Otherwise extract definition after is/are
            is_pattern = r"^(.+?)\s+(?:is|are)\s+(.+?)(?:\.|$)"
            match = re.search(is_pattern, answer, re.IGNORECASE)
            if match:
                subject = match.group(1).strip()
                definition = match.group(2).strip()

                # If subject looks important (capitalized, multiple words), include it
                if subject[0].isupper() or len(subject.split()) > 1:
                    extracted = answer  # Keep full answer
                else:
                    # Clean up articles from definition
                    definition = re.sub(
                        r"^(?:a|an|the)\s+", "", definition, flags=re.IGNORECASE
                    )
                    extracted = definition

        # STRATEGY 3: Validate extraction quality
        # If extraction seems incomplete/too generic, return full answer
        if extracted:
            words = extracted.split()
            word_count = len(words)

            # Too short? Probably lost context
            if word_count < 2:
                logger.debug(
                    f"Extraction too short ({word_count} words) - using full content"
                )
                return answer

            # Too generic? (just articles/pronouns)
            if extracted.lower() in ["the", "a", "an", "it", "this", "that"]:
                logger.debug("Extraction too generic - using full content")
                return answer

            # Extraction looks good!
            logger.debug(f"Using extracted answer: '{extracted}'")
            return extracted

        # No extraction worked - return full answer (safe default)
        logger.debug("No extraction - returning full content")
        return answer

    def _assess_query_complexity(
        self, query: str, query_intent: Dict[str, Any]
    ) -> float:
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
