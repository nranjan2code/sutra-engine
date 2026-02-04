"""
Application Events - Extends sutra-grid-events for full application observability.

Events stored in Sutra Storage, queryable via natural language:
- "Show me slow queries in the last hour"
- "What queries failed today?"
- "Which operations have high error rates?"
- "Show me learning failures"

Architecture:
    Application → EventEmitter → Storage (TCP) → Natural Language Queries
"""

import time
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Dict, Optional
import json


class ApplicationEventType(Enum):
    """Application-level event types."""
    # Query Events
    QUERY_RECEIVED = "query_received"
    QUERY_COMPLETED = "query_completed"
    QUERY_FAILED = "query_failed"
    QUERY_LOW_CONFIDENCE = "query_low_confidence"
    QUERY_HIGH_LATENCY = "query_high_latency"
    QUERY_CACHE_HIT = "query_cache_hit"
    QUERY_CACHE_MISS = "query_cache_miss"
    
    # Learning Events
    LEARN_RECEIVED = "learn_received"
    LEARN_COMPLETED = "learn_completed"
    LEARN_FAILED = "learn_failed"
    LEARN_BATCH_START = "learn_batch_start"
    LEARN_BATCH_COMPLETED = "learn_batch_completed"
    LEARN_BATCH_FAILED = "learn_batch_failed"
    
    # Storage Events
    STORAGE_READ = "storage_read"
    STORAGE_WRITE = "storage_write"
    STORAGE_ERROR = "storage_error"
    STORAGE_SLOW = "storage_slow"
    
    # Embedding Events
    EMBEDDING_GENERATED = "embedding_generated"
    EMBEDDING_FAILED = "embedding_failed"
    EMBEDDING_SLOW = "embedding_slow"
    
    # Path Finding Events
    PATH_SEARCH_START = "path_search_start"
    PATH_SEARCH_COMPLETED = "path_search_completed"
    PATH_SEARCH_NO_RESULTS = "path_search_no_results"
    PATH_SEARCH_FAILED = "path_search_failed"
    
    # NLG Events
    NLG_GENERATED = "nlg_generated"
    NLG_FAILED = "nlg_failed"
    
    # System Health Events
    SYSTEM_HEALTHY = "system_healthy"
    SYSTEM_DEGRADED = "system_degraded"
    SYSTEM_ERROR = "system_error"


@dataclass
class ApplicationEvent:
    """Base application event."""
    event_type: ApplicationEventType
    timestamp: str
    component: str  # "reasoning_engine", "query_processor", "storage", etc.
    success: bool = True
    duration_ms: Optional[float] = None
    metadata: Optional[Dict[str, Any]] = None
    error_message: Optional[str] = None
    
    def to_concept_content(self) -> str:
        """Convert event to natural language for storage."""
        ts = self.timestamp[:19]  # YYYY-MM-DDTHH:MM:SS
        
        # Map to human-readable descriptions
        if self.event_type == ApplicationEventType.QUERY_COMPLETED:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            conf = self.metadata.get('confidence', 0.0) if self.metadata else 0.0
            return (
                f"{self.component} completed query '{query}' at {ts} "
                f"in {self.duration_ms:.1f}ms with confidence {conf:.2f}"
            )
        elif self.event_type == ApplicationEventType.QUERY_FAILED:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            return (
                f"{self.component} failed query '{query}' at {ts} "
                f"after {self.duration_ms:.1f}ms: {self.error_message}"
            )
        elif self.event_type == ApplicationEventType.QUERY_LOW_CONFIDENCE:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            conf = self.metadata.get('confidence', 0.0) if self.metadata else 0.0
            return (
                f"{self.component} returned low confidence {conf:.2f} for query '{query}' at {ts}"
            )
        elif self.event_type == ApplicationEventType.QUERY_HIGH_LATENCY:
            query = self.metadata.get('query', 'unknown') if self.metadata else 'unknown'
            return (
                f"{self.component} query '{query}' took {self.duration_ms:.1f}ms at {ts} (high latency)"
            )
        elif self.event_type == ApplicationEventType.LEARN_COMPLETED:
            content = self.metadata.get('content_preview', 'unknown') if self.metadata else 'unknown'
            return (
                f"{self.component} learned content '{content}' at {ts} in {self.duration_ms:.1f}ms"
            )
        elif self.event_type == ApplicationEventType.LEARN_FAILED:
            content = self.metadata.get('content_preview', 'unknown') if self.metadata else 'unknown'
            return (
                f"{self.component} failed to learn '{content}' at {ts}: {self.error_message}"
            )
        elif self.event_type == ApplicationEventType.STORAGE_ERROR:
            op = self.metadata.get('operation', 'operation') if self.metadata else 'operation'
            return (
                f"{self.component} storage {op} failed at {ts}: {self.error_message}"
            )
        elif self.event_type == ApplicationEventType.PATH_SEARCH_COMPLETED:
            paths = self.metadata.get('paths_found', 0) if self.metadata else 0
            return (
                f"{self.component} found {paths} reasoning paths at {ts} in {self.duration_ms:.1f}ms"
            )
        elif self.event_type == ApplicationEventType.PATH_SEARCH_NO_RESULTS:
            return (
                f"{self.component} found no reasoning paths at {ts} after {self.duration_ms:.1f}ms"
            )
        else:
            return (
                f"{self.component} {self.event_type.value} at {ts}: "
                f"{'success' if self.success else self.error_message}"
            )
    
    def to_json(self) -> str:
        """Serialize event to JSON."""
        data = asdict(self)
        data['event_type'] = self.event_type.value
        return json.dumps(data)


