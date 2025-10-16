"""
Explanation generator for Sutra AI reasoning.

Converts reasoning paths and results into human-readable explanations.
"""

from typing import List, Optional

from .results import ConfidenceBreakdown, ReasoningPathDetail


class ExplanationGenerator:
    """
    Generate human-readable explanations for reasoning results.

    Provides transparency by explaining:
    - How the answer was found
    - Why paths were chosen
    - How confidence was calculated
    - Alternative paths considered
    """

    def generate(
        self,
        query: str,
        answer: str,
        confidence: float,
        reasoning_paths: List[ReasoningPathDetail],
        semantic_boost: bool = False,
        semantic_contribution: float = 0.0,
    ) -> str:
        """
        Generate comprehensive explanation.

        Args:
            query: Original query
            answer: Answer provided
            confidence: Final confidence score
            reasoning_paths: List of reasoning paths used
            semantic_boost: Whether semantic boosting was used
            semantic_contribution: How much semantic similarity contributed

        Returns:
            Human-readable explanation string
        """
        lines = []

        # Header
        lines.append("=" * 70)
        lines.append("REASONING EXPLANATION")
        lines.append("=" * 70)

        # Query and answer
        lines.append(f'\nQuery: "{query}"')
        lines.append(f"Answer: {answer}")
        lines.append(
            f"Confidence: {confidence:.2f} ({self._confidence_label(confidence)})"
        )

        # Reasoning method
        lines.append("\n" + "-" * 70)
        lines.append("REASONING METHOD")
        lines.append("-" * 70)

        if semantic_boost:
            lines.append("Method: Graph traversal with semantic enhancement")
            lines.append(f"  • Graph reasoning: {(1-semantic_contribution)*100:.0f}%")
            lines.append(f"  • Semantic boost: {semantic_contribution*100:.0f}%")
        else:
            lines.append("Method: Pure graph traversal (100% explainable)")

        # Primary reasoning path
        if reasoning_paths:
            lines.append("\n" + "-" * 70)
            lines.append("PRIMARY REASONING PATH")
            lines.append("-" * 70)

            primary_path = reasoning_paths[0]
            lines.append(f"Confidence: {primary_path.confidence:.2f}")
            lines.append(f"\nPath: {self._format_path(primary_path)}")
            lines.append(f"\nExplanation: {primary_path.explanation}")

            # Show concepts in path
            if len(primary_path.concepts) > 0:
                lines.append("\nConcepts in path:")
                for i, concept in enumerate(
                    primary_path.concepts[:5], 1
                ):  # Show first 5
                    lines.append(f"  {i}. {concept[:60]}...")

            # Show association types
            if len(primary_path.association_types) > 0:
                assoc_str = ", ".join(primary_path.association_types[:5])
                lines.append(f"\nAssociation types: {assoc_str}")

        # Alternative paths
        if len(reasoning_paths) > 1:
            lines.append("\n" + "-" * 70)
            lines.append("ALTERNATIVE PATHS CONSIDERED")
            lines.append("-" * 70)

            for i, path in enumerate(
                reasoning_paths[1:4], 2
            ):  # Show up to 3 alternatives
                lines.append(f"\nPath {i} (confidence: {path.confidence:.2f}):")
                lines.append(f"  {self._format_path(path, short=True)}")

        # How confidence was calculated
        lines.append("\n" + "-" * 70)
        lines.append("CONFIDENCE CALCULATION")
        lines.append("-" * 70)
        lines.append(
            self._explain_confidence(confidence, reasoning_paths, semantic_boost)
        )

        # Footer
        lines.append("\n" + "=" * 70)

        return "\n".join(lines)

    def generate_short(
        self,
        query: str,
        answer: str,
        confidence: float,
        primary_path: Optional[ReasoningPathDetail] = None,
    ) -> str:
        """
        Generate short explanation (1-2 paragraphs).

        For use in API responses where full explanation is too verbose.
        """
        lines = []

        lines.append(f'Query: "{query}"')
        lines.append(f"Answer: {answer} (confidence: {confidence:.2f})")

        if primary_path:
            lines.append(
                f"\nFound via {len(primary_path.concepts)}-concept path using "
                f"{', '.join(set(primary_path.association_types))} associations."
            )
            lines.append(primary_path.explanation)

        return "\n".join(lines)

    def _format_path(self, path: ReasoningPathDetail, short: bool = False) -> str:
        """Format a reasoning path as string."""
        if short:
            return f"{path.concepts[0][:30]}... → ... → {path.concepts[-1][:30]}..."
        else:
            if len(path.concepts) <= 4:
                return " → ".join([c[:40] for c in path.concepts])
            else:
                return f"{path.concepts[0][:30]}... → ({len(path.concepts)-2} concepts) → ...{path.concepts[-1][:30]}"

    def _confidence_label(self, confidence: float) -> str:
        """Convert confidence score to label."""
        if confidence >= 0.9:
            return "Very High"
        elif confidence >= 0.75:
            return "High"
        elif confidence >= 0.6:
            return "Medium"
        elif confidence >= 0.4:
            return "Low"
        else:
            return "Very Low"

    def _explain_confidence(
        self, confidence: float, paths: List[ReasoningPathDetail], semantic_boost: bool
    ) -> str:
        """Explain how confidence was calculated."""
        lines = []

        if not paths:
            lines.append("No reasoning paths found - answer is speculative.")
            return "\n".join(lines)

        # Path quality
        avg_path_conf = sum(p.confidence for p in paths) / len(paths)
        lines.append(f"Average path confidence: {avg_path_conf:.2f}")

        # Number of paths
        lines.append(f"Number of reasoning paths: {len(paths)}")

        if len(paths) > 1:
            # Consensus
            confidence_std = self._std([p.confidence for p in paths])
            if confidence_std < 0.1:
                lines.append("Strong consensus: All paths agree (low variance)")
            elif confidence_std < 0.2:
                lines.append("Moderate consensus: Most paths agree")
            else:
                lines.append("Weak consensus: Paths disagree (high variance)")

        # Semantic boost
        if semantic_boost:
            lines.append("Semantic similarity used to enhance relevance")

        # Final calculation
        lines.append(f"\nFinal confidence: {confidence:.2f}")
        lines.append(f"Interpretation: {self._interpret_confidence(confidence)}")

        return "\n".join(lines)

    def _interpret_confidence(self, confidence: float) -> str:
        """Interpret what confidence score means."""
        if confidence >= 0.9:
            return "Very strong evidence for this answer"
        elif confidence >= 0.75:
            return "Strong evidence for this answer"
        elif confidence >= 0.6:
            return "Moderate evidence for this answer"
        elif confidence >= 0.4:
            return "Weak evidence - answer may be incomplete"
        else:
            return "Very weak evidence - highly uncertain"

    def _std(self, values: List[float]) -> float:
        """Calculate standard deviation."""
        if len(values) < 2:
            return 0.0
        mean = sum(values) / len(values)
        variance = sum((x - mean) ** 2 for x in values) / len(values)
        return variance**0.5

    def generate_multi_strategy_explanation(
        self,
        query: str,
        graph_answer: str,
        graph_conf: float,
        semantic_answer: str,
        semantic_conf: float,
        agreement: float,
    ) -> str:
        """
        Explain multi-strategy reasoning comparison.

        Shows how different reasoning approaches compare.
        """
        lines = []

        lines.append("=" * 70)
        lines.append("MULTI-STRATEGY REASONING COMPARISON")
        lines.append("=" * 70)

        lines.append(f'\nQuery: "{query}"')

        lines.append("\n" + "-" * 70)
        lines.append("STRATEGY 1: Pure Graph Reasoning")
        lines.append("-" * 70)
        lines.append(f"Answer: {graph_answer}")
        lines.append(f"Confidence: {graph_conf:.2f}")
        lines.append("Method: Graph traversal only (100% explainable)")

        lines.append("\n" + "-" * 70)
        lines.append("STRATEGY 2: Semantic-Enhanced Reasoning")
        lines.append("-" * 70)
        lines.append(f"Answer: {semantic_answer}")
        lines.append(f"Confidence: {semantic_conf:.2f}")
        lines.append("Method: Graph + semantic similarity")

        lines.append("\n" + "-" * 70)
        lines.append("AGREEMENT ANALYSIS")
        lines.append("-" * 70)
        lines.append(f"Agreement score: {agreement:.2f}")

        if agreement > 0.9:
            lines.append("Status: Strong agreement - both methods concur")
            lines.append("Recommendation: Use semantic-enhanced (higher confidence)")
        elif agreement > 0.7:
            lines.append("Status: Moderate agreement - methods mostly align")
            lines.append("Recommendation: Use semantic-enhanced with caution")
        else:
            lines.append("Status: Weak agreement - methods disagree")
            lines.append("Recommendation: Use graph-only for explainability")

        lines.append("\n" + "=" * 70)

        return "\n".join(lines)
