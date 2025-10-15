"""
Quick smoke test for Phase 1 & 2 improvements.

Tests:
- Input validation
- Type-safe operations
- NLP processing with spaCy
"""

import sys
sys.path.insert(0, '/Users/nisheethranjan/Projects/sutra-models/packages/sutra-core')

from sutra_core import Validator
from sutra_core.utils.nlp import TextProcessor
from sutra_core.exceptions import ValidationError

def test_validation():
    """Test input validation."""
    print("Testing input validation...")
    
    # Valid inputs
    assert Validator.validate_content("Hello world") == "Hello world"
    assert Validator.validate_query("What is AI?") == "What is AI?"
    assert Validator.validate_confidence(0.5) == 0.5
    assert Validator.validate_confidence(1.5) == 1.0  # Clamped
    assert Validator.validate_depth(5) == 5
    
    # Invalid inputs
    try:
        Validator.validate_content("")
        assert False, "Should have raised ValidationError"
    except ValidationError:
        pass
    
    try:
        Validator.validate_query("x" * 2000)  # Too long
        assert False, "Should have raised ValidationError"
    except ValidationError:
        pass
    
    print("✅ Validation tests passed")

def test_nlp():
    """Test NLP processing."""
    print("\nTesting NLP processing...")
    
    processor = TextProcessor()
    
    # Test tokenization
    tokens = processor.extract_meaningful_tokens("The cats are running quickly")
    print(f"Tokens: {tokens}")
    assert "cat" in tokens or "cats" in tokens  # Should lemmatize
    assert "run" in tokens or "running" in tokens
    assert "the" not in tokens  # Stop word removed
    
    # Test entity extraction
    entities = processor.extract_entities("Apple Inc. is based in Cupertino, California")
    print(f"Entities: {entities}")
    assert len(entities) > 0
    
    # Test negation detection
    assert processor.detect_negation("The sun is not a planet") == True
    assert processor.detect_negation("The sun is a star") == False
    
    # Test subject-verb-object extraction
    triples = processor.extract_subject_verb_object("Cats chase mice")
    print(f"Triples: {triples}")
    assert len(triples) > 0
    
    # Test causal relations
    causals = processor.extract_causal_relations("Rain causes flooding")
    print(f"Causals: {causals}")
    
    print("✅ NLP tests passed")

def test_backward_compatibility():
    """Test backward compatibility."""
    print("\nTesting backward compatibility...")
    
    from sutra_core.utils.text import extract_words, clean_text
    
    # Should still work with fallback or spaCy
    words = extract_words("Hello world")
    assert len(words) > 0
    
    text = clean_text("  Hello   world  ")
    assert text == "Hello world"
    
    print("✅ Backward compatibility maintained")

if __name__ == "__main__":
    print("=" * 60)
    print("SUTRA AI - PHASE 1 & 2 SMOKE TESTS")
    print("=" * 60)
    
    test_validation()
    test_nlp()
    test_backward_compatibility()
    
    print("\n" + "=" * 60)
    print("✅ ALL TESTS PASSED - Ready for Phase 3")
    print("=" * 60)
