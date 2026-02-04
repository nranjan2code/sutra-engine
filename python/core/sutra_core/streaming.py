"""
Streaming Query Processing with Progressive Answer Refinement.

Streams results as they become available:
1. Initial answer from best path (fast response)
2. Progressive refinement as more paths are found
3. Final consensus answer with full confidence

Uses async/await for non-blocking operation.
Emits events to track streaming performance.
"""

import asyncio
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from enum import Enum
from typing import AsyncIterator, List, Optional, Any
import logging

from .reasoning.mppa import ConsensusResult

logger = logging.getLogger(__name__)


class StreamingStage(Enum):
    """Stages of streaming response."""
    INITIAL = "initial"  # First path found
    REFINING = "refining"  # Adding more paths
    CONSENSUS = "consensus"  # Final consensus reached
    COMPLETE = "complete"  # Stream complete


@dataclass
class StreamingChunk:
    """A chunk of streaming response."""
    stage: StreamingStage
    answer: str
    confidence: float
    paths_found: int
    total_paths_searched: int
    reasoning_explanation: str
    timestamp: str
    is_final: bool = False
    
    def to_dict(self) -> dict:
        """Convert to dictionary for JSON serialization."""
        return {
            'stage': self.stage.value,
            'answer': self.answer,
            'confidence': self.confidence,
            'paths_found': self.paths_found,
            'total_paths_searched': self.total_paths_searched,
            'reasoning_explanation': self.reasoning_explanation,
            'timestamp': self.timestamp,
            'is_final': self.is_final,
        }


