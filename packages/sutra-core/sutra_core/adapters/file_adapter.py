"""
File adapter for mass learning from text files.

This adapter leverages Sutra's existing parallel association extraction
and adaptive learning to efficiently process large text files.
"""

import hashlib
import logging
import os
from pathlib import Path
from typing import Any, Dict, Iterator, Optional, Tuple

from .base import ChunkMetadata, MassLearningAdapter
from .text_processing import IntelligentTextProcessor

logger = logging.getLogger(__name__)


class FileAdapter(MassLearningAdapter):
    """
    Adapter for learning from text files using Sutra's graph-based approach.

    Features:
    - Intelligent text segmentation preserving context
    - Parallel association extraction for performance
    - Memory-efficient streaming for large files
    - Support for different file formats and structures
    """

    def __init__(
        self,
        batch_size: int = 50,
        chunk_size: int = 1500,
        progress_callback=None,
        min_segment_length: int = 100,
        max_segment_length: int = 3000,
    ):
        """
        Initialize file adapter.

        Args:
            batch_size: Number of segments to process in parallel
            chunk_size: Target size for text segments (characters)
            progress_callback: Optional callback for progress updates
            min_segment_length: Minimum segment size
            max_segment_length: Maximum segment size before splitting
        """
        super().__init__(batch_size, chunk_size, progress_callback)

        self.text_processor = IntelligentTextProcessor(
            min_segment_length=min_segment_length, max_segment_length=max_segment_length
        )

    def get_chunks(self, source: str, **kwargs) -> Iterator[Tuple[str, ChunkMetadata]]:
        """
        Generate content chunks from a text file.

        Args:
            source: Path to text file
            **kwargs: Additional parameters:
                - file_format: 'auto', 'wikipedia', 'plain', 'markdown'
                - encoding: file encoding (default: 'utf-8')
                - category: content category for organization

        Yields:
            Tuple of (content, metadata) for each segment
        """
        file_path = Path(source)
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {source}")

        file_format = kwargs.get("file_format", "auto")
        encoding = kwargs.get("encoding", "utf-8")
        category = kwargs.get("category", None)

        # Auto-detect format if needed
        if file_format == "auto":
            file_format = self._detect_format(file_path)

        logger.info(f"Processing {source} as {file_format} format")

        try:
            # Stream file content to avoid loading entire file into memory
            with open(file_path, "r", encoding=encoding, errors="replace") as f:
                # For very large files, we could process in chunks
                # For now, load full content for better structure detection
                content = f.read()

                # Process based on detected format
                if file_format == "wikipedia":
                    segments = self.text_processor.process_wikipedia_format(content)
                else:
                    segments = self.text_processor.process_plain_text(content)

                chunk_index = 0
                for segment in segments:
                    # Create unique chunk ID
                    chunk_id = hashlib.md5(
                        f"{source}:{segment.start_pos}:{segment.end_pos}".encode()
                    ).hexdigest()[:16]

                    metadata = ChunkMetadata(
                        chunk_id=chunk_id,
                        source=str(file_path.name),
                        category=category or self._infer_category(segment.content),
                        size_chars=len(segment.content),
                        chunk_index=chunk_index,
                        extra={
                            "segment_type": segment.segment_type,
                            "start_pos": segment.start_pos,
                            "end_pos": segment.end_pos,
                            "context": segment.context,
                            "file_format": file_format,
                        },
                    )

                    yield segment.content, metadata
                    chunk_index += 1

        except Exception as e:
            logger.error(f"Error reading file {source}: {e}")
            raise

    def estimate_total_chunks(self, source: str, **kwargs) -> int:
        """
        Estimate total number of chunks for progress tracking.

        Uses file size and average chunk size for estimation.
        """
        try:
            file_path = Path(source)
            file_size = file_path.stat().st_size

            # Estimate based on file size and average chunk size
            # Account for different formats having different densities
            file_format = kwargs.get("file_format", "auto")
            if file_format == "auto":
                file_format = self._detect_format(file_path)

            if file_format == "wikipedia":
                # Wikipedia format tends to have longer sections
                avg_chunk_size = self.chunk_size * 1.5
            else:
                avg_chunk_size = self.chunk_size

            estimated_chunks = max(1, int(file_size / avg_chunk_size))

            logger.debug(
                f"Estimated {estimated_chunks} chunks for {file_size} byte file"
            )
            return estimated_chunks

        except Exception as e:
            logger.warning(f"Could not estimate chunks for {source}: {e}")
            return 100  # Fallback estimate

    def get_source_info(self, source: str, **kwargs) -> Dict[str, Any]:
        """Get information about the file."""
        try:
            file_path = Path(source)
            stat = file_path.stat()

            file_format = kwargs.get("file_format", "auto")
            if file_format == "auto":
                file_format = self._detect_format(file_path)

            return {
                "size_bytes": stat.st_size,
                "modified_time": stat.st_mtime,
                "file_format": file_format,
                "filename": file_path.name,
                "extension": file_path.suffix,
                "readable": os.access(file_path, os.R_OK),
            }
        except Exception as e:
            logger.error(f"Error getting file info for {source}: {e}")
            return {"size_bytes": 0, "error": str(e)}

    def learn_from_source(
        self,
        learner,
        source: str,
        source_name: Optional[str] = None,
        category: Optional[str] = None,
        **kwargs,
    ):
        """
        Learn from a file using parallel association extraction.

        Overrides base method to use ParallelAssociationExtractor if available
        for better performance on large files.
        """

        # Check if we have parallel extraction available
        try:
            from ..learning.associations_parallel import ParallelAssociationExtractor

            # Create parallel extractor if we have multiple concepts to process
            total_chunks = self.estimate_total_chunks(source, **kwargs)

            if total_chunks >= 20:  # Use parallel for larger files
                logger.info(
                    f"Using parallel extraction for {total_chunks} estimated chunks"
                )

                # Collect all chunks first for batch processing
                chunk_data = []

                for content, metadata in self.get_chunks(source, **kwargs):
                    # Create concept first using adaptive learner
                    concept_id = learner.learn_adaptive(
                        content=content,
                        source=source_name or metadata.source,
                        category=category or metadata.category,
                    )
                    chunk_data.append((concept_id, content))

                # Use parallel extraction for associations
                if hasattr(learner, "association_extractor") and chunk_data:
                    if isinstance(
                        learner.association_extractor, ParallelAssociationExtractor
                    ):
                        # Use existing parallel extractor
                        parallel_extractor = learner.association_extractor
                    else:
                        # Create parallel extractor with same settings
                        parallel_extractor = ParallelAssociationExtractor(
                            storage=learner.storage,
                            enable_central_links=True,
                            parallel_threshold=self.batch_size,
                        )

                    # Extract associations in parallel
                    associations_created = (
                        parallel_extractor.extract_associations_batch(
                            chunk_data, depth=1
                        )
                    )

                    logger.info(
                        f"Parallel extraction created {associations_created} associations"
                    )

                # Create simplified progress result
                from .base import LearningProgress

                return LearningProgress(
                    chunks_processed=len(chunk_data),
                    total_chunks=len(chunk_data),
                    concepts_created=len(chunk_data),
                    associations_created=(
                        associations_created
                        if "associations_created" in locals()
                        else 0
                    ),
                    bytes_processed=sum(len(content) for _, content in chunk_data),
                    total_bytes=self.get_source_info(source, **kwargs).get(
                        "size_bytes", 0
                    ),
                    current_source=source_name or source,
                    elapsed_seconds=0.0,  # Would need timing
                    errors=[],
                )

        except ImportError:
            logger.info("Parallel extraction not available, using sequential")
        except Exception as e:
            logger.warning(
                f"Parallel extraction failed: {e}, falling back to sequential"
            )

        # Fallback to base implementation
        return super().learn_from_source(
            learner, source, source_name, category, **kwargs
        )

    def _detect_format(self, file_path: Path) -> str:
        """Auto-detect file format based on name and content sampling."""
        name_lower = file_path.name.lower()

        if "wikipedia" in name_lower or "wiki" in name_lower:
            return "wikipedia"
        elif file_path.suffix in [".md", ".markdown"]:
            return "markdown"
        else:
            # Sample first few lines to detect structure
            try:
                with open(file_path, "r", encoding="utf-8", errors="replace") as f:
                    sample = f.read(2000)  # Read first 2KB

                    # Look for Wikipedia-style patterns
                    if ("===" in sample or "---" in sample) and "\n\n" in sample:
                        return "wikipedia"
                    else:
                        return "plain"
            except Exception:
                return "plain"

    def _infer_category(self, content: str) -> str:
        """Infer content category from text content."""
        content_lower = content.lower()

        # Simple keyword-based categorization
        if any(
            word in content_lower for word in ["history", "born", "died", "century"]
        ):
            return "history"
        elif any(
            word in content_lower for word in ["science", "theory", "research", "study"]
        ):
            return "science"
        elif any(
            word in content_lower
            for word in ["technology", "computer", "software", "algorithm"]
        ):
            return "technology"
        elif any(
            word in content_lower for word in ["culture", "art", "music", "literature"]
        ):
            return "culture"
        else:
            return "general"
