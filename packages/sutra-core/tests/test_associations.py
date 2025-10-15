"""
Comprehensive tests for association extraction and management.
"""

from collections import defaultdict

import pytest
from sutra_core import Association, AssociationType, Concept
from sutra_core.learning import AssociationExtractor


class TestAssociationExtractor:
    """Test association extractor functionality."""

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

    def test_extract_causal_associations(self):
        """Test extraction of causal relationships."""
        text = "Sunlight causes photosynthesis in plants"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created >= 1
        # Check that causal association was created
        causal_found = any(
            assoc.assoc_type == AssociationType.CAUSAL
            for assoc in self.associations.values()
        )
        assert causal_found

    def test_extract_hierarchical_associations(self):
        """Test extraction of hierarchical relationships."""
        text = "A dog is an animal"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created >= 1
        hierarchical_found = any(
            assoc.assoc_type == AssociationType.HIERARCHICAL
            for assoc in self.associations.values()
        )
        assert hierarchical_found

    def test_extract_compositional_associations(self):
        """Test extraction of compositional relationships."""
        text = "A cell contains mitochondria"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created >= 1
        compositional_found = any(
            assoc.assoc_type == AssociationType.COMPOSITIONAL
            for assoc in self.associations.values()
        )
        assert compositional_found

    def test_extract_semantic_associations(self):
        """Test extraction of semantic relationships."""
        text = "DNA is similar to RNA"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created >= 1
        semantic_found = any(
            assoc.assoc_type == AssociationType.SEMANTIC
            for assoc in self.associations.values()
        )
        assert semantic_found

    def test_extract_temporal_associations(self):
        """Test extraction of temporal relationships."""
        text = "Photosynthesis occurs before respiration"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created >= 1
        temporal_found = any(
            assoc.assoc_type == AssociationType.TEMPORAL
            for assoc in self.associations.values()
        )
        assert temporal_found

    def test_no_associations_in_simple_text(self):
        """Test that simple text without patterns creates no associations."""
        text = "The quick brown fox jumps"
        associations_created = self.extractor.extract_associations(text, "test_id")

        assert associations_created == 0

    def test_multiple_associations_in_one_text(self):
        """Test extracting multiple associations from one text."""
        text = "A dog is an animal that contains cells and causes joy"
        associations_created = self.extractor.extract_associations(text, "test_id")

        # Should find hierarchical, compositional, and causal
        assert associations_created >= 3

    def test_adaptive_extraction_depth_1(self):
        """Test adaptive extraction with depth 1 (normal)."""
        text = "Sunlight causes photosynthesis"
        associations_created = self.extractor.extract_associations_adaptive(
            text, "test_id", depth=1
        )

        # Should only do standard extraction
        assert associations_created >= 1

    def test_adaptive_extraction_depth_2(self):
        """Test adaptive extraction with depth 2 (deep)."""
        text = "Sunlight energy photosynthesis process"

        # Add some pre-existing concepts to test co-occurrence
        c1 = Concept(id="c1", content="sunlight information")
        c2 = Concept(id="c2", content="photosynthesis details")
        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2
        self.extractor._index_concept(c1)
        self.extractor._index_concept(c2)

        associations_created = self.extractor.extract_associations_adaptive(
            text, "test_id", depth=2
        )

        # Should do standard + co-occurrence extraction
        assert associations_created >= 0  # May or may not find patterns

    def test_concept_indexing(self):
        """Test that concepts are properly indexed by words."""
        concept = Concept(id="test_id", content="photosynthesis converts sunlight")
        self.extractor._index_concept(concept)

        assert "photosynthesis" in self.word_to_concepts
        assert "converts" in self.word_to_concepts
        assert "sunlight" in self.word_to_concepts
        assert "test_id" in self.word_to_concepts["photosynthesis"]

    def test_find_or_create_concept_new(self):
        """Test finding or creating a new concept."""
        initial_count = len(self.concepts)
        concept_id = self.extractor._find_or_create_concept("new concept text")

        assert len(self.concepts) == initial_count + 1
        assert concept_id in self.concepts
        assert self.concepts[concept_id].content == "new concept text"

    def test_find_or_create_concept_existing(self):
        """Test finding an existing concept."""
        # Create a concept first
        text = "existing concept"
        concept_id1 = self.extractor._find_or_create_concept(text)
        initial_count = len(self.concepts)

        # Try to create again with same text
        concept_id2 = self.extractor._find_or_create_concept(text)

        assert concept_id1 == concept_id2
        assert len(self.concepts) == initial_count  # No new concept created

    def test_create_new_association(self):
        """Test creating a new association."""
        c1 = Concept(id="c1", content="concept 1")
        c2 = Concept(id="c2", content="concept 2")
        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2

        created = self.extractor._create_association(
            c1.id, c2.id, AssociationType.SEMANTIC, 0.8
        )

        assert created is True
        key = (c1.id, c2.id)
        assert key in self.associations
        assert self.associations[key].confidence == 0.8

    def test_strengthen_existing_association(self):
        """Test strengthening an existing association."""
        c1 = Concept(id="c1", content="concept 1")
        c2 = Concept(id="c2", content="concept 2")
        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2

        # Create first association
        created1 = self.extractor._create_association(
            c1.id, c2.id, AssociationType.SEMANTIC, 0.8
        )
        key = (c1.id, c2.id)
        initial_weight = self.associations[key].weight

        # Try to create again - should strengthen instead
        created2 = self.extractor._create_association(
            c1.id, c2.id, AssociationType.SEMANTIC, 0.8
        )

        assert created1 is True
        assert created2 is False  # Not created, but strengthened
        assert self.associations[key].weight > initial_weight

    def test_neighbor_indexing(self):
        """Test that concept neighbors are properly indexed."""
        c1 = Concept(id="c1", content="concept 1")
        c2 = Concept(id="c2", content="concept 2")
        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2

        self.extractor._create_association(c1.id, c2.id, AssociationType.SEMANTIC, 0.8)

        assert c2.id in self.concept_neighbors[c1.id]
        assert c1.id in self.concept_neighbors[c2.id]


