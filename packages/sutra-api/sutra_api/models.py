"""
Pydantic models for API request and response schemas.

Defines all data models used for API validation and documentation.
"""

from enum import Enum
from typing import Dict, List, Optional

from pydantic import BaseModel, Field


class AssociationType(str, Enum):
    """Types of associations between concepts."""

    SEMANTIC = "semantic"
    CAUSAL = "causal"
    TEMPORAL = "temporal"
    HIERARCHICAL = "hierarchical"
    COMPOSITIONAL = "compositional"


class LearnRequest(BaseModel):
    """Request model for learning new knowledge."""

    content: str = Field(..., description="Knowledge content to learn")
    source: Optional[str] = Field(None, description="Source of the knowledge")
    metadata: Optional[Dict[str, str]] = Field(None, description="Additional metadata")


class LearnResponse(BaseModel):
    """Response model for learning operations."""

    concept_id: str = Field(..., description="ID of the created/updated concept")
    message: str = Field(..., description="Success message")
    concepts_created: int = Field(..., description="Number of concepts created")
    associations_created: int = Field(..., description="Number of associations created")


class BatchLearnRequest(BaseModel):
    """Request model for batch learning."""

    items: List[LearnRequest] = Field(
        ..., description="List of knowledge items to learn"
    )


class BatchLearnResponse(BaseModel):
    """Response model for batch learning operations."""

    concept_ids: List[str] = Field(..., description="IDs of all created concepts")
    total_concepts: int = Field(..., description="Total concepts created")
    total_associations: int = Field(..., description="Total associations created")
    message: str = Field(..., description="Success message")


class ReasonRequest(BaseModel):
    """Request model for reasoning queries."""

    query: str = Field(..., description="Question or reasoning query")
    max_steps: int = Field(
        default=3, ge=1, le=10, description="Maximum reasoning depth"
    )
    num_paths: int = Field(
        default=3, ge=1, le=10, description="Number of reasoning paths to explore"
    )
    threshold: float = Field(
        default=0.3, ge=0.0, le=1.0, description="Minimum confidence threshold"
    )


class ReasoningPath(BaseModel):
    """A single reasoning path through the knowledge graph."""

    concepts: List[str] = Field(..., description="Concept IDs in the path")
    confidence: float = Field(..., description="Path confidence score")
    explanation: str = Field(..., description="Human-readable explanation of the path")


class ReasonResponse(BaseModel):
    """Response model for reasoning operations."""

    query: str = Field(..., description="Original query")
    answer: str = Field(..., description="Reasoning result/answer")
    confidence: float = Field(..., description="Overall confidence score")
    paths: List[ReasoningPath] = Field(..., description="Reasoning paths explored")
    concepts_accessed: int = Field(..., description="Number of concepts accessed")


class SemanticSearchRequest(BaseModel):
    """Request model for semantic similarity search."""

    query: str = Field(..., description="Search query")
    top_k: int = Field(
        default=5, ge=1, le=100, description="Number of results to return"
    )
    threshold: float = Field(
        default=0.5, ge=0.0, le=1.0, description="Minimum similarity threshold"
    )


class SearchResult(BaseModel):
    """A single search result."""

    concept_id: str = Field(..., description="Concept ID")
    content: str = Field(..., description="Concept content")
    similarity: float = Field(..., description="Similarity score")
    strength: float = Field(..., description="Concept strength")


class SemanticSearchResponse(BaseModel):
    """Response model for semantic search operations."""

    query: str = Field(..., description="Original search query")
    results: List[SearchResult] = Field(..., description="Search results")
    total_found: int = Field(..., description="Total number of results found")


class ConceptDetail(BaseModel):
    """Detailed information about a concept."""

    id: str = Field(..., description="Concept ID")
    content: str = Field(..., description="Concept content")
    strength: float = Field(..., description="Concept strength (1.0-10.0)")
    access_count: int = Field(..., description="Number of times accessed")
    created_at: Optional[str] = Field(None, description="Creation timestamp")
    source: Optional[str] = Field(None, description="Source of knowledge")
    associations: List[str] = Field(..., description="IDs of associated concepts")


class SystemStats(BaseModel):
    """System statistics and metrics."""

    total_concepts: int = Field(..., description="Total number of concepts")
    total_associations: int = Field(..., description="Total number of associations")
    total_embeddings: int = Field(..., description="Total number of embeddings")
    embedding_provider: str = Field(..., description="Active embedding provider")
    embedding_dimension: int = Field(..., description="Embedding dimensionality")
    average_strength: float = Field(..., description="Average concept strength")
    memory_usage_mb: Optional[float] = Field(
        None, description="Approximate memory usage in MB"
    )


class HealthResponse(BaseModel):
    """Health check response."""

    status: str = Field(..., description="Service status")
    version: str = Field(..., description="API version")
    uptime_seconds: float = Field(..., description="Service uptime in seconds")
    concepts_loaded: int = Field(..., description="Number of concepts loaded")


class ErrorResponse(BaseModel):
    """Error response model."""

    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Error message")
    detail: Optional[str] = Field(None, description="Detailed error information")
