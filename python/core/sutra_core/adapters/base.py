"""
Base adapter interface for mass learning operations.

Defines the abstract interface that all mass learning adapters must implement,
along with common data structures for progress tracking and metadata.
"""

import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Callable, Dict, Iterator, List, Optional

logger = logging.getLogger(__name__)


@dataclass
class ChunkMetadata:
    """Metadata for a learning chunk."""

    chunk_id: str
    source: str
    category: Optional[str] = None
    size_chars: int = 0
    chunk_index: int = 0
    total_chunks: Optional[int] = None
    extra: Optional[Dict[str, Any]] = None


@dataclass
class LearningProgress:
    """Progress information for mass learning operations."""

    chunks_processed: int
    total_chunks: int
    concepts_created: int
    associations_created: int
    bytes_processed: int
    total_bytes: int
    current_source: str
    elapsed_seconds: float
    errors: List[str]

    @property
    def progress_percent(self) -> float:
        """Calculate progress percentage."""
        if self.total_chunks == 0:
            return 0.0
        return (self.chunks_processed / self.total_chunks) * 100.0

    @property
    def bytes_per_second(self) -> float:
        """Calculate processing rate in bytes per second."""
        if self.elapsed_seconds == 0:
            return 0.0
        return self.bytes_processed / self.elapsed_seconds


class MassLearningAdapter(ABC):
    """
    Abstract base class for mass learning adapters.

    All adapters must implement this interface to provide consistent
    mass learning capabilities across different data sources.
    """

    def __init__(
        self,
        batch_size: int = 50,
        chunk_size: int = 1000,
        progress_callback: Optional[Callable[[LearningProgress], None]] = None,
    ):
        """
        Initialize the adapter.

        Args:
            batch_size: Number of chunks to process in each batch
            chunk_size: Target size for text chunks (characters)
            progress_callback: Optional callback for progress updates
        """
        self.batch_size = batch_size
        self.chunk_size = chunk_size
        self.progress_callback = progress_callback

    @abstractmethod
    def get_chunks(self, source: str, **kwargs) -> Iterator[tuple[str, ChunkMetadata]]:
        """
        Generate chunks of content from the data source.

        Args:
            source: Source identifier (file path, database connection, etc.)
            **kwargs: Additional source-specific parameters

        Yields:
            Tuple of (content, metadata) for each chunk
        """
        pass

    @abstractmethod
    def estimate_total_chunks(self, source: str, **kwargs) -> int:
        """
        Estimate the total number of chunks for progress tracking.

        Args:
            source: Source identifier
            **kwargs: Additional parameters

        Returns:
            Estimated number of chunks
        """
        pass

    @abstractmethod
    def get_source_info(self, source: str, **kwargs) -> Dict[str, Any]:
        """
        Get information about the data source.

        Args:
            source: Source identifier
            **kwargs: Additional parameters

        Returns:
            Dictionary with source information (size, type, etc.)
        """
        pass

    def learn_from_source(
        self,
        learner,
        source: str,
        source_name: Optional[str] = None,
        category: Optional[str] = None,
        **kwargs,
    ) -> LearningProgress:
        """
        Learn all content from a data source.

        Args:
            learner: AdaptiveLearner instance
            source: Source identifier
            source_name: Human-readable source name
            category: Content category for organization
            **kwargs: Source-specific parameters

        Returns:
            Final learning progress
        """
        import time

        start_time = time.time()

        # Get source information
        source_info = self.get_source_info(source, **kwargs)
        total_chunks = self.estimate_total_chunks(source, **kwargs)

        # Initialize progress
        progress = LearningProgress(
            chunks_processed=0,
            total_chunks=total_chunks,
            concepts_created=0,
            associations_created=0,
            bytes_processed=0,
            total_bytes=source_info.get("size_bytes", 0),
            current_source=source_name or source,
            elapsed_seconds=0.0,
            errors=[],
        )

        logger.info(f"Starting mass learning from {progress.current_source}")
        logger.info(f"Estimated {total_chunks} chunks, {progress.total_bytes} bytes")

        # Process in batches
        batch_contents = []
        batch_metadatas = []

        try:
            for content, metadata in self.get_chunks(source, **kwargs):
                # Add to current batch
                batch_contents.append(
                    (
                        content,
                        source_name or metadata.source,
                        category or metadata.category,
                    )
                )
                batch_metadatas.append(metadata)

                # Process batch when full
                if len(batch_contents) >= self.batch_size:
                    self._process_batch(
                        learner, batch_contents, batch_metadatas, progress
                    )
                    batch_contents = []
                    batch_metadatas = []

                    # Update progress
                    progress.elapsed_seconds = time.time() - start_time
                    if self.progress_callback:
                        self.progress_callback(progress)

        except Exception as e:
            error_msg = f"Error processing {source}: {str(e)}"
            progress.errors.append(error_msg)
            logger.error(error_msg, exc_info=True)

        # Process remaining items
        if batch_contents:
            self._process_batch(learner, batch_contents, batch_metadatas, progress)

        # Final progress update
        progress.elapsed_seconds = time.time() - start_time
        if self.progress_callback:
            self.progress_callback(progress)

        logger.info(
            f"Mass learning completed: {progress.chunks_processed} chunks, "
            f"{progress.concepts_created} concepts, {progress.associations_created} associations"
        )

        return progress

    def _process_batch(
        self,
        learner,
        batch_contents: List[tuple[str, str, Optional[str]]],
        batch_metadatas: List[ChunkMetadata],
        progress: LearningProgress,
    ) -> None:
        """Process a batch of content chunks."""

        try:
            # Learn batch using adaptive learner
            for content, source, category in batch_contents:
                concept_id = learner.learn_adaptive(
                    content=content, source=source, category=category
                )

                # Update statistics (simplified - could be more accurate)
                progress.concepts_created += 1
                progress.associations_created += 2  # Estimate

            # Update progress counters
            progress.chunks_processed += len(batch_contents)
            progress.bytes_processed += sum(m.size_chars for m in batch_metadatas)

        except Exception as e:
            error_msg = f"Error processing batch: {str(e)}"
            progress.errors.append(error_msg)
            logger.error(error_msg, exc_info=True)
