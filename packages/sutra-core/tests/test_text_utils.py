"""
Comprehensive tests for text utility functions.
"""

import pytest
from sutra_core.utils import (
    calculate_word_overlap,
    clean_text,
    extract_words,
    get_association_patterns,
)


class TestExtractWords:
    """Test word extraction functionality."""

    def test_basic_word_extraction(self):
        """Test basic word extraction."""
        words = extract_words("The quick brown fox")
        assert "quick" in words
        assert "brown" in words
        assert "fox" in words
        assert "the" not in words  # Stop word filtered

    def test_empty_string(self):
        """Test with empty string."""
        words = extract_words("")
        assert words == []

    def test_special_characters(self):
        """Test with special characters."""
        words = extract_words("Hello, world! How are you?")
        assert "hello" in words
        assert "world" in words
        assert "," not in words
        assert "!" not in words

    def test_numbers_and_mixed_content(self):
        """Test with numbers and mixed content."""
        words = extract_words("Python3 is amazing for AI100")
        assert "python3" in words
        assert "amazing" in words
        assert "ai100" in words

    def test_unicode_characters(self):
        """Test with unicode characters."""
        words = extract_words("café résumé naïve")
        # Should handle unicode properly
        assert len(words) > 0

    def test_very_long_text(self):
        """Test with very long text."""
        long_text = "word " * 1000
        words = extract_words(long_text)
        assert "word" in words
        assert len(words) > 0


class TestGetAssociationPatterns:
    """Test association pattern retrieval."""

    def test_returns_patterns(self):
        """Test that patterns are returned."""
        patterns = get_association_patterns()
        assert len(patterns) > 0
        assert all(isinstance(p, tuple) for p in patterns)

    def test_pattern_structure(self):
        """Test pattern structure."""
        patterns = get_association_patterns()
        for pattern, assoc_type in patterns:
            assert isinstance(pattern, str)
            assert hasattr(assoc_type, "value")

    def test_all_association_types_present(self):
        """Test that all expected association types are present."""
        from sutra_core import AssociationType

        patterns = get_association_patterns()
        types_present = {assoc_type for _, assoc_type in patterns}

        expected_types = {
            AssociationType.CAUSAL,
            AssociationType.HIERARCHICAL,
            AssociationType.COMPOSITIONAL,
            AssociationType.SEMANTIC,
            AssociationType.TEMPORAL,
        }

        assert expected_types.issubset(types_present)


class TestCleanText:
    """Test text cleaning functionality."""

    def test_basic_cleaning(self):
        """Test basic text cleaning."""
        text = "  Hello   world  "
        cleaned = clean_text(text)
        assert cleaned == "Hello world"

    def test_special_character_removal(self):
        """Test special character handling."""
        text = "Hello @world #test"
        cleaned = clean_text(text)
        assert "@" not in cleaned
        assert "#" not in cleaned

    def test_preserve_sentence_structure(self):
        """Test that sentence structure is preserved."""
        text = "Hello, world. How are you?"
        cleaned = clean_text(text)
        assert "," in cleaned
        assert "." in cleaned
        assert "?" in cleaned

    def test_empty_string(self):
        """Test with empty string."""
        cleaned = clean_text("")
        assert cleaned == ""

    def test_only_whitespace(self):
        """Test with only whitespace."""
        cleaned = clean_text("   \t\n  ")
        assert cleaned == ""

    def test_multiple_spaces_collapsed(self):
        """Test that multiple spaces are collapsed."""
        text = "Hello    world     test"
        cleaned = clean_text(text)
        assert "  " not in cleaned


class TestCalculateWordOverlap:
    """Test word overlap calculation."""

    def test_identical_lists(self):
        """Test with identical word lists."""
        words1 = ["hello", "world", "test"]
        words2 = ["hello", "world", "test"]
        overlap = calculate_word_overlap(words1, words2)
        assert overlap == 1.0

    def test_no_overlap(self):
        """Test with no overlapping words."""
        words1 = ["hello", "world"]
        words2 = ["foo", "bar"]
        overlap = calculate_word_overlap(words1, words2)
        assert overlap == 0.0

    def test_partial_overlap(self):
        """Test with partial overlap."""
        words1 = ["hello", "world", "test"]
        words2 = ["hello", "foo", "bar"]
        overlap = calculate_word_overlap(words1, words2)
        assert 0.0 < overlap < 1.0
        assert overlap == pytest.approx(1.0 / 3.0, abs=0.01)

    def test_empty_first_list(self):
        """Test with empty first list."""
        words1 = []
        words2 = ["hello", "world"]
        overlap = calculate_word_overlap(words1, words2)
        assert overlap == 0.0

    def test_empty_second_list(self):
        """Test with empty second list."""
        words1 = ["hello", "world"]
        words2 = []
        overlap = calculate_word_overlap(words1, words2)
        assert overlap == 0.0

    def test_both_empty_lists(self):
        """Test with both lists empty."""
        overlap = calculate_word_overlap([], [])
        assert overlap == 0.0

    def test_different_sizes(self):
        """Test with different sized lists."""
        words1 = ["hello", "world"]
        words2 = ["hello", "world", "foo", "bar", "baz"]
        overlap = calculate_word_overlap(words1, words2)
        assert overlap == 1.0  # All words in smaller list are in larger

    def test_duplicates_handled(self):
        """Test that duplicates are handled correctly."""
        words1 = ["hello", "hello", "world"]
        words2 = ["hello", "foo"]
        overlap = calculate_word_overlap(words1, words2)
        # Should convert to sets internally
        assert overlap > 0.0


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_extract_words_with_none_input(self):
        """Test extract_words with None input."""
        # This should raise an error or handle gracefully
        with pytest.raises((TypeError, AttributeError)):
            extract_words(None)

    def test_clean_text_with_none_input(self):
        """Test clean_text with None input."""
        with pytest.raises((TypeError, AttributeError)):
            clean_text(None)

    def test_very_short_words(self):
        """Test filtering of very short words."""
        words = extract_words("I a am is be at on in to")
        # All should be filtered (either stop words or too short)
        assert len(words) == 0

    def test_mixed_case_handling(self):
        """Test that case is handled consistently."""
        words1 = extract_words("Hello World")
        words2 = extract_words("hello world")
        # Should be lowercase
        assert words1 == words2
        assert all(w.islower() for w in words1)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
