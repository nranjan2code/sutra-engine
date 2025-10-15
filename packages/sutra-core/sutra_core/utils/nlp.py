"""
Next-generation NLP text processing for Sutra AI.

Replaces naive regex-based processing with proper NLP using spaCy:
- Lemmatization and morphological analysis
- Named entity recognition
- Dependency parsing for association extraction
- Negation detection
- Multi-language support (extensible)
"""

import logging
from typing import List, Optional, Set, Tuple

logger = logging.getLogger(__name__)


class TextProcessor:
    """
    Advanced text processing using spaCy NLP.
    
    Provides:
    - Lemmatization (running → run)
    - Entity extraction (COVID-19, self-esteem as single tokens)
    - POS tagging
    - Dependency parsing
    - Negation detection
    """
    
    def __init__(self, model: str = "en_core_web_sm", disable: Optional[List[str]] = None):
        """
        Initialize text processor with spaCy model.
        
        Args:
            model: spaCy model name (default: en_core_web_sm)
            disable: Pipeline components to disable for speed
        """
        try:
            import spacy
            
            # Load model with optional component disabling
            disable_components = disable or []
            self.nlp = spacy.load(model, disable=disable_components)
            self.model_name = model
            
            logger.info(f"Loaded spaCy model: {model}")
            
        except ImportError:
            raise ImportError(
                "spaCy is required for NLP text processing. "
                "Install with: pip install spacy && python -m spacy download en_core_web_sm"
            )
        except OSError:
            raise OSError(
                f"spaCy model '{model}' not found. "
                f"Download with: python -m spacy download {model}"
            )
    
    def extract_meaningful_tokens(
        self, 
        text: str,
        min_length: int = 2,
        include_entities: bool = True
    ) -> List[str]:
        """
        Extract meaningful tokens from text with lemmatization.
        
        Args:
            text: Input text
            min_length: Minimum token length
            include_entities: Include named entities as single tokens
            
        Returns:
            List of lemmatized tokens
        """
        if not text or not text.strip():
            return []
        
        doc = self.nlp(text)
        tokens: List[str] = []
        
        # Add named entities first (as multi-word units)
        if include_entities:
            for ent in doc.ents:
                if len(ent.text) >= min_length:
                    tokens.append(ent.text.lower())
        
        # Add lemmatized tokens (excluding stopwords, punct, spaces)
        for token in doc:
            if (
                not token.is_stop 
                and not token.is_punct 
                and not token.is_space
                and len(token.text) >= min_length
                and token.ent_type_ == ""  # Don't duplicate entities
            ):
                tokens.append(token.lemma_.lower())
        
        return tokens
    
    def extract_entities(self, text: str) -> List[Tuple[str, str]]:
        """
        Extract named entities from text.
        
        Args:
            text: Input text
            
        Returns:
            List of (entity_text, entity_label) tuples
        """
        if not text or not text.strip():
            return []
        
        doc = self.nlp(text)
        return [(ent.text, ent.label_) for ent in doc.ents]
    
    def extract_noun_chunks(self, text: str) -> List[str]:
        """
        Extract noun phrases/chunks from text.
        
        Args:
            text: Input text
            
        Returns:
            List of noun chunk strings
        """
        if not text or not text.strip():
            return []
        
        doc = self.nlp(text)
        return [chunk.text.lower() for chunk in doc.noun_chunks]
    
    def detect_negation(self, text: str) -> bool:
        """
        Detect if text contains negation.
        
        Args:
            text: Input text
            
        Returns:
            True if negation detected
        """
        if not text or not text.strip():
            return False
        
        doc = self.nlp(text)
        
        # Check for negation dependencies
        for token in doc:
            if token.dep_ == "neg":  # Negation modifier
                return True
        
        return False
    
    def extract_subject_verb_object(
        self, text: str
    ) -> List[Tuple[str, str, str, bool]]:
        """
        Extract (subject, verb, object, is_negated) triples.
        
        Uses dependency parsing to find semantic relationships.
        
        Args:
            text: Input text
            
        Returns:
            List of (subject, verb, object, is_negated) tuples
        """
        if not text or not text.strip():
            return []
        
        doc = self.nlp(text)
        triples: List[Tuple[str, str, str, bool]] = []
        
        # Find root verbs
        for token in doc:
            if token.pos_ == "VERB":
                subject = None
                obj = None
                is_negated = False
                
                # Find subject
                for child in token.children:
                    if child.dep_ in ["nsubj", "nsubjpass"]:
                        subject = child.text.lower()
                    elif child.dep_ in ["dobj", "pobj", "attr"]:
                        obj = child.text.lower()
                    elif child.dep_ == "neg":
                        is_negated = True
                
                if subject and obj:
                    verb = token.lemma_.lower()
                    triples.append((subject, verb, obj, is_negated))
        
        return triples
    
    def extract_causal_relations(
        self, text: str
    ) -> List[Tuple[str, str, bool]]:
        """
        Extract causal relationships (cause, effect, is_negated).
        
        Looks for causal verbs and markers.
        
        Args:
            text: Input text
            
        Returns:
            List of (cause, effect, is_negated) tuples
        """
        if not text or not text.strip():
            return []
        
        doc = self.nlp(text)
        causal_relations: List[Tuple[str, str, bool]] = []
        
        # Causal verbs and markers
        causal_markers = {
            "cause", "causes", "caused", 
            "lead", "leads", "led",
            "result", "results", "resulted",
            "produce", "produces", "produced",
            "trigger", "triggers", "triggered",
            "because", "since", "due to"
        }
        
        for token in doc:
            if token.lemma_.lower() in causal_markers:
                cause = None
                effect = None
                is_negated = False
                
                # Find cause (subject) and effect (object)
                for child in token.children:
                    if child.dep_ in ["nsubj", "nsubjpass"]:
                        cause = self._get_full_phrase(child)
                    elif child.dep_ in ["dobj", "pobj"]:
                        effect = self._get_full_phrase(child)
                    elif child.dep_ == "neg":
                        is_negated = True
                
                if cause and effect:
                    causal_relations.append((cause, effect, is_negated))
        
        return causal_relations
    
    def _get_full_phrase(self, token) -> str:
        """Get full noun phrase including modifiers."""
        # Get all children recursively
        phrase_tokens = [token]
        for child in token.children:
            if child.dep_ in ["det", "amod", "compound", "nummod", "prep", "pobj"]:
                phrase_tokens.append(child)
        
        # Sort by position and join
        phrase_tokens.sort(key=lambda t: t.i)
        return " ".join([t.text for t in phrase_tokens]).lower()
    
    def similarity(self, text1: str, text2: str) -> float:
        """
        Compute semantic similarity between two texts.
        
        Uses spaCy's built-in word vectors if available.
        
        Args:
            text1: First text
            text2: Second text
            
        Returns:
            Similarity score (0.0 - 1.0)
        """
        if not text1 or not text2:
            return 0.0
        
        doc1 = self.nlp(text1)
        doc2 = self.nlp(text2)
        
        # Use built-in similarity if vectors available
        if doc1.has_vector and doc2.has_vector:
            return float(doc1.similarity(doc2))
        
        # Fallback to token overlap
        tokens1 = set(self.extract_meaningful_tokens(text1))
        tokens2 = set(self.extract_meaningful_tokens(text2))
        
        if not tokens1 or not tokens2:
            return 0.0
        
        intersection = len(tokens1 & tokens2)
        union = len(tokens1 | tokens2)
        
        return intersection / union if union > 0 else 0.0


# Backward compatibility: Simple functions for existing code
_default_processor: Optional[TextProcessor] = None


def _get_processor() -> TextProcessor:
    """Get or create default text processor."""
    global _default_processor
    if _default_processor is None:
        _default_processor = TextProcessor()
    return _default_processor


def extract_words(text: str) -> List[str]:
    """
    Extract meaningful words from text (backward compatible).
    
    Now uses spaCy instead of naive regex.
    
    Args:
        text: Input text
        
    Returns:
        List of lemmatized tokens
    """
    try:
        processor = _get_processor()
        return processor.extract_meaningful_tokens(text)
    except (ImportError, OSError):
        # Fallback to old implementation if spaCy not available
        logger.warning("spaCy not available, using fallback text processing")
        import re
        
        words = re.findall(r"\b\w+\b", text.lower())
        stop_words = {
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to",
            "for", "of", "with", "by", "is", "was", "are", "were"
        }
        return [w for w in words if len(w) > 2 and w not in stop_words]


def clean_text(text: str) -> str:
    """
    Clean and normalize text for processing.
    
    Args:
        text: Raw input text
        
    Returns:
        Cleaned text
    """
    import re
    
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
