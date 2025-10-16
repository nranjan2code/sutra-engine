"""OpenAI-compatible API endpoints.

Drop-in replacement for OpenAI API with full explainability.
"""

import time
import uuid
from datetime import datetime
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException

from ..engine import SutraAI
from .models import (
    ChatCompletionChoice,
    ChatCompletionRequest,
    ChatCompletionResponse,
    ChatCompletionUsage,
    ChatMessage,
    ErrorResponse,
)

# Router for OpenAI-compatible endpoints
router = APIRouter(prefix="/v1", tags=["OpenAI-Compatible"])

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


@router.post("/chat/completions", response_model=ChatCompletionResponse)
async def chat_completions(
    request: ChatCompletionRequest,
    ai: SutraAI = Depends(get_ai),
) -> ChatCompletionResponse:
    """OpenAI-compatible chat completions endpoint.

    This endpoint mimics OpenAI's /v1/chat/completions API but uses
    Sutra's explainable reasoning engine instead of a black-box LLM.

    Args:
        request: Chat completion request with messages
        ai: SutraAI instance (injected)

    Returns:
        OpenAI-compatible chat completion response

    Raises:
        HTTPException: If query fails
    """
    try:
        # Extract the last user message as the query
        user_messages = [msg for msg in request.messages if msg.role == "user"]
        if not user_messages:
            raise HTTPException(
                status_code=400, detail="No user message found in request"
            )

        query = user_messages[-1].content

        # Check if there's a system message to learn from
        system_messages = [msg for msg in request.messages if msg.role == "system"]
        if system_messages:
            # Learn from system messages (context)
            for msg in system_messages:
                ai.learn(msg.content)

        # Query the AI with semantic boost enabled by default
        result = ai.ask(query, semantic_boost=True, max_depth=3, max_paths=5)

        # Convert temperature to semantic boost strength (optional mapping)
        # Higher temperature = more semantic boost
        if request.temperature and request.temperature > 0.7:
            # Re-query with stronger semantic emphasis if high temperature
            result = ai.ask(query, semantic_boost=True, max_depth=4, max_paths=10)

        # Build OpenAI-compatible response
        completion_id = f"chatcmpl-{uuid.uuid4().hex[:24]}"
        created_timestamp = int(time.time())

        # Create assistant message with Sutra's response
        assistant_message = ChatMessage(
            role="assistant",
            content=result.answer,
        )

        # Calculate token usage (approximate based on character count)
        prompt_tokens = sum(len(msg.content.split()) for msg in request.messages)
        completion_tokens = len(result.answer.split())
        total_tokens = prompt_tokens + completion_tokens

        # Build choice
        choice = ChatCompletionChoice(
            index=0,
            message=assistant_message,
            finish_reason="stop",
        )

        # Build usage
        usage = ChatCompletionUsage(
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
            total_tokens=total_tokens,
        )

        # Build response
        response = ChatCompletionResponse(
            id=completion_id,
            object="chat.completion",
            created=created_timestamp,
            model=request.model,
            choices=[choice],
            usage=usage,
        )

        return response

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Query failed: {str(e)}")


@router.get("/models")
async def list_models():
    """List available models (OpenAI-compatible).

    Returns:
        List of available Sutra models
    """
    return {
        "object": "list",
        "data": [
            {
                "id": "sutra-1",
                "object": "model",
                "created": int(time.time()),
                "owned_by": "sutra",
                "permission": [],
                "root": "sutra-1",
                "parent": None,
            }
        ],
    }


@router.get("/models/{model_id}")
async def retrieve_model(model_id: str):
    """Retrieve model information (OpenAI-compatible).

    Args:
        model_id: Model identifier

    Returns:
        Model information

    Raises:
        HTTPException: If model not found
    """
    if model_id != "sutra-1":
        raise HTTPException(status_code=404, detail="Model not found")

    return {
        "id": "sutra-1",
        "object": "model",
        "created": int(time.time()),
        "owned_by": "sutra",
        "permission": [],
        "root": "sutra-1",
        "parent": None,
    }
