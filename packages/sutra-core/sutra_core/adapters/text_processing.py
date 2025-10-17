"""
Text processing utilities for mass learning from structured content.

This module provides intelligent text segmentation that preserves context
and respects document structure for optimal concept extraction.
"""

import re
from dataclasses import dataclass
from typing import Iterator, List, Tuple


@dataclass
class TextSegment:
    """A segment of text with metadata."""

    content: str
    start_pos: int
    end_pos: int
    segment_type: str  # 'paragraph', 'sentence', 'section', 'article'
    context: str = ""  # Surrounding context for better understanding


class IntelligentTextProcessor:
    """
    Processes large text files into semantically meaningful segments.

    Unlike typical RAG chunking, this preserves linguistic and structural
    boundaries to maximize concept extraction and association quality.
    """

    def __init__(
        self,
        min_segment_length: int = 50,
        max_segment_length: int = 2000,
        context_window: int = 100,
    ):
        """
        Initialize text processor.

        Args:
            min_segment_length: Minimum characters per segment
            max_segment_length: Maximum characters per segment
            context_window: Characters of context to include
        """
        self.min_segment_length = min_segment_length
        self.max_segment_length = max_segment_length
        self.context_window = context_window

    def process_wikipedia_format(self, content: str) -> Iterator[TextSegment]:
        """
        Process Wikipedia-style text with articles and sections.

        Args:
            content: Raw Wikipedia text content

        Yields:
            TextSegment objects for each article/section
        """
        # Wikipedia article boundaries (common patterns)
        article_pattern = r"^([A-Z][^\n]+)\n={2,}\n"
        section_pattern = r"^([A-Z][^\n]+)\n-{2,}\n"

        # Split by articles first
        articles = re.split(article_pattern, content, flags=re.MULTILINE)

        current_pos = 0

        for i in range(1, len(articles), 2):  # Skip first empty and alternate
            if i + 1 < len(articles):
                title = articles[i].strip()
                article_content = articles[i + 1].strip()

                if (
                    not article_content
                    or len(article_content) < self.min_segment_length
                ):
                    current_pos += len(title) + len(article_content) + 10
                    continue

                # Process sections within article
                sections = re.split(
                    section_pattern, article_content, flags=re.MULTILINE
                )

                if len(sections) > 1:
                    # Multiple sections - process each
                    section_pos = current_pos + len(title) + 10

                    for j in range(1, len(sections), 2):
                        if j + 1 < len(sections):
                            section_title = sections[j].strip()
                            section_content = sections[j + 1].strip()

                            full_content = (
                                f"{title}\n\n{section_title}\n{section_content}"
                            )

                            if len(full_content) >= self.min_segment_length:
                                # Get context from surrounding sections
                                context = self._get_context(
                                    content, section_pos, self.context_window
                                )

                                yield TextSegment(
                                    content=full_content,
                                    start_pos=section_pos,
                                    end_pos=section_pos + len(full_content),
                                    segment_type="section",
                                    context=context,
                                )

                            section_pos += (
                                len(section_title) + len(section_content) + 10
                            )
                else:
                    # Single article without sections
                    full_content = f"{title}\n\n{article_content}"

                    if len(full_content) >= self.min_segment_length:
                        context = self._get_context(
                            content, current_pos, self.context_window
                        )

                        # Split long articles into paragraphs
                        if len(full_content) > self.max_segment_length:
                            yield from self._split_long_content(
                                full_content, current_pos, "article", context
                            )
                        else:
                            yield TextSegment(
                                content=full_content,
                                start_pos=current_pos,
                                end_pos=current_pos + len(full_content),
                                segment_type="article",
                                context=context,
                            )

                current_pos += len(title) + len(article_content) + 10

    def process_plain_text(self, content: str) -> Iterator[TextSegment]:
        """
        Process plain text by paragraphs and sentences.

        Args:
            content: Raw text content

        Yields:
            TextSegment objects for meaningful text units
        """
        # Split by double newlines (paragraphs)
        paragraphs = re.split(r"\n\s*\n", content)
        current_pos = 0

        for paragraph in paragraphs:
            paragraph = paragraph.strip()

            if len(paragraph) < self.min_segment_length:
                current_pos += len(paragraph) + 2
                continue

            context = self._get_context(content, current_pos, self.context_window)

            if len(paragraph) > self.max_segment_length:
                # Split long paragraphs by sentences
                yield from self._split_by_sentences(paragraph, current_pos, context)
            else:
                yield TextSegment(
                    content=paragraph,
                    start_pos=current_pos,
                    end_pos=current_pos + len(paragraph),
                    segment_type="paragraph",
                    context=context,
                )

            current_pos += len(paragraph) + 2

    def _split_long_content(
        self, content: str, start_pos: int, segment_type: str, context: str
    ) -> Iterator[TextSegment]:
        """Split content that's too long into smaller segments."""

        # Try splitting by paragraphs first
        paragraphs = re.split(r"\n\s*\n", content)

        if len(paragraphs) > 1:
            current_pos = start_pos
            current_chunk = ""

            for paragraph in paragraphs:
                paragraph = paragraph.strip()

                if (
                    len(current_chunk + paragraph) > self.max_segment_length
                    and current_chunk
                ):
                    # Yield current chunk
                    yield TextSegment(
                        content=current_chunk.strip(),
                        start_pos=current_pos - len(current_chunk),
                        end_pos=current_pos,
                        segment_type=segment_type,
                        context=context,
                    )
                    current_chunk = paragraph
                else:
                    current_chunk += "\n\n" + paragraph if current_chunk else paragraph

                current_pos += len(paragraph) + 2

            # Yield remaining chunk
            if current_chunk.strip():
                yield TextSegment(
                    content=current_chunk.strip(),
                    start_pos=current_pos - len(current_chunk),
                    end_pos=current_pos,
                    segment_type=segment_type,
                    context=context,
                )
        else:
            # Single paragraph - split by sentences
            yield from self._split_by_sentences(content, start_pos, context)

    def _split_by_sentences(
        self, content: str, start_pos: int, context: str
    ) -> Iterator[TextSegment]:
        """Split content by sentences when paragraphs are too long."""

        # Simple sentence splitting (could be enhanced with NLP)
        sentences = re.split(r"(?<=[.!?])\s+", content)

        current_pos = start_pos
        current_chunk = ""

        for sentence in sentences:
            sentence = sentence.strip()

            if (
                len(current_chunk + sentence) > self.max_segment_length
                and current_chunk
            ):
                if len(current_chunk.strip()) >= self.min_segment_length:
                    yield TextSegment(
                        content=current_chunk.strip(),
                        start_pos=current_pos - len(current_chunk),
                        end_pos=current_pos,
                        segment_type="sentence_group",
                        context=context,
                    )
                current_chunk = sentence
            else:
                current_chunk += " " + sentence if current_chunk else sentence

            current_pos += len(sentence) + 1

        # Yield remaining chunk
        if (
            current_chunk.strip()
            and len(current_chunk.strip()) >= self.min_segment_length
        ):
            yield TextSegment(
                content=current_chunk.strip(),
                start_pos=current_pos - len(current_chunk),
                end_pos=current_pos,
                segment_type="sentence_group",
                context=context,
            )

    def _get_context(self, content: str, pos: int, window: int) -> str:
        """Extract context around a position."""
        start = max(0, pos - window)
        end = min(len(content), pos + window)

        context_text = content[start:end]

        # Clean up context
        context_text = re.sub(r"\s+", " ", context_text).strip()

        return context_text[:200]  # Limit context length
