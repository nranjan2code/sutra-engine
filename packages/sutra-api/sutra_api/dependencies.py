"""
Dependency injection for FastAPI endpoints.

Provides shared instances and dependency functions using FastAPI app.state
instead of global variables for better testability and multi-worker support.
"""

import logging
import time
from typing import Optional

from fastapi import FastAPI, Request
from sutra_core.graph.concepts import AssociationType
from sutra_core.reasoning import ReasoningEngine
from sutra_hybrid import SutraAI

from .config import settings

logger = logging.getLogger(__name__)

# Track service start time
_start_time: float = time.time()


def init_dependencies(app: FastAPI) -> None:
    """Initialize dependencies with embeddings DISABLED for fast dev."""
    """
    Initialize dependencies and store them in app.state.
    Called during lifespan startup.

    Args:
        app: FastAPI application instance
    """
    logger.info("Initializing dependencies...")

    # Create SutraAI instance (includes core engine + semantic embeddings)
    # DISABLED for fast development (embedding models take forever to load)
    app.state.ai_instance = SutraAI(
        storage_path=settings.storage_path,
        enable_semantic=False,  # Hardcoded False for dev speed
    )
    
    # Load existing knowledge from disk
    try:
        app.state.ai_instance.load()
        stats = app.state.ai_instance.get_stats()
        logger.info(f"Loaded {stats['total_concepts']} concepts, {stats['total_associations']} associations")
    except Exception as e:
        logger.info(f"No existing knowledge to load: {e}")

    # The reasoning engine is already initialized inside SutraAI
    # Access it via ai.engine property
    logger.info("Dependencies initialized successfully")


def shutdown_dependencies(app: FastAPI) -> None:
    """
    Clean up dependencies during shutdown.
    Called during lifespan shutdown.

    Args:
        app: FastAPI application instance
    """
    if settings.auto_save and hasattr(app.state, "ai_instance"):
        logger.info("Saving knowledge before shutdown...")
        app.state.ai_instance.save()
        logger.info("Knowledge saved")

    # Clean up references
    if hasattr(app.state, "ai_instance"):
        delattr(app.state, "ai_instance")
    if hasattr(app.state, "reasoner_instance"):
        delattr(app.state, "reasoner_instance")


def get_ai(request: Request) -> SutraAI:
    """
    Dependency to get SutraAI instance from request state.

    Args:
        request: FastAPI request containing app.state

    Returns:
        SutraAI instance
    """
    return request.app.state.ai_instance


def get_uptime() -> float:
    """
    Get service uptime in seconds.

    Returns:
        Uptime in seconds
    """
    return time.time() - _start_time


def get_reasoner(request: Request) -> ReasoningEngine:
    """
    Dependency to get ReasoningEngine instance from request state.

    The engine is accessed from SutraAI's internal engine.

    Args:
        request: FastAPI request containing app.state

    Returns:
        ReasoningEngine instance
    """
    ai = request.app.state.ai_instance
    return ai.engine
