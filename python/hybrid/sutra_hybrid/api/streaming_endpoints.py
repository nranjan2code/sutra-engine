"""
Streaming API endpoints using Server-Sent Events (SSE).

Provides real-time progressive answer refinement with:
- Initial fast response
- Progressive refinement as more paths are found
- Final consensus answer
- Quality gate validation on streaming results
"""

import asyncio
import logging
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

from ..engine import SutraAI
from sutra_core.streaming import create_async_engine, format_sse_chunk
from sutra_core.quality_gates import create_quality_validator, MODERATE_QUALITY_GATE

logger = logging.getLogger(__name__)

# Router for streaming endpoints
router = APIRouter(prefix="/sutra/stream", tags=["Streaming"])

# Global AI instance (will be injected)
_ai_instance: Optional[SutraAI] = None


def set_ai_instance(ai: SutraAI) -> None:
    """Set the global AI instance."""
    global _ai_instance
    _ai_instance = ai


def get_ai() -> SutraAI:
    """Dependency to get AI instance."""
    if _ai_instance is None:
        raise HTTPException(status_code=500, detail="AI instance not initialized")
    return _ai_instance


class StreamingQueryRequest(BaseModel):
    """Request model for streaming query."""
    query: str
    max_concepts: int = 10
    enable_quality_gates: bool = True
    min_confidence: float = 0.3


@router.post("/query")
async def stream_query(
    request: StreamingQueryRequest,
    ai: SutraAI = Depends(get_ai),
):
    """
    Stream query results with progressive answer refinement.
    
    Returns Server-Sent Events (SSE) stream with:
    - `stage`: initial, refining, consensus, complete
    - `answer`: Current best answer
    - `confidence`: Current confidence score
    - `paths_found`: Number of reasoning paths found so far
    - `is_final`: Whether this is the final answer
    
    Example usage:
        ```javascript
        const eventSource = new EventSource('/sutra/stream/query', {
            method: 'POST',
            body: JSON.stringify({query: "What is AI?"}),
        });
        
        eventSource.onmessage = (event) => {
            const chunk = JSON.parse(event.data);
            console.log(`${chunk.stage}: ${chunk.answer}`);
            
            if (chunk.is_final) {
                eventSource.close();
            }
        };
        ```
    
    Args:
        request: Streaming query request
        ai: SutraAI instance (injected)
    
    Returns:
        SSE streaming response
    """
    
    async def event_generator():
        """Generate SSE events."""
        try:
            # Create async engine wrapper
            async_engine = create_async_engine(ai._core)
            
            # Optional: Create quality validator
            validator = None
            if request.enable_quality_gates:
                validator = create_quality_validator(
                    ai._core.storage,
                    MODERATE_QUALITY_GATE
                )
            
            # Stream query results
            async for chunk in async_engine.ask_stream(
                request.query,
                max_concepts=request.max_concepts
            ):
                # Apply quality gates if enabled
                if validator and chunk.is_final and request.enable_quality_gates:
                    assessment = validator.validate(
                        raw_confidence=chunk.confidence,
                        consensus_strength=0.5,  # Approximate from chunk
                        num_paths=chunk.paths_found,
                        has_evidence=chunk.paths_found > 0
                    )
                    
                    if not assessment.passed:
                        # Replace with "I don't know" response
                        chunk.answer = "I don't have enough information to answer confidently."
                        chunk.confidence = 0.0
                        chunk.reasoning_explanation = assessment.recommendation
                
                # Format as SSE and yield
                sse_message = format_sse_chunk(chunk, event_name="chunk")
                yield sse_message
            
            # Send completion event
            yield "event: done\ndata: {\"status\": \"complete\"}\n\n"
        
        except Exception as e:
            logger.error(f"Streaming error: {e}")
            # Send error event
            error_data = f'{{\"error\": \"{str(e)}\"}}'
            yield f"event: error\ndata: {error_data}\n\n"
    
    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "X-Accel-Buffering": "no",  # Disable nginx buffering
        }
    )


@router.get("/health")
async def streaming_health():
    """Health check for streaming endpoints."""
    return {
        "status": "healthy",
        "streaming_enabled": True,
        "protocol": "Server-Sent Events (SSE)"
    }
