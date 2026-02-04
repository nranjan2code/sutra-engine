"""Sutra AI API module.

OpenAI-compatible and custom endpoints for Sutra AI.
"""

from .app import app, create_app

__all__ = ["create_app", "app"]
