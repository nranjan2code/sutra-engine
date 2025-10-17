"""
Dataset adapter for mass learning from HuggingFace and other structured datasets.

This adapter efficiently processes large text datasets like Wikipedia dumps,
handling article boundaries and streaming for memory efficiency.
"""

import hashlib
import logging
import re
from pathlib import Path
from typing import Any, Dict, Iterator, Optional, Tuple

from .base import ChunkMetadata, MassLearningAdapter
from .text_formats import FormatDetector, ProcessingStrategy, TextFormat

logger = logging.getLogger(__name__)


class DatasetAdapter(MassLearningAdapter):
    """
    Adapter for learning from structured text datasets.

    Optimized for:
    - Wikipedia datasets with article boundaries
    - Large text files that need streaming
    - Preserving document structure and context
    - Memory-efficient processing of 100MB+ files
    """

    def __init__(
        self,
        batch_size: int = 100,
        chunk_size: int = 2000,
        progress_callback=None,
        min_article_length: int = 200,
        max_article_length: int = 10000,
        stream_buffer_size: int = 8192,
    ):
        """
        Initialize dataset adapter.

        Args:
            batch_size: Number of articles to process in parallel
            chunk_size: Target size for article segments
            progress_callback: Optional callback for progress updates
            min_article_length: Skip articles shorter than this
            max_article_length: Split articles longer than this
            stream_buffer_size: Buffer size for file streaming (bytes)
        """
        super().__init__(batch_size, chunk_size, progress_callback)

        self.min_article_length = min_article_length
        self.max_article_length = max_article_length
        self.stream_buffer_size = stream_buffer_size

    def get_chunks(self, source: str, **kwargs) -> Iterator[Tuple[str, ChunkMetadata]]:
        """
        Generate article chunks from dataset file.

        Args:
            source: Path to dataset file
            **kwargs: Additional parameters:
                - encoding: file encoding (default: 'utf-8')
                - category: content category
                - article_separator: regex pattern for article boundaries

        Yields:
            Tuple of (article_content, metadata) for each article
        """
        file_path = Path(source)
        if not file_path.exists():
            raise FileNotFoundError(f"Dataset file not found: {source}")

        encoding = kwargs.get("encoding", "utf-8")
        category = kwargs.get("category", "encyclopedia")

        # Auto-detect text format or use provided
        text_format = kwargs.get("text_format", None)
        if text_format is None:
            # Sample file to detect format
            with open(file_path, "r", encoding=encoding, errors="replace") as f:
                sample = f.read(10000)  # Read first 10KB for detection
            text_format = FormatDetector.detect_format(sample, str(file_path))
        elif isinstance(text_format, str):
            # Convert string to enum
            text_format = TextFormat(text_format)

        # Get processing strategy for this format
        strategy = ProcessingStrategy.get_strategy(text_format)
        article_separator = strategy["separator_pattern"]

        logger.info(
            f"Processing dataset: {source} ({self._format_size(file_path.stat().st_size)}) as {text_format.value}"
        )

        try:
            article_index = 0

            # Stream the file to handle large datasets
            with open(
                file_path,
                "r",
                encoding=encoding,
                errors="replace",
                buffering=self.stream_buffer_size,
            ) as f:

                buffer = ""

                while True:
                    # Read chunk into buffer
                    chunk = f.read(self.stream_buffer_size)
                    if not chunk:
                        # Process final article if any
                        if buffer.strip():
                            yield from self._process_article(
                                buffer.strip(),
                                article_index,
                                str(file_path.name),
                                category,
                            )
                        break

                    buffer += chunk

                    # Split by article boundaries
                    articles = re.split(article_separator, buffer)

                    # Keep last incomplete article in buffer
                    buffer = articles[-1]

                    # Process complete articles
                    for article_text in articles[:-1]:
                        article_text = article_text.strip()

                        if len(article_text) >= self.min_article_length:
                            yield from self._process_article(
                                article_text,
                                article_index,
                                str(file_path.name),
                                category,
                            )
                            article_index += 1

        except Exception as e:
            logger.error(f"Error reading dataset {source}: {e}")
            raise

    def _process_article(
        self, article_text: str, article_index: int, source_name: str, category: str
    ) -> Iterator[Tuple[str, ChunkMetadata]]:
        """Process a single article, potentially splitting if too long."""

        # Extract title from first line
        lines = article_text.split("\n", 1)
        if len(lines) >= 2:
            title = lines[0].strip()
            content = lines[1].strip()
        else:
            title = f"Article {article_index}"
            content = article_text

        # Skip very short articles
        if len(content) < self.min_article_length:
            return

        # Split very long articles
        if len(article_text) > self.max_article_length:
            yield from self._split_long_article(
                title, content, article_index, source_name, category
            )
        else:
            # Process as single article
            chunk_id = hashlib.md5(
                f"{source_name}:{article_index}".encode()
            ).hexdigest()[:16]

            # Infer more specific category from content
            inferred_category = self._infer_category_detailed(content)
            final_category = (
                inferred_category if inferred_category != "general" else category
            )

            metadata = ChunkMetadata(
                chunk_id=chunk_id,
                source=source_name,
                category=final_category,
                size_chars=len(article_text),
                chunk_index=article_index,
                extra={
                    "article_title": title,
                    "article_index": article_index,
                    "is_split": False,
                    "content_length": len(content),
                },
            )

            yield article_text, metadata

    def _split_long_article(
        self,
        title: str,
        content: str,
        article_index: int,
        source_name: str,
        category: str,
    ) -> Iterator[Tuple[str, ChunkMetadata]]:
        """Split a long article into smaller segments."""

        # Try to split by paragraphs first
        paragraphs = re.split(r"\n\s*\n", content)

        current_segment = title + "\n\n"  # Start each segment with title
        segment_index = 0

        for paragraph in paragraphs:
            paragraph = paragraph.strip()
            if not paragraph:
                continue

            # Check if adding this paragraph would exceed limit
            test_segment = current_segment + paragraph + "\n\n"

            if (
                len(test_segment) > self.max_article_length
                and len(current_segment) > len(title) + 10
            ):
                # Yield current segment
                chunk_id = hashlib.md5(
                    f"{source_name}:{article_index}:{segment_index}".encode()
                ).hexdigest()[:16]

                metadata = ChunkMetadata(
                    chunk_id=chunk_id,
                    source=source_name,
                    category=category,
                    size_chars=len(current_segment),
                    chunk_index=f"{article_index}_{segment_index}",
                    extra={
                        "article_title": title,
                        "article_index": article_index,
                        "segment_index": segment_index,
                        "is_split": True,
                        "total_segments": "?",  # Unknown until complete
                    },
                )

                yield current_segment.strip(), metadata

                # Start new segment
                current_segment = title + "\n\n" + paragraph + "\n\n"
                segment_index += 1
            else:
                current_segment = test_segment

        # Yield final segment
        if len(current_segment.strip()) > len(title) + 10:
            chunk_id = hashlib.md5(
                f"{source_name}:{article_index}:{segment_index}".encode()
            ).hexdigest()[:16]

            metadata = ChunkMetadata(
                chunk_id=chunk_id,
                source=source_name,
                category=category,
                size_chars=len(current_segment),
                chunk_index=f"{article_index}_{segment_index}",
                extra={
                    "article_title": title,
                    "article_index": article_index,
                    "segment_index": segment_index,
                    "is_split": True,
                    "total_segments": segment_index + 1,
                },
            )

            yield current_segment.strip(), metadata

    def estimate_total_chunks(self, source: str, **kwargs) -> int:
        """Estimate total number of articles based on file structure."""
        try:
            file_path = Path(source)
            file_size = file_path.stat().st_size

            # Sample first portion to estimate article density
            sample_size = min(100000, file_size)  # Sample first 100KB

            with open(file_path, "r", encoding="utf-8", errors="replace") as f:
                sample = f.read(sample_size)

            # Count article separators in sample
            article_separator = kwargs.get("article_separator", r"\n\n\n+")
            articles_in_sample = len(re.split(article_separator, sample))

            if articles_in_sample > 1:
                # Estimate based on sample density
                estimated_total = int((file_size / sample_size) * articles_in_sample)

                # Account for long articles that get split
                split_factor = 1.2  # 20% increase for splits
                estimated_total = int(estimated_total * split_factor)
            else:
                # Fallback: estimate based on average article size
                avg_article_size = 3000  # Conservative estimate
                estimated_total = max(1, file_size // avg_article_size)

            logger.debug(
                f"Estimated {estimated_total} articles from {file_size:,} byte file"
            )
            return estimated_total

        except Exception as e:
            logger.warning(f"Could not estimate articles for {source}: {e}")
            # Fallback based on file size
            return max(100, file_path.stat().st_size // 5000)

    def get_source_info(self, source: str, **kwargs) -> Dict[str, Any]:
        """Get detailed information about the dataset file."""
        try:
            file_path = Path(source)
            stat = file_path.stat()

            # Sample file to detect characteristics
            with open(file_path, "r", encoding="utf-8", errors="replace") as f:
                sample = f.read(10000)  # First 10KB

            # Detect likely format
            if "wikipedia" in file_path.name.lower():
                dataset_type = "wikipedia"
            elif "\n\n\n" in sample:
                dataset_type = "multi_article"
            else:
                dataset_type = "single_text"

            # Count newline patterns to estimate structure
            triple_newlines = sample.count("\n\n\n")
            double_newlines = sample.count("\n\n")

            return {
                "size_bytes": stat.st_size,
                "size_formatted": self._format_size(stat.st_size),
                "modified_time": stat.st_mtime,
                "dataset_type": dataset_type,
                "filename": file_path.name,
                "extension": file_path.suffix,
                "readable": True,
                "sample_characteristics": {
                    "triple_newlines_per_10k": triple_newlines,
                    "double_newlines_per_10k": double_newlines,
                    "estimated_article_boundary": triple_newlines > 0,
                },
            }

        except Exception as e:
            logger.error(f"Error analyzing dataset {source}: {e}")
            return {"size_bytes": 0, "error": str(e)}

    def _format_size(self, size_bytes: int) -> str:
        """Format file size in human readable format."""
        for unit in ["B", "KB", "MB", "GB"]:
            if size_bytes < 1024:
                return f"{size_bytes:.1f}{unit}"
            size_bytes /= 1024
        return f"{size_bytes:.1f}TB"

    def _infer_category_detailed(self, content: str) -> str:
        """Enhanced category inference for Wikipedia-style content."""
        content_lower = content.lower()

        # Define keyword sets for different categories
        categories = {
            "biography": [
                "born",
                "died",
                "life",
                "career",
                "childhood",
                "biography",
                "death",
                "birth",
            ],
            "history": [
                "century",
                "war",
                "battle",
                "empire",
                "ancient",
                "historical",
                "dynasty",
                "revolution",
            ],
            "science": [
                "theory",
                "research",
                "study",
                "scientific",
                "experiment",
                "hypothesis",
                "discovery",
            ],
            "technology": [
                "computer",
                "software",
                "algorithm",
                "programming",
                "digital",
                "internet",
                "technical",
            ],
            "geography": [
                "country",
                "city",
                "region",
                "located",
                "population",
                "climate",
                "geography",
                "capital",
            ],
            "arts": [
                "art",
                "music",
                "literature",
                "painting",
                "artist",
                "cultural",
                "creative",
                "aesthetic",
            ],
            "sports": [
                "sport",
                "team",
                "player",
                "game",
                "championship",
                "competition",
                "athletic",
                "tournament",
            ],
            "politics": [
                "government",
                "political",
                "policy",
                "election",
                "democracy",
                "parliament",
                "minister",
            ],
            "nature": [
                "species",
                "animal",
                "plant",
                "habitat",
                "ecosystem",
                "wildlife",
                "conservation",
                "biology",
            ],
            "mathematics": [
                "mathematical",
                "equation",
                "theorem",
                "formula",
                "calculation",
                "geometry",
                "algebra",
            ],
        }

        # Count keyword matches for each category
        category_scores = {}
        for category, keywords in categories.items():
            score = sum(1 for keyword in keywords if keyword in content_lower)
            if score > 0:
                category_scores[category] = score

        if category_scores:
            # Return category with highest score
            best_category = max(category_scores.items(), key=lambda x: x[1])
            return best_category[0]

        return "general"
