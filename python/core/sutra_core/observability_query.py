"""
Natural Language Observability Queries - Query your system using AI.

Uses Sutra's own reasoning engine to answer operational questions:
- "Show me failed queries in the last hour"
- "What's the average latency today?"
- "Which queries have the lowest confidence?"
- "How many learning events failed?"
- "Show me all high latency operations"

Pure graph reasoning - no external tools needed.
"""

from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta, timezone
import logging

logger = logging.getLogger(__name__)


class ObservabilityQueryInterface:
    """
    Natural language interface for querying operational data.
    
    Leverages Sutra's own reasoning engine to answer questions about
    system performance, errors, and behavior patterns.
    """
    
    def __init__(self, storage):
        """
        Initialize observability query interface.
        
        Args:
            storage: Storage adapter (contains all events as concepts)
        """
        self.storage = storage
    
    def query(self, natural_language_query: str) -> Dict[str, Any]:
        """
        Query operational data using natural language.
        
        Examples:
            >>> interface.query("Show me failed queries today")
            >>> interface.query("What's causing high latency?")
            >>> interface.query("How many events in the last hour?")
        
        Returns:
            Dict with 'answer', 'events', and 'insights'
        """
        # Normalize query
        query_lower = natural_language_query.lower()
        
        # Extract time range
        time_filter = self._extract_time_range(query_lower)
        
        # Extract event type filter
        event_types = self._extract_event_types(query_lower)
        
        # Extract metric type (count, average, etc)
        metric_type = self._extract_metric_type(query_lower)
        
        # Search for matching events
        events = self._search_events(
            event_types=event_types,
            time_range=time_filter,
            query_text=natural_language_query
        )
        
        # Compute metrics
        answer, insights = self._compute_answer(
            events=events,
            metric_type=metric_type,
            query=natural_language_query
        )
        
        return {
            'query': natural_language_query,
            'answer': answer,
            'events': events[:10],  # Top 10 events
            'total_events': len(events),
            'insights': insights,
            'time_range': time_filter,
        }
    
    def _extract_time_range(self, query: str) -> Dict[str, datetime]:
        """Extract time range from query."""
        now = datetime.now(timezone.utc)
        
        if 'last hour' in query or 'past hour' in query:
            return {
                'start': now - timedelta(hours=1),
                'end': now
            }
        elif 'last day' in query or 'today' in query:
            return {
                'start': now - timedelta(days=1),
                'end': now
            }
        elif 'last week' in query or 'past week' in query:
            return {
                'start': now - timedelta(weeks=1),
                'end': now
            }
        else:
            # Default: last 24 hours
            return {
                'start': now - timedelta(days=1),
                'end': now
            }
    
    def _extract_event_types(self, query: str) -> List[str]:
        """Extract event types from query."""
        event_types = []
        
        # Map query keywords to event types
        if 'failed' in query or 'failure' in query or 'error' in query:
            event_types.extend(['query_failed', 'learn_failed', 'storage_error'])
        
        if 'query' in query or 'queries' in query:
            event_types.extend(['query_completed', 'query_failed', 'query_received'])
        
        if 'learn' in query or 'learning' in query:
            event_types.extend(['learn_completed', 'learn_failed'])
        
        if 'slow' in query or 'latency' in query or 'performance' in query:
            event_types.extend(['query_high_latency', 'storage_slow'])
        
        if 'confidence' in query:
            event_types.extend(['query_low_confidence', 'query_completed'])
        
        return list(set(event_types)) if event_types else None
    
    def _extract_metric_type(self, query: str) -> str:
        """Extract metric type from query."""
        if 'count' in query or 'how many' in query:
            return 'count'
        elif 'average' in query or 'avg' in query or 'mean' in query:
            return 'average'
        elif 'max' in query or 'maximum' in query or 'worst' in query:
            return 'max'
        elif 'min' in query or 'minimum' in query or 'best' in query:
            return 'min'
        else:
            return 'list'
    
    def _search_events(
        self,
        event_types: Optional[List[str]],
        time_range: Dict[str, datetime],
        query_text: str
    ) -> List[Dict[str, Any]]:
        """
        Search for events matching criteria.
        
        Uses vector search on event concepts in storage.
        """
        try:
            # Search by category filter
            results = []
            
            if event_types:
                for event_type in event_types:
                    # Get concepts with this event category
                    category = f"event_{event_type}"
                    # Note: Storage would need category filter support
                    # For now, use vector search
                    matches = self.storage.vector_search_by_content(
                        query_text,
                        k=50,
                        category_filter=category
                    )
                    results.extend(matches)
            else:
                # General search across all events
                results = self.storage.vector_search_by_content(
                    query_text,
                    k=100,
                    category_filter="event_"
                )
            
            # Convert to event dicts
            events = []
            for concept_id, similarity in results:
                concept = self.storage.get_concept(concept_id)
                if concept:
                    # Parse timestamp from concept
                    try:
                        # Extract timestamp from category or created field
                        ts = datetime.fromisoformat(concept.created.replace('Z', '+00:00'))
                        
                        # Filter by time range
                        if time_range['start'] <= ts <= time_range['end']:
                            events.append({
                                'id': concept.id,
                                'content': concept.content,
                                'category': concept.category,
                                'timestamp': concept.created,
                                'similarity': similarity
                            })
                    except Exception:
                        pass
            
            return events
            
        except Exception as e:
            logger.error(f\"Event search failed: {e}\")
            return []
    
    def _compute_answer(
        self,
        events: List[Dict[str, Any]],
        metric_type: str,
        query: str
    ) -> tuple[str, List[str]]:
        \"\"\"Compute answer from events.\"\"\"
        if not events:
            return (
                \"No matching events found in the specified time range.\",
                [\"Try expanding the time range or adjusting search criteria\"]
            )
        
        insights = []
        
        if metric_type == 'count':
            answer = f\"Found {len(events)} events matching your query.\"
            
            # Group by category for insights
            categories = {}
            for event in events:
                cat = event.get('category', 'unknown')
                categories[cat] = categories.get(cat, 0) + 1
            
            if categories:
                top_category = max(categories, key=categories.get)
                insights.append(
                    f\"Most common event type: {top_category} ({categories[top_category]} occurrences)\"
                )
        
        elif metric_type == 'average':
            # Extract durations from event content if available
            durations = []
            for event in events:
                content = event.get('content', '')
                if 'ms' in content:
                    try:
                        # Extract duration like "123.4ms"
                        parts = content.split('ms')[0].split()
                        duration = float(parts[-1])
                        durations.append(duration)
                    except (ValueError, IndexError):
                        pass
            
            if durations:
                avg = sum(durations) / len(durations)
                answer = f\"Average duration: {avg:.1f}ms across {len(durations)} operations.\"
                insights.append(f\"Min: {min(durations):.1f}ms, Max: {max(durations):.1f}ms\")
            else:
                answer = f\"Found {len(events)} events but no duration data available.\"
        
        elif metric_type == 'list':
            # Return sample events
            answer = f\"Found {len(events)} matching events. Here are the most recent:\"
            for event in events[:5]:
                insights.append(f\"- {event.get('content', '')[:100]}\")
        
        else:
            answer = f\"Found {len(events)} matching events.\"
        
        # Add general insights
        if len(events) > 50:
            insights.append(\"⚠️ High volume of events detected - consider investigating\")
        
        return answer, insights


def create_observability_interface(storage) -> ObservabilityQueryInterface:
    \"\"\"Factory function to create observability query interface.\"\"\"
    return ObservabilityQueryInterface(storage)
