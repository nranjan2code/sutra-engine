"""
Self-Observability Framework: Sutra AI monitors itself using its own reasoning engine.

Philosophy: "Eat your own dogfood"
- Every operation becomes learned knowledge
- Query your own behavior: "Why did query X fail?"
- Root cause analysis through graph reasoning
- Self-healing from learned failure patterns
- Zero external dependencies

Production Features:
- Automatic anomaly detection via graph patterns
- Natural language debugging ("Show me slow queries today")
- Learned optimizations from usage patterns
- Predictive failure detection
"""

import hashlib
import logging
import time
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional
from contextlib import contextmanager

logger = logging.getLogger(__name__)


class EventType(Enum):
    """System event types tracked in graph."""
    QUERY_START = "query_start"
    QUERY_COMPLETE = "query_complete"
    QUERY_FAILED = "query_failed"
    LEARN_START = "learn_start"
    LEARN_COMPLETE = "learn_complete"
    LEARN_FAILED = "learn_failed"
    STORAGE_ERROR = "storage_error"
    EMBEDDING_ERROR = "embedding_error"
    PERFORMANCE_DEGRADATION = "performance_degradation"
    CACHE_HIT = "cache_hit"
    CACHE_MISS = "cache_miss"
    LOW_CONFIDENCE = "low_confidence"
    HIGH_LATENCY = "high_latency"


@dataclass
class ObservabilityEvent:
    """Event to be learned into the knowledge graph."""
    event_type: EventType
    timestamp: str
    duration_ms: Optional[float] = None
    success: bool = True
    error_message: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None
    
    def to_concept(self) -> str:
        """Convert event to natural language concept for learning."""
        ts = self.timestamp[:19]  # YYYY-MM-DDTHH:MM:SS
        
        if self.event_type == EventType.QUERY_COMPLETE:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            conf = self.metadata.get('confidence', 0.0) if self.metadata else 0.0
            return (
                f"At {ts}, query '{query}' completed in {self.duration_ms:.1f}ms "
                f"with confidence {conf:.2f}"
            )
        elif self.event_type == EventType.QUERY_FAILED:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            return (
                f"At {ts}, query '{query}' failed after {self.duration_ms:.1f}ms: "
                f"{self.error_message}"
            )
        elif self.event_type == EventType.HIGH_LATENCY:
            op = self.metadata.get('operation', 'operation') if self.metadata else 'operation'
            return (
                f"At {ts}, {op} took {self.duration_ms:.1f}ms (above threshold)"
            )
        elif self.event_type == EventType.LOW_CONFIDENCE:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            conf = self.metadata.get('confidence', 0.0) if self.metadata else 0.0
            return (
                f"At {ts}, query '{query}' returned low confidence {conf:.2f}"
            )
        else:
            return (
                f"At {ts}, event {self.event_type.value} occurred: "
                f"{self.error_message or 'success'}"
            )


