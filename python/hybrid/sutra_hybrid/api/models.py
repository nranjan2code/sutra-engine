"""Pydantic models for Sutra AI API.

OpenAI-compatible and custom Sutra endpoints.
"""

from datetime import datetime
from typing import Any, Dict, List, Literal, Optional

from pydantic import BaseModel, Field

# ============================================================================
# OpenAI-Compatible Models
# ============================================================================


class ChatMessage(BaseModel):
    """OpenAI chat message format."""

    role: Literal["system", "user", "assistant"] = Field(
        ..., description="Role of the message sender"
    )
    content: str = Field(..., description="Content of the message")


class ChatCompletionRequest(BaseModel):
    """OpenAI chat completion request."""

    model: str = Field(default="sutra-1", description="Model identifier")
    messages: List[ChatMessage] = Field(..., description="List of chat messages")
    temperature: Optional[float] = Field(
        default=0.7, ge=0.0, le=2.0, description="Sampling temperature"
    )
    max_tokens: Optional[int] = Field(
        default=None, ge=1, description="Maximum tokens to generate"
    )
    stream: Optional[bool] = Field(default=False, description="Stream responses")


class ChatCompletionChoice(BaseModel):
    """OpenAI chat completion choice."""

    index: int = Field(..., description="Choice index")
    message: ChatMessage = Field(..., description="Generated message")
    finish_reason: str = Field(..., description="Reason for finishing")


class ChatCompletionUsage(BaseModel):
    """OpenAI usage statistics."""

    prompt_tokens: int = Field(..., description="Tokens in prompt")
    completion_tokens: int = Field(..., description="Tokens in completion")
    total_tokens: int = Field(..., description="Total tokens")


class ChatCompletionResponse(BaseModel):
    """OpenAI chat completion response."""

    id: str = Field(..., description="Unique completion ID")
    object: str = Field(default="chat.completion", description="Object type")
    created: int = Field(..., description="Unix timestamp")
    model: str = Field(..., description="Model used")
    choices: List[ChatCompletionChoice] = Field(..., description="Generated choices")
    usage: ChatCompletionUsage = Field(..., description="Token usage")


# ============================================================================
# Sutra-Specific Models
# ============================================================================


class LearnRequest(BaseModel):
    """Request to learn new knowledge."""

    text: str = Field(..., description="Text to learn from")
    context: Optional[Dict[str, Any]] = Field(
        default=None, description="Additional context"
    )


class LearnResponse(BaseModel):
    """Response from learning."""

    success: bool = Field(..., description="Whether learning succeeded")
    concepts_learned: int = Field(..., description="Number of concepts learned")
    associations_created: int = Field(..., description="Number of associations created")
    message: str = Field(..., description="Human-readable message")


class QueryRequest(BaseModel):
    """Request to query knowledge."""

    query: str = Field(..., description="Question to ask")
    semantic_boost: bool = Field(
        default=True, description="Enable semantic similarity boost"
    )
    max_depth: int = Field(default=3, ge=1, le=10, description="Maximum search depth")
    max_paths: int = Field(
        default=5, ge=1, le=50, description="Maximum reasoning paths"
    )
    # NLG (optional) controls
    tone: Optional[str] = Field(default=None, description="Response tone: friendly|formal|concise|regulatory")
    moves: Optional[List[str]] = Field(default=None, description="Rhetorical moves, e.g., [define, evidence]")


class ConfidenceBreakdown(BaseModel):
    """Confidence score breakdown."""

    graph_confidence: float = Field(..., description="Graph reasoning confidence")
    semantic_confidence: float = Field(
        ..., description="Semantic similarity confidence"
    )
    final_confidence: float = Field(..., description="Final combined confidence")


class SemanticMatch(BaseModel):
    """Semantic similarity match."""

    concept: str = Field(..., description="Matched concept")
    similarity: float = Field(..., description="Similarity score")


class ReasoningPath(BaseModel):
    """A single reasoning path."""

    path: List[str] = Field(..., description="Sequence of concepts in path")
    confidence: float = Field(..., description="Path confidence score")


class QueryResponse(BaseModel):
    """Response from query."""

    answer: str = Field(..., description="Generated answer")
    confidence: float = Field(..., description="Overall confidence score")
    confidence_breakdown: ConfidenceBreakdown = Field(
        ..., description="Detailed confidence breakdown"
    )
    reasoning_paths: List[ReasoningPath] = Field(
        ..., description="Graph reasoning paths used"
    )
    semantic_support: Optional[List[SemanticMatch]] = Field(
        default=None, description="Semantic similarity matches"
    )
    explanation: str = Field(..., description="Human-readable explanation")
    timestamp: str = Field(..., description="ISO timestamp")


class MultiStrategyRequest(BaseModel):
    """Request for multi-strategy comparison."""

    query: str = Field(..., description="Question to ask")
    max_depth: int = Field(default=3, ge=1, le=10, description="Maximum search depth")
    max_paths: int = Field(
        default=5, ge=1, le=50, description="Maximum reasoning paths"
    )


class StrategyResult(BaseModel):
    """Result from a single strategy."""

    strategy: str = Field(..., description="Strategy name")
    answer: str = Field(..., description="Generated answer")
    confidence: float = Field(..., description="Confidence score")
    reasoning_paths: List[ReasoningPath] = Field(..., description="Reasoning paths")


class MultiStrategyResponse(BaseModel):
    """Response from multi-strategy comparison."""

    query: str = Field(..., description="Original query")
    strategies: List[StrategyResult] = Field(
        ..., description="Results from each strategy"
    )
    agreement_score: float = Field(
        ..., description="Agreement between strategies (0-1)"
    )
    recommended_answer: str = Field(..., description="Best recommended answer")
    explanation: str = Field(..., description="Explanation of comparison")
    timestamp: str = Field(..., description="ISO timestamp")


class AuditLogEntry(BaseModel):
    """Audit log entry."""

    timestamp: str = Field(..., description="ISO timestamp")
    operation: str = Field(..., description="Operation type (learn/query)")
    input_data: Dict[str, Any] = Field(..., description="Input data")
    output_data: Dict[str, Any] = Field(..., description="Output data")
    confidence: Optional[float] = Field(default=None, description="Confidence score")


class AuditLogResponse(BaseModel):
    """Response containing audit logs."""

    total_entries: int = Field(..., description="Total number of entries")
    entries: List[AuditLogEntry] = Field(..., description="Audit log entries")


class HealthResponse(BaseModel):
    """Health check response."""

    status: str = Field(..., description="Service status")
    version: str = Field(..., description="API version")
    uptime_seconds: float = Field(..., description="Uptime in seconds")
    total_concepts: int = Field(..., description="Total concepts in knowledge base")
    total_associations: int = Field(..., description="Total associations")


class ErrorResponse(BaseModel):
    """Error response."""

    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Error message")
    details: Optional[Dict[str, Any]] = Field(
        default=None, description="Additional error details"
    )


# ============================================================================
# Internal Models
# ============================================================================


class ConceptInfo(BaseModel):
    """Information about a concept."""

    word: str = Field(..., description="Concept word")
    strength: float = Field(..., description="Learning strength (0-10)")
    access_count: int = Field(..., description="Number of times accessed")
    associations: int = Field(..., description="Number of associations")


class StatsResponse(BaseModel):
    """Statistics about the knowledge base."""

    total_concepts: int = Field(..., description="Total concepts")
    total_associations: int = Field(..., description="Total associations")
    avg_concept_strength: float = Field(..., description="Average concept strength")
    top_concepts: List[ConceptInfo] = Field(..., description="Most accessed concepts")
