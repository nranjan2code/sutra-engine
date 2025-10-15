"""
Basic tests for sutra-core functionality.
"""

from collections import defaultdict

import pytest
from sutra_core import (
    Association,
    AssociationType,
    Concept,
    extract_words,
    get_association_patterns,
)
from sutra_core.learning import AdaptiveLearner, AssociationExtractor


class TestConcept:
    """Test Concept class functionality."""

    def test_concept_creation(self):
        """Test basic concept creation."""
        concept = Concept(id="test_id", content="test content", source="test_source")

        assert concept.id == "test_id"
        assert concept.content == "test content"
        assert concept.source == "test_source"
        assert concept.strength == 1.0
        assert concept.access_count == 0

    def test_concept_access(self):
        """Test concept access strengthening."""
        concept = Concept(id="test", content="test")
        initial_strength = concept.strength

        concept.access()

        assert concept.access_count == 1
        assert concept.strength > initial_strength
        assert concept.strength == pytest.approx(1.02, abs=0.001)

    def test_concept_serialization(self):
        """Test concept to_dict and from_dict."""
        original = Concept(
            id="test", content="test content", source="test_source", strength=5.0
        )

        # Serialize and deserialize
        data = original.to_dict()
        restored = Concept.from_dict(data)

        assert restored.id == original.id
        assert restored.content == original.content
        assert restored.source == original.source
        assert restored.strength == original.strength


class TestAssociation:
    """Test Association class functionality."""

    def test_association_creation(self):
        """Test basic association creation."""
        assoc = Association(
            source_id="source",
            target_id="target",
            assoc_type=AssociationType.SEMANTIC,
            confidence=0.8,
        )

        assert assoc.source_id == "source"
        assert assoc.target_id == "target"
        assert assoc.assoc_type == AssociationType.SEMANTIC
        assert assoc.confidence == 0.8
        assert assoc.weight == 1.0

    def test_association_strengthen(self):
        """Test association strengthening."""
        assoc = Association(
            source_id="a", target_id="b", assoc_type=AssociationType.CAUSAL
        )
        initial_weight = assoc.weight

        assoc.strengthen()

        assert assoc.weight > initial_weight
        assert assoc.weight == pytest.approx(1.1, abs=0.001)


class TestTextUtils:
    """Test text processing utilities."""

    def test_extract_words(self):
        """Test word extraction."""
        words = extract_words("The quick brown fox jumps over lazy dogs")

        # Should filter out stop words and short words
        assert "the" not in words
        assert "quick" in words
        assert "brown" in words
        assert "jumps" in words

    def test_get_association_patterns(self):
        """Test association pattern retrieval."""
        patterns = get_association_patterns()

        assert len(patterns) > 0
        assert any(assoc_type == AssociationType.CAUSAL for _, assoc_type in patterns)
        assert any(assoc_type == AssociationType.SEMANTIC for _, assoc_type in patterns)


class TestAdaptiveLearner:
    """Test adaptive learning functionality."""

    def setup_method(self):
        """Set up test fixtures."""
        self.concepts = {}
        self.associations = {}
        self.word_to_concepts = defaultdict(set)
        self.concept_neighbors = defaultdict(set)

        self.extractor = AssociationExtractor(
            self.concepts,
            self.word_to_concepts,
            self.concept_neighbors,
            self.associations,
        )

        self.learner = AdaptiveLearner(self.concepts, self.extractor)

    def test_basic_learning(self):
        """Test basic learning functionality."""
        concept_id = self.learner.learn_adaptive(
            "Test content for learning", source="test"
        )

        assert concept_id in self.concepts
        concept = self.concepts[concept_id]
        assert concept.content == "Test content for learning"
        assert concept.source == "test"

    def test_adaptive_reinforcement(self):
        """Test adaptive reinforcement logic."""
        # First learn a concept to establish it in the system
        concept_id = self.learner.learn_adaptive("test concept with low strength")
        concept = self.concepts[concept_id]

        # Manually set it to difficult strength
        concept.strength = 3.0
        initial_strength = concept.strength

        # Learn the same content again - should get strong reinforcement
        self.learner.learn_adaptive("test concept with low strength")

        # Strength should increase more than standard 1.02× because it's difficult
        # Expected: 3.0 * 1.02 (access) * 1.15 (adaptive) = 3.519
        expected_min = (
            initial_strength * 1.02 * 1.15
        )  # Both access() and adaptive reinforcement
        assert (
            concept.strength >= expected_min
        ), f"Expected >= {expected_min}, got {concept.strength}"
        # Also verify it's more than just the standard access reinforcement
        standard_reinforcement = initial_strength * 1.02
        assert (
            concept.strength > standard_reinforcement
        ), f"Should exceed standard reinforcement of {standard_reinforcement}"

    def test_learning_stats(self):
        """Test learning statistics."""
        # Add some concepts with different strengths
        self.concepts["easy"] = Concept(id="easy", content="easy", strength=8.0)
        self.concepts["hard"] = Concept(id="hard", content="hard", strength=2.0)
        self.concepts["medium"] = Concept(id="medium", content="medium", strength=5.0)

        stats = self.learner.get_learning_stats()

        assert stats["total_concepts"] == 3
        assert stats["difficult_concepts"] == 1  # strength < 4.0
        assert stats["easy_concepts"] == 1  # strength > 7.0
        assert stats["moderate_concepts"] == 1  # between 4.0 and 7.0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
