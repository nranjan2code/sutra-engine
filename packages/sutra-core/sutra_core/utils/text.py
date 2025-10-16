"""
Text processing utilities for the Sutra AI system.

This module contains functions for:
- Text tokenization and word extraction
- Stop word filtering
- Pattern matching for associations
"""

import re
from typing import List, Tuple

from ..graph.concepts import AssociationType

# Module-level constant for stop words (avoid recreation on every call)
STOP_WORDS = frozenset(
    {
        "the",
        "a",
        "an",
        "and",
        "or",
        "but",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "is",
        "was",
        "are",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "can",
        "must",
        "shall",
        "this",
        "that",
        "these",
        "those",
    }
)


def extract_words(text: str) -> List[str]:
    """
    Extract meaningful words from text.

    Filters out short words and common stop words to focus on
    semantically meaningful terms.

    Args:
        text: Input text to process

    Returns:
        List of filtered words
    """
    # Simple tokenization - could be enhanced with NLP
    words = re.findall(r"\b\w+\b", text.lower())

    # Filter out very short words and common stop words (using module constant)
    return [w for w in words if len(w) > 2 and w not in STOP_WORDS]


def get_association_patterns() -> List[Tuple[str, AssociationType]]:
    """
    Get regex patterns for association extraction.

    Returns patterns for different types of relationships:
    - Causal: X causes Y
    - Hierarchical: X is a Y
    - Compositional: X contains Y
    - Semantic: X similar to Y
    - Temporal: X before Y

    Returns:
        List of (pattern, AssociationType) tuples
    """
    from ..graph.concepts import AssociationType

    return [
        # Causal relationships
        (r"(.+?) causes (.+)", AssociationType.CAUSAL),
        (r"(.+?) leads to (.+)", AssociationType.CAUSAL),
        (r"(.+?) results in (.+)", AssociationType.CAUSAL),
        # Hierarchical relationships
        (r"(.+?) is (?:a|an) (.+)", AssociationType.HIERARCHICAL),
        (r"(.+?) are (?:a|an|types? of) (.+)", AssociationType.HIERARCHICAL),
        # Compositional relationships
        (r"(.+?) contains (.+)", AssociationType.COMPOSITIONAL),
        (r"(.+?) includes (.+)", AssociationType.COMPOSITIONAL),
        (r"(.+?) consists of (.+)", AssociationType.COMPOSITIONAL),
        # Semantic relationships (definitions, descriptions)
        (r"(.+?) refers to (.+)", AssociationType.SEMANTIC),
        (r"(.+?) means (.+)", AssociationType.SEMANTIC),
        (r"(.+?) involves (.+)", AssociationType.SEMANTIC),
        (r"(.+?) similar to (.+)", AssociationType.SEMANTIC),
        (r"(.+?) related to (.+)", AssociationType.SEMANTIC),
        # Temporal relationships
        (r"(.+?) before (.+)", AssociationType.TEMPORAL),
        (r"(.+?) after (.+)", AssociationType.TEMPORAL),
        (r"(.+?) follows (.+)", AssociationType.TEMPORAL),
    ]


def clean_text(text: str) -> str:
    """
    Clean and normalize text for processing.

    Args:
        text: Raw input text

    Returns:
        Cleaned text
    """
    # Remove extra whitespace
    text = re.sub(r"\s+", " ", text.strip())

    # Remove special characters that might interfere with parsing
    text = re.sub(r"[^\w\s\-.,;:!?()]", "", text)

    return text


def calculate_word_overlap(words1: List[str], words2: List[str]) -> float:
    """
    Calculate word overlap ratio between two word lists.

    Args:
        words1: First word list
        words2: Second word list

    Returns:
        Overlap ratio (0.0 to 1.0)
    """
    if not words1 or not words2:
        return 0.0

    set1 = set(words1)
    set2 = set(words2)

    overlap = len(set1 & set2)
    total = min(len(set1), len(set2))

    return overlap / total if total > 0 else 0.0