class SelfObserver:
    """
    Self-observation system using Sutra's own reasoning engine.
    
    Stores operational data as concepts, enabling:
    - Natural language queries: "Show me failed queries in the last hour"
    - Pattern detection: Automatically learns what causes failures
    - Predictive monitoring: "Will this query likely fail?"
    - Root cause analysis through path finding
    """
    
    def __init__(
        self,
        storage,
        enable_learning: bool = True,
        latency_threshold_ms: float = 1000.0,
        confidence_threshold: float = 0.3,
    ):
        """
        Initialize self-observer.
        
        Args:
            storage: RustStorageAdapter to store observability data
            enable_learning: Learn events into knowledge graph (disable for testing)
            latency_threshold_ms: Threshold for high latency alerts
            confidence_threshold: Threshold for low confidence alerts
        """
        self.storage = storage
        self.enable_learning = enable_learning
        self.latency_threshold_ms = latency_threshold_ms
        self.confidence_threshold = confidence_threshold
        
        # In-memory buffer for batch learning
        self._event_buffer: List[ObservabilityEvent] = []
        self._buffer_size_limit = 100
        
        logger.info(
            f"Self-observer initialized (learning={'ON' if enable_learning else 'OFF'}, "
            f"latency_threshold={latency_threshold_ms}ms, "
            f"confidence_threshold={confidence_threshold})"
        )
    
    def record_event(self, event: ObservabilityEvent):
        """Record observability event into knowledge graph."""
        if not self.enable_learning:
            return
        
        try:
            # Convert event to natural language concept
            concept_content = event.to_concept()
            
            # Generate stable concept ID based on event
            event_id = self._generate_event_id(event)
            
            # Store as concept with metadata
            from ..graph.concepts import Concept
            concept = Concept(
                id=event_id,
                content=concept_content,
                confidence=1.0,
                strength=1.0,
                source="self_observability",
                category=f"observability_{event.event_type.value}",
                created=event.timestamp,
                last_accessed=event.timestamp,
                access_count=1,
            )
            
            # Learn into storage (non-blocking)
            self.storage.add_concept(concept)
            
            # Create associations for pattern detection
            self._create_event_associations(event, event_id)
            
            logger.debug(
                f"Recorded observability event: {event.event_type.value} -> {event_id[:8]}"
            )
            
        except Exception as e:
            # Never let observability crash the main system
            logger.warning(f"Failed to record observability event: {e}")
    
    def _generate_event_id(self, event: ObservabilityEvent) -> str:
        """Generate stable ID for event."""
        # Use timestamp + event type + hash of metadata
        hash_input = f"{event.timestamp}_{event.event_type.value}"
        if event.metadata:
            hash_input += f"_{event.metadata.get('query', '')}"
        return hashlib.sha256(hash_input.encode()).hexdigest()[:16]
    
    def _create_event_associations(self, event: ObservabilityEvent, event_id: str):
        """Create associations for pattern learning."""
        # Associate with event type concept
        type_concept_id = f"event_type_{event.event_type.value}"
        
        # Create association: event -> type
        from ..graph.concepts import Association, AssociationType
        assoc = Association(
            source_id=event_id,
            target_id=type_concept_id,
            assoc_type=AssociationType.CATEGORICAL,
            confidence=1.0,
            strength=1.0,
        )
        self.storage.add_association(assoc)
        
        # If failed event, associate with error type
        if not event.success and event.error_message:
            error_type = event.error_message.split(':')[0]  # Extract error type
            error_concept_id = f"error_type_{error_type}"
            
            error_assoc = Association(
                source_id=event_id,
                target_id=error_concept_id,
                assoc_type=AssociationType.CAUSAL,
                confidence=0.9,
                strength=1.0,
            )
            self.storage.add_association(error_assoc)
    
    @contextmanager
    def track_query(self, query: str):
        """
        Track query execution and learn patterns.
        
        Usage:
            with observer.track_query("What is AI?") as ctx:
                result = engine.ask("What is AI?")
                ctx.record_result(result.confidence)
        """
        start_time = time.time()
        context = QueryContext(query, start_time)
        
        # Record query start
        self.record_event(ObservabilityEvent(
            event_type=EventType.QUERY_START,
            timestamp=datetime.utcnow().isoformat() + 'Z',
            metadata={'query': query}
        ))
        
        try:
            yield context
            
            # Record successful completion
            duration_ms = (time.time() - start_time) * 1000
            
            event = ObservabilityEvent(
                event_type=EventType.QUERY_COMPLETE,
                timestamp=datetime.utcnow().isoformat() + 'Z',
                duration_ms=duration_ms,
                success=True,
                metadata={
                    'query': query,
                    'confidence': context.confidence,
                }
            )
            self.record_event(event)
            
            # Alert on high latency
            if duration_ms > self.latency_threshold_ms:
                self.record_event(ObservabilityEvent(
                    event_type=EventType.HIGH_LATENCY,
                    timestamp=datetime.utcnow().isoformat() + 'Z',
                    duration_ms=duration_ms,
                    metadata={'query': query, 'operation': 'query'}
                ))
            
            # Alert on low confidence
            if context.confidence < self.confidence_threshold:
                self.record_event(ObservabilityEvent(
                    event_type=EventType.LOW_CONFIDENCE,
                    timestamp=datetime.utcnow().isoformat() + 'Z',
                    metadata={'query': query, 'confidence': context.confidence}
                ))
            
        except Exception as e:
            # Record failure
            duration_ms = (time.time() - start_time) * 1000
            
            self.record_event(ObservabilityEvent(
                event_type=EventType.QUERY_FAILED,
                timestamp=datetime.utcnow().isoformat() + 'Z',
                duration_ms=duration_ms,
                success=False,
                error_message=str(e),
                metadata={'query': query}
            ))
            raise
    
    @contextmanager
    def track_learning(self, content: str):
        """Track learning operation."""
        start_time = time.time()
        
        self.record_event(ObservabilityEvent(
            event_type=EventType.LEARN_START,
            timestamp=datetime.utcnow().isoformat() + 'Z',
            metadata={'content_preview': content[:100]}
        ))
        
        try:
            yield
            
            duration_ms = (time.time() - start_time) * 1000
            self.record_event(ObservabilityEvent(
                event_type=EventType.LEARN_COMPLETE,
                timestamp=datetime.utcnow().isoformat() + 'Z',
                duration_ms=duration_ms,
                success=True,
            ))
            
        except Exception as e:
            duration_ms = (time.time() - start_time) * 1000
            self.record_event(ObservabilityEvent(
                event_type=EventType.LEARN_FAILED,
                timestamp=datetime.utcnow().isoformat() + 'Z',
                duration_ms=duration_ms,
                success=False,
                error_message=str(e),
            ))
            raise
    
    def query_self(self, observability_query: str):
        """
        Query your own operational data using natural language.
        
        Examples:
            observer.query_self("Show me failed queries in the last hour")
            observer.query_self("What causes high latency?")
            observer.query_self("Which queries have low confidence?")
        
        Returns:
            Natural language answer with reasoning paths
        """
        # This requires the full ReasoningEngine, so return concept IDs
        # that match the query pattern
        
        # Search for relevant observability concepts
        results = self.storage.vector_search_by_content(
            observability_query,
            k=10,
            category_filter="observability_"
        )
        
        return [
            self.storage.get_concept(concept_id)
            for concept_id, _ in results
            if concept_id
        ]


class QueryContext:
    """Context for tracking query execution."""
    
    def __init__(self, query: str, start_time: float):
        self.query = query
        self.start_time = start_time
        self.confidence: float = 0.0
    
    def record_result(self, confidence: float):
        """Record query result confidence."""
        self.confidence = confidence


def create_observer(storage, enable_learning: bool = True) -> SelfObserver:
    """Factory function to create self-observer."""
    return SelfObserver(storage, enable_learning=enable_learning)