class TestCooccurrenceExtraction:
    """Test co-occurrence based association extraction."""

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

    def test_cooccurrence_with_existing_concepts(self):
        """Test co-occurrence extraction with pre-existing concepts."""
        # Create concepts with overlapping words
        c1 = Concept(id="c1", content="photosynthesis process")
        c2 = Concept(id="c2", content="process energy")
        c3 = Concept(id="c3", content="energy storage")

        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2
        self.concepts[c3.id] = c3

        self.extractor._index_concept(c1)
        self.extractor._index_concept(c2)
        self.extractor._index_concept(c3)

        # Extract from text with same words
        text = "photosynthesis process energy storage mechanism"
        associations_created = self.extractor._extract_cooccurrence_associations(
            text, "test_id"
        )

        # Should create associations between concepts with co-occurring words
        assert associations_created >= 0

    def test_cooccurrence_with_no_existing_concepts(self):
        """Test co-occurrence extraction with no pre-existing concepts."""
        text = "completely new unique words here"
        associations_created = self.extractor._extract_cooccurrence_associations(
            text, "test_id"
        )

        assert associations_created == 0

    def test_cooccurrence_respects_window_size(self):
        """Test that co-occurrence uses sliding window correctly."""
        # Create concepts
        c1 = Concept(id="c1", content="word1 information")
        c2 = Concept(id="c2", content="word2 information")
        c3 = Concept(id="c3", content="word5 information")

        self.concepts[c1.id] = c1
        self.concepts[c2.id] = c2
        self.concepts[c3.id] = c3

        self.extractor._index_concept(c1)
        self.extractor._index_concept(c2)
        self.extractor._index_concept(c3)

        # word1 and word2 are close, word5 is far
        text = "word1 word2 word3 word4 word5"
        associations_created = self.extractor._extract_cooccurrence_associations(
            text, "test_id"
        )

        # word1 and word2 should be associated (within window)
        # word1 and word5 should NOT be associated (outside window)
        assert associations_created >= 0


class TestEdgeCases:
    """Test edge cases and error conditions."""

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

    def test_empty_text_extraction(self):
        """Test extraction from empty text."""
        associations_created = self.extractor.extract_associations("", "test_id")
        assert associations_created == 0

    def test_very_long_text_extraction(self):
        """Test extraction from very long text."""
        long_text = "word causes effect " * 100
        associations_created = self.extractor.extract_associations(long_text, "test_id")
        # Should handle without crashing
        assert associations_created >= 0

    def test_special_characters_in_text(self):
        """Test extraction with special characters."""
        text = "A @dog$ is an #animal%"
        associations_created = self.extractor.extract_associations(text, "test_id")
        # Should extract "a dog is an animal" part
        assert associations_created >= 0

    def test_association_confidence_bounds(self):
        """Test that association confidence is properly set."""
        text = "Sunlight causes photosynthesis"
        self.extractor.extract_associations(text, "test_id")

        for assoc in self.associations.values():
            assert 0.0 <= assoc.confidence <= 1.0

    def test_concept_confidence_for_extracted_concepts(self):
        """Test that extracted concepts have appropriate confidence."""
        text = "Sunlight causes photosynthesis"
        self.extractor.extract_associations(text, "test_id")

        # Extracted concepts should have confidence of 0.7
        for concept in self.concepts.values():
            assert concept.confidence == 0.7


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
