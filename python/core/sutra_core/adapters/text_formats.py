"""
Text format definitions for different document structures.

This module defines text processing strategies based on STRUCTURE, not content source.
Any dataset can use any format - they are orthogonal concepts.
"""

import re
from enum import Enum
from typing import Dict, List, Tuple


class TextFormat(Enum):
    """
    Text structure formats (independent of content source).

    These define HOW the text is structured, not WHERE it came from.
    """

    # Multi-document formats
    ARTICLE_COLLECTION = (
        "article_collection"  # Multiple articles separated by boundaries
    )
    STRUCTURED_DOCS = "structured_docs"  # Documents with headers/sections

    # Single document formats
    PLAIN_TEXT = "plain_text"  # Simple paragraph-based text
    MARKDOWN = "markdown"  # Markdown-formatted text
    SECTIONED_TEXT = "sectioned_text"  # Text with clear section boundaries

    # Specialized formats
    DIALOGUE = "dialogue"  # Conversation/chat format
    TABULAR = "tabular"  # Structured data in text form
    STREAM = "stream"  # Continuous text stream


class FormatDetector:
    """Detects text format based on structural patterns, not content source."""

    @staticmethod
    def detect_format(content: str, filename: str = "") -> TextFormat:
        """
        Detect text format based on structural patterns.

        Args:
            content: Sample of text content to analyze
            filename: Optional filename for hints (structure, not content)

        Returns:
            Detected TextFormat
        """
        # Check for article collection pattern (what was called "wikipedia")
        if FormatDetector._has_article_boundaries(content):
            return TextFormat.ARTICLE_COLLECTION

        # Check for markdown
        if FormatDetector._has_markdown_structure(content):
            return TextFormat.MARKDOWN

        # Check for sectioned text
        if FormatDetector._has_section_headers(content):
            return TextFormat.SECTIONED_TEXT

        # Check for dialogue
        if FormatDetector._has_dialogue_pattern(content):
            return TextFormat.DIALOGUE

        # Default to plain text
        return TextFormat.PLAIN_TEXT

    @staticmethod
    def _has_article_boundaries(content: str) -> bool:
        """Check if content has multiple articles separated by blank lines."""
        # Look for pattern: Title followed by multiple newlines (article boundaries)
        triple_newlines = content.count("\n\n\n")
        article_title_pattern = r"\n\n\n\s*[A-Z][^\n]+\n\n"

        return (
            triple_newlines >= 2 or len(re.findall(article_title_pattern, content)) >= 2
        )

    @staticmethod
    def _has_markdown_structure(content: str) -> bool:
        """Check for markdown headers."""
        markdown_headers = re.findall(r"^#{1,6}\s", content, re.MULTILINE)
        return len(markdown_headers) >= 2

    @staticmethod
    def _has_section_headers(content: str) -> bool:
        """Check for section headers with underlines."""
        underlined_headers = re.findall(r"^.+\n[-=]{3,}$", content, re.MULTILINE)
        return len(underlined_headers) >= 2

    @staticmethod
    def _has_dialogue_pattern(content: str) -> bool:
        """Check for dialogue/conversation patterns."""
        dialogue_patterns = [
            r"^\w+:\s",  # "Name: message"
            r"^\[\d+:\d+\]",  # "[12:34] message"
            r"^<\w+>",  # "<username> message"
        ]

        for pattern in dialogue_patterns:
            if len(re.findall(pattern, content, re.MULTILINE)) >= 3:
                return True
        return False


class ProcessingStrategy:
    """Maps text formats to processing strategies."""

    FORMAT_STRATEGIES = {
        TextFormat.ARTICLE_COLLECTION: {
            "separator_pattern": r"\n\n\n+",
            "boundary_type": "article",
            "preserve_titles": True,
            "context_window": 200,
        },
        TextFormat.STRUCTURED_DOCS: {
            "separator_pattern": r"\n\n={3,}\n\n",
            "boundary_type": "document",
            "preserve_titles": True,
            "context_window": 150,
        },
        TextFormat.PLAIN_TEXT: {
            "separator_pattern": r"\n\s*\n",
            "boundary_type": "paragraph",
            "preserve_titles": False,
            "context_window": 100,
        },
        TextFormat.MARKDOWN: {
            "separator_pattern": r"\n#{1,6}\s",
            "boundary_type": "section",
            "preserve_titles": True,
            "context_window": 150,
        },
        TextFormat.SECTIONED_TEXT: {
            "separator_pattern": r"\n.+\n[-=]{3,}\n",
            "boundary_type": "section",
            "preserve_titles": True,
            "context_window": 150,
        },
    }

    @staticmethod
    def get_strategy(text_format: TextFormat) -> Dict:
        """Get processing strategy for a text format."""
        return ProcessingStrategy.FORMAT_STRATEGIES.get(
            text_format, ProcessingStrategy.FORMAT_STRATEGIES[TextFormat.PLAIN_TEXT]
        )


def get_format_info() -> Dict[str, str]:
    """Get human-readable descriptions of supported formats."""
    return {
        TextFormat.ARTICLE_COLLECTION.value: "Multiple articles separated by blank lines (e.g., Wikipedia dumps, article collections)",
        TextFormat.STRUCTURED_DOCS.value: "Documents with clear boundaries and headers",
        TextFormat.PLAIN_TEXT.value: "Simple text organized by paragraphs",
        TextFormat.MARKDOWN.value: "Markdown-formatted text with # headers",
        TextFormat.SECTIONED_TEXT.value: "Text with underlined section headers",
        TextFormat.DIALOGUE.value: "Conversation or chat-style text",
        TextFormat.TABULAR.value: "Structured data in text form",
        TextFormat.STREAM.value: "Continuous text without clear boundaries",
    }