class EventEmitter:
    """
    Event emitter that writes application events to Sutra Storage via TCP.
    
    Integrates with sutra-grid-events architecture but for application-level events.
    Uses the same TCP protocol and storage backend.
    """
    
    def __init__(self, storage_adapter, component: str = "application"):
        """
        Initialize event emitter.
        
        Args:
            storage_adapter: TcpStorageAdapter or RustStorageAdapter
            component: Component name (e.g., "reasoning_engine", "hybrid_api")
        """
        self.storage = storage_adapter
        self.component = component
        self.enabled = True
        
    def emit(self, event: ApplicationEvent):
        """
        Emit an event (non-blocking, fire-and-forget).
        
        Events are written to storage as concepts with associations for querying.
        """
        if not self.enabled:
            return
        
        try:
            # Generate unique event ID
            event_id = self._generate_event_id(event)
            
            # Convert to natural language concept
            concept_content = event.to_concept_content()
            
            # Write to storage as concept
            from .graph.concepts import Concept, Association, AssociationType
            
            concept = Concept(
                id=event_id,
                content=concept_content,
                confidence=1.0,
                strength=1.0,
                source="application_events",
                category=f"event_{event.event_type.value}",
                created=event.timestamp,
                last_accessed=event.timestamp,
                access_count=1,
            )
            
            # Add concept (non-blocking)
            self.storage.add_concept(concept)
            
            # Create associations for temporal and semantic queries
            self._create_event_associations(event, event_id)
            
        except Exception as e:
            # Never crash the main application due to observability
            import logging
            logging.warning(f"Failed to emit event: {e}")
    
    def _generate_event_id(self, event: ApplicationEvent) -> str:
        """Generate stable event ID."""
        import hashlib
        ts_micro = datetime.fromisoformat(event.timestamp.replace('Z', '+00:00')).timestamp()
        hash_input = f"{event.event_type.value}_{ts_micro}_{event.component}"
        if event.metadata and 'query' in event.metadata:
            hash_input += f"_{event.metadata['query']}"
        return f"evt_{hashlib.sha256(hash_input.encode()).hexdigest()[:16]}"
    
    def _create_event_associations(self, event: ApplicationEvent, event_id: str):
        """Create associations for event querying."""
        from .graph.concepts import Association, AssociationType
        
        try:
            # Association: component -> event_type -> event
            type_concept_id = f"event_type_{event.event_type.value}"
            
            assoc = Association(
                source_id=event_id,
                target_id=type_concept_id,
                assoc_type=AssociationType.CATEGORICAL,
                confidence=1.0,
                strength=1.0,
            )
            self.storage.add_association(assoc)
            
            # Association: event -> timestamp (for temporal queries)
            ts = datetime.fromisoformat(event.timestamp.replace('Z', '+00:00'))
            timestamp_id = f"ts_{ts.strftime('%Y%m%d_%H%M')}"  # Minute-level buckets
            
            ts_assoc = Association(
                source_id=event_id,
                target_id=timestamp_id,
                assoc_type=AssociationType.TEMPORAL,
                confidence=1.0,
                strength=1.0,
            )
            self.storage.add_association(ts_assoc)
            
            # If error event, associate with error type
            if not event.success and event.error_message:
                error_type = event.error_message.split(':')[0] if ':' in event.error_message else event.error_message[:30]
                error_concept_id = f"error_{error_type.replace(' ', '_').lower()}"
                
                error_assoc = Association(
                    source_id=event_id,
                    target_id=error_concept_id,
                    assoc_type=AssociationType.CAUSAL,
                    confidence=0.9,
                    strength=1.0,
                )
                self.storage.add_association(error_assoc)
        
        except Exception:
            # Silent failure for associations
            pass
    
    def emit_query_start(self, query: str):
        """Emit query received event."""
        event = ApplicationEvent(
            event_type=ApplicationEventType.QUERY_RECEIVED,
            timestamp=datetime.now(timezone.utc).isoformat(),
            component=self.component,
            metadata={'query': query}
        )
        self.emit(event)
    
    def emit_query_complete(self, query: str, duration_ms: float, confidence: float):
        """Emit query completed event."""
        event = ApplicationEvent(
            event_type=ApplicationEventType.QUERY_COMPLETED,
            timestamp=datetime.now(timezone.utc).isoformat(),
            component=self.component,
            success=True,
            duration_ms=duration_ms,
            metadata={'query': query, 'confidence': confidence}
        )
        self.emit(event)
    
    def emit_query_failed(self, query: str, duration_ms: float, error: str):
        """Emit query failed event."""
        event = ApplicationEvent(
            event_type=ApplicationEventType.QUERY_FAILED,
            timestamp=datetime.now(timezone.utc).isoformat(),
            component=self.component,
            success=False,
            duration_ms=duration_ms,
            error_message=error,
            metadata={'query': query}
        )
        self.emit(event)
    
    def emit_low_confidence(self, query: str, confidence: float):
        """Emit low confidence alert."""
        event = ApplicationEvent(
            event_type=ApplicationEventType.QUERY_LOW_CONFIDENCE,
            timestamp=datetime.now(timezone.utc).isoformat(),
            component=self.component,
            metadata={'query': query, 'confidence': confidence}
        )
        self.emit(event)
    
    def emit_high_latency(self, query: str, duration_ms: float):
        """Emit high latency alert."""
        event = ApplicationEvent(
            event_type=ApplicationEventType.QUERY_HIGH_LATENCY,
            timestamp=datetime.now(timezone.utc).isoformat(),
            component=self.component,
            duration_ms=duration_ms,
            metadata={'query': query}
        )
        self.emit(event)