class StreamingQueryProcessor:
    """
    Processes queries with progressive answer refinement.
    
    Strategy:
    1. Start path finding immediately
    2. Yield initial answer as soon as first path found
    3. Continue finding paths, yield refined answers
    4. Final yield with full consensus
    """
    
    def __init__(
        self,
        query_processor,
        path_finder,
        mppa,
        storage,
        event_emitter=None,
        target_paths: int = 5,
        min_paths_for_refinement: int = 2,
    ):
        """
        Initialize streaming processor.
        
        Args:
            query_processor: Regular QueryProcessor for concept finding
            path_finder: PathFinder for graph traversal
            mppa: MultiPathAggregator for consensus
            storage: Storage adapter
            event_emitter: Optional event emitter for observability
            target_paths: Target number of paths to find
            min_paths_for_refinement: Min paths before refining answer
        """
        self.query_processor = query_processor
        self.path_finder = path_finder
        self.mppa = mppa
        self.storage = storage
        self.event_emitter = event_emitter
        self.target_paths = target_paths
        self.min_paths_for_refinement = min_paths_for_refinement
    
    async def stream_query(
        self,
        query: str,
        max_concepts: int = 10
    ) -> AsyncIterator[StreamingChunk]:
        """
        Stream query results with progressive refinement.
        
        Yields:
            StreamingChunk objects as answer is refined
        """
        start_time = time.time()
        
        try:
            # Step 1: Find relevant concepts (fast)
            cleaned_query = query.lower().strip()
            query_intent = self.query_processor._classify_query_intent(cleaned_query)
            relevant_concepts = self.query_processor._find_relevant_concepts(
                cleaned_query, max_concepts
            )
            
            if not relevant_concepts:
                # No concepts found - return immediately
                yield StreamingChunk(
                    stage=StreamingStage.COMPLETE,
                    answer="I don't have enough knowledge to answer this question.",
                    confidence=0.0,
                    paths_found=0,
                    total_paths_searched=0,
                    reasoning_explanation="No relevant concepts found",
                    timestamp=datetime.now(timezone.utc).isoformat(),
                    is_final=True
                )
                return
            
            # Step 2: Expand context
            expanded_concepts = self.query_processor._expand_query_context(
                relevant_concepts, query_intent
            )
            
            if len(expanded_concepts) < 2:
                # Not enough concepts for path finding - return direct match
                best_concept = self.storage.get_concept(relevant_concepts[0][0])
                if best_concept:
                    answer = self.query_processor._extract_targeted_answer(
                        best_concept.content,
                        cleaned_query,
                        query_intent,
                        similarity_score=relevant_concepts[0][1]
                    )
                    yield StreamingChunk(
                        stage=StreamingStage.COMPLETE,
                        answer=answer,
                        confidence=best_concept.confidence,
                        paths_found=0,
                        total_paths_searched=0,
                        reasoning_explanation="Direct semantic match (no reasoning paths)",
                        timestamp=datetime.now(timezone.utc).isoformat(),
                        is_final=True
                    )
                return
            
            # Step 3: Progressive path finding
            paths_accumulated = []
            total_searched = 0
            
            # Use async path finding
            async for path_batch in self._find_paths_progressively(
                expanded_concepts,
                query_intent,
                cleaned_query
            ):
                paths_accumulated.extend(path_batch)
                total_searched += len(path_batch)
                
                # Determine stage
                if len(paths_accumulated) == 1:
                    stage = StreamingStage.INITIAL
                elif len(paths_accumulated) < self.target_paths:
                    stage = StreamingStage.REFINING
                else:
                    stage = StreamingStage.CONSENSUS
                
                # Generate answer from current paths
                if paths_accumulated:
                    consensus = self.mppa.aggregate_reasoning_paths(
                        paths_accumulated, query
                    )
                    
                    # Extract targeted answer
                    enhanced_answer = self.query_processor._extract_targeted_answer(
                        consensus.primary_answer,
                        cleaned_query,
                        query_intent,
                        similarity_score=0.5
                    )
                    
                    # Yield chunk
                    yield StreamingChunk(
                        stage=stage,
                        answer=enhanced_answer,
                        confidence=consensus.confidence,
                        paths_found=len(paths_accumulated),
                        total_paths_searched=total_searched,
                        reasoning_explanation=consensus.reasoning_explanation,
                        timestamp=datetime.now(timezone.utc).isoformat(),
                        is_final=False
                    )
                
                # Stop if we have enough paths
                if len(paths_accumulated) >= self.target_paths:
                    break
            
            # Step 4: Final consensus
            if paths_accumulated:
                final_consensus = self.mppa.aggregate_reasoning_paths(
                    paths_accumulated, query
                )
                
                final_answer = self.query_processor._extract_targeted_answer(
                    final_consensus.primary_answer,
                    cleaned_query,
                    query_intent,
                    similarity_score=0.5
                )
                
                yield StreamingChunk(
                    stage=StreamingStage.COMPLETE,
                    answer=final_answer,
                    confidence=final_consensus.confidence,
                    paths_found=len(paths_accumulated),
                    total_paths_searched=total_searched,
                    reasoning_explanation=final_consensus.reasoning_explanation,
                    timestamp=datetime.now(timezone.utc).isoformat(),
                    is_final=True
                )
            
            # Emit completion event
            duration_ms = (time.time() - start_time) * 1000
            if self.event_emitter:
                self.event_emitter.emit_query_complete(
                    query, duration_ms, final_consensus.confidence
                )
        
        except Exception as e:
            logger.error(f"Streaming query failed: {e}")
            
            # Emit failure event
            duration_ms = (time.time() - start_time) * 1000
            if self.event_emitter:
                self.event_emitter.emit_query_failed(query, duration_ms, str(e))
            
            # Yield error chunk
            yield StreamingChunk(
                stage=StreamingStage.COMPLETE,
                answer=f"Error processing query: {str(e)}",
                confidence=0.0,
                paths_found=0,
                total_paths_searched=0,
                reasoning_explanation=f"Query failed: {str(e)}",
                timestamp=datetime.now(timezone.utc).isoformat(),
                is_final=True
            )
    
    async def _find_paths_progressively(
        self,
        concepts: List[str],
        query_intent: dict,
        query: str
    ) -> AsyncIterator[List[Any]]:
        """
        Find paths progressively, yielding batches as found.
        
        Yields:
            Batches of paths as they're discovered
        """
        if len(concepts) < 2:
            return
        
        start_concepts = concepts[:3]
        target_concepts = concepts[:5] if len(concepts) > 3 else concepts
        
        # Run path finding in executor to avoid blocking
        loop = asyncio.get_event_loop()
        
        try:
            # Find paths (this is sync, so run in executor)
            paths = await loop.run_in_executor(
                None,
                self.storage.find_paths,
                start_concepts,
                target_concepts,
                5,  # max_depth
                self.target_paths,
                query
            )
            
            if paths:
                # Yield paths in batches for progressive refinement
                batch_size = max(1, len(paths) // 3)  # 3 updates minimum
                
                for i in range(0, len(paths), batch_size):
                    batch = paths[i:i + batch_size]
                    yield batch
                    
                    # Small delay for progressive streaming effect
                    await asyncio.sleep(0.01)
        
        except Exception as e:
            logger.warning(f"Path finding failed: {e}")
            return


class AsyncReasoningEngine:
    """
    Async version of ReasoningEngine with streaming support.
    
    Wraps synchronous ReasoningEngine with async streaming capabilities.
    """
    
    def __init__(self, reasoning_engine):
        """
        Initialize async wrapper.
        
        Args:
            reasoning_engine: Synchronous ReasoningEngine instance
        """
        self.engine = reasoning_engine
        
        # Create streaming processor
        self.streaming_processor = StreamingQueryProcessor(
            query_processor=reasoning_engine.query_processor,
            path_finder=reasoning_engine.path_finder,
            mppa=reasoning_engine.mppa,
            storage=reasoning_engine.storage,
            event_emitter=getattr(reasoning_engine, '_event_emitter', None),
        )
    
    async def ask_stream(
        self,
        question: str,
        max_concepts: int = 10
    ) -> AsyncIterator[StreamingChunk]:
        """
        Ask a question with streaming response.
        
        Yields:
            StreamingChunk objects as answer is refined
        
        Example:
            async for chunk in engine.ask_stream("What is AI?"):
                print(f"{chunk.stage.value}: {chunk.answer} ({chunk.confidence:.2f})")
        """
        async for chunk in self.streaming_processor.stream_query(
            question, max_concepts
        ):
            yield chunk
    
    async def ask(self, question: str, **kwargs) -> ConsensusResult:
        """
        Async version of regular ask() - returns final result only.
        
        For streaming, use ask_stream() instead.
        """
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            self.engine.ask,
            question
        )


def create_async_engine(reasoning_engine) -> AsyncReasoningEngine:
    """Factory function to create async engine wrapper."""
    return AsyncReasoningEngine(reasoning_engine)


# Utility for Server-Sent Events (SSE) formatting
def format_sse_chunk(chunk: StreamingChunk, event_name: str = "message") -> str:
    """
    Format StreamingChunk as Server-Sent Events (SSE) message.
    
    Args:
        chunk: StreamingChunk to format
        event_name: SSE event name (default: "message")
    
    Returns:
        Formatted SSE string
    """
    import json
    
    data = json.dumps(chunk.to_dict())
    return f"event: {event_name}\ndata: {data}\n\n"
