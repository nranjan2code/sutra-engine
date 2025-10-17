"""
Minimal exceptions for Sutra API.

These replace heavy imports from sutra-core for thin client architecture.
"""


class SutraError(Exception):
    """Base exception for Sutra AI errors."""
    pass


class StorageError(SutraError):
    """Storage-related errors."""
    pass


class ValidationError(SutraError):
    """Request validation errors."""
    pass
