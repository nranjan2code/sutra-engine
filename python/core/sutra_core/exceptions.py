"""
Custom exceptions for the Sutra AI system.

This module defines specific exception types for better error handling
and user feedback across the Sutra AI ecosystem.
"""


class SutraError(Exception):
    """Base exception for all Sutra AI errors."""

    pass


class ConceptError(SutraError):
    """Errors related to concept operations."""

    pass


class AssociationError(SutraError):
    """Errors related to association operations."""

    pass


class LearningError(SutraError):
    """Errors related to learning operations."""

    pass


class ValidationError(SutraError):
    """Errors related to data validation."""

    pass


class StorageError(SutraError):
    """Errors related to persistence and storage."""

    pass


class ConfigurationError(SutraError):
    """Errors related to system configuration."""

    pass
