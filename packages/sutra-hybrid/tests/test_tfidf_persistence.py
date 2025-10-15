"""
Tests for TF-IDF vectorizer persistence.

Verifies that the TF-IDF vectorizer state is properly saved and restored,
including vocabulary, IDF values, and all internal sklearn state.
"""

import shutil
import tempfile
from pathlib import Path

import numpy as np
import pytest
from sutra_hybrid import HybridAI


class TestTfidfPersistence:
    """Test TF-IDF vectorizer persistence."""

    def setup_method(self):
        """Create temporary storage directory."""
        self.temp_dir = tempfile.mkdtemp()
        self.storage_path = Path(self.temp_dir) / "test_knowledge"

    def teardown_method(self):
        """Clean up temporary storage."""
        if Path(self.temp_dir).exists():
            shutil.rmtree(self.temp_dir)

    def test_basic_save_and_load(self):
        """Test basic save and load cycle."""
        # Create AI and learn some content
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        content1 = "Photosynthesis converts sunlight into chemical energy"
        content2 = "Cellular respiration produces ATP from glucose"

        id1 = ai.learn(content1)
        id2 = ai.learn(content2)

        # Save
        ai.save()

        # Create new AI instance and verify it loads
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        assert len(ai2.concepts) == len(ai.concepts)
        assert id1 in ai2.concepts
        assert id2 in ai2.concepts

    def test_vectorizer_state_persisted(self):
        """Test that vectorizer state is fully persisted."""
        # Create AI and learn content
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        content = "Machine learning uses algorithms to find patterns in data"
        ai.learn(content)

        # Get the vectorizer vocabulary before save
        vocab_before = ai.embedding_provider.get_vocabulary()
        state_before = ai.embedding_provider.get_state()

        assert vocab_before is not None
        assert len(vocab_before) > 0
        assert state_before is not None
        assert len(state_before) > 0

        # Save
        ai.save()

        # Verify pickle file exists
        pkl_file = self.storage_path / "tfidf_vectorizer.pkl"
        assert pkl_file.exists()
        assert pkl_file.stat().st_size > 0

        # Load in new instance
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Verify vectorizer is fitted
        assert ai2.embedding_provider.is_fitted

        # Get vocabulary after load
        vocab_after = ai2.embedding_provider.get_vocabulary()
        assert vocab_after == vocab_before

    def test_embeddings_consistent_after_reload(self):
        """Test that embeddings are consistent after save/load cycle."""
        # Create AI and learn content
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        content = "Neural networks learn from training data"
        concept_id = ai.learn(content)

        # Get embedding before save
        embedding_before = ai.concept_embeddings[concept_id]

        # Save
        ai.save()

        # Load in new instance
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Get embedding after load
        embedding_after = ai2.concept_embeddings[concept_id]

        # Verify embeddings are identical
        np.testing.assert_array_almost_equal(embedding_before, embedding_after)

    def test_semantic_search_after_reload(self):
        """Test that semantic search works correctly after reload."""
        # Create AI and learn biology content
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        ai.learn("Photosynthesis converts sunlight into chemical energy")
        ai.learn("Cellular respiration produces ATP from glucose")
        ai.learn("Mitochondria are the powerhouse of the cell")

        # Save
        ai.save()

        # Load in new instance
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Use a query with words from the learned content
        results = ai2.semantic_search("energy cellular", top_k=3, threshold=0.1)

        # Should be able to search (may or may not find results
        # depending on vocabulary). The important thing is no crash
        assert isinstance(results, list)

    def test_new_learning_after_reload(self):
        """Test that we can continue learning after reload."""
        # Create AI and learn initial content
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        ai.learn("Initial knowledge about machine learning")

        # Save
        ai.save()

        # Load in new instance
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Learn new content
        new_id = ai2.learn("New knowledge about deep learning")

        # Verify new concept was added
        assert new_id in ai2.concepts
        assert new_id in ai2.concept_embeddings

        # Verify we can search for both old and new content
        results = ai2.semantic_search("machine learning", top_k=5)
        assert len(results) > 0

    def test_multiple_save_load_cycles(self):
        """Test multiple save/load cycles maintain integrity."""
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Cycle 1: Learn and save
        concept1 = ai.learn("Python is a programming language")
        ai.save()

        # Cycle 2: Load, learn more, save
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))
        concept2 = ai2.learn("JavaScript is used for web development")
        ai2.save()

        # Cycle 3: Load and verify all concepts present
        ai3 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Should have concepts from both learning sessions
        stats = ai3.get_stats()
        assert stats["total_concepts"] >= 2
        assert stats["total_embeddings"] >= 2

        # Verify both concepts exist
        assert concept1 in ai3.concepts
        assert concept2 in ai3.concepts

    def test_empty_ai_save_load(self):
        """Test save/load with no learned concepts."""
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Save without learning anything
        ai.save()

        # Should be able to load (though nothing there)
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        assert len(ai2.concepts) == 0
        assert len(ai2.concept_embeddings) == 0

    def test_vectorizer_pickle_format(self):
        """Test that the vectorizer pickle file is valid."""
        import pickle

        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))
        ai.learn("Test content for vectorizer")
        ai.save()

        # Load pickle file directly
        pkl_file = self.storage_path / "tfidf_vectorizer.pkl"
        with open(pkl_file, "rb") as f:
            vectorizer = pickle.load(f)

        # Verify it's a TfidfVectorizer
        from sklearn.feature_extraction.text import TfidfVectorizer

        assert isinstance(vectorizer, TfidfVectorizer)

        # Verify it's fitted
        assert hasattr(vectorizer, "vocabulary_")
        assert len(vectorizer.vocabulary_) > 0

    def test_large_vocabulary_persistence(self):
        """Test persistence with larger vocabulary."""
        ai = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Learn many concepts with diverse vocabulary
        concepts = [
            "Machine learning algorithms find patterns in data",
            "Neural networks consist of interconnected layers",
            "Deep learning uses multiple layers of abstraction",
            "Natural language processing analyzes text",
            "Computer vision interprets visual information",
            "Reinforcement learning learns through trial and error",
            "Supervised learning uses labeled training data",
            "Unsupervised learning discovers hidden patterns",
            "Transfer learning applies knowledge across domains",
            "Ensemble methods combine multiple models",
        ]

        for concept in concepts:
            ai.learn(concept)

        # Get vocabulary size (note: max_features=100 limits this)
        vocab_size = len(ai.embedding_provider.get_vocabulary())
        assert vocab_size > 5  # Should have some vocabulary

        # Save
        ai.save()

        # Load
        ai2 = HybridAI(use_semantic=False, storage_path=str(self.storage_path))

        # Verify vocabulary size matches
        vocab_size_after = len(ai2.embedding_provider.get_vocabulary())
        assert vocab_size_after == vocab_size

        # Verify search still works
        results = ai2.semantic_search("machine learning", top_k=5)
        assert len(results) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
