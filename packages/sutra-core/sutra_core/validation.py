"""
Input validation for Sutra AI system.

Provides validation for all user inputs to ensure data integrity,
prevent DOS attacks, and maintain system stability.
"""

from typing import Any, Dict, Optional

from .exceptions import ValidationError


class Validator:
    """Centralized validation for all Sutra AI inputs."""

    # Limits
    MAX_CONTENT_LENGTH = 10_000  # 10KB per concept
    MAX_QUERY_LENGTH = 1_000  # 1KB per query
    MAX_DEPTH = 10  # Maximum reasoning depth
    MAX_NUM_PATHS = 20  # Maximum reasoning paths
    MAX_BATCH_SIZE = 1_000  # Maximum batch learning size
    MIN_CONFIDENCE = 0.0
    MAX_CONFIDENCE = 1.0
    MIN_STRENGTH = 0.1
    MAX_STRENGTH = 10.0

    @classmethod
    def validate_content(cls, content: str, field_name: str = "content") -> str:
        """
        Validate concept content.

        Args:
            content: Content to validate
            field_name: Name of field for error messages

        Returns:
            Validated content (stripped)

        Raises:
            ValidationError: If content is invalid
        """
        if not isinstance(content, str):
            raise ValidationError(f"{field_name} must be a string, got {type(content)}")

        if not content or not content.strip():
            raise ValidationError(f"{field_name} cannot be empty")

        content = content.strip()

        if len(content) > cls.MAX_CONTENT_LENGTH:
            raise ValidationError(
                f"{field_name} exceeds maximum length of {cls.MAX_CONTENT_LENGTH} "
                f"characters (got {len(content)})"
            )

        return content

    @classmethod
    def validate_query(cls, query: str) -> str:
        """
        Validate query string.

        Args:
            query: Query to validate

        Returns:
            Validated query (stripped)

        Raises:
            ValidationError: If query is invalid
        """
        if not isinstance(query, str):
            raise ValidationError(f"Query must be a string, got {type(query)}")

        if not query or not query.strip():
            raise ValidationError("Query cannot be empty")

        query = query.strip()

        if len(query) > cls.MAX_QUERY_LENGTH:
            raise ValidationError(
                f"Query exceeds maximum length of {cls.MAX_QUERY_LENGTH} "
                f"characters (got {len(query)})"
            )

        return query

    @classmethod
    def validate_confidence(cls, confidence: float, field_name: str = "confidence") -> float:
        """
        Validate and clamp confidence score.

        Args:
            confidence: Confidence value to validate
            field_name: Name of field for error messages

        Returns:
            Clamped confidence value [0.0, 1.0]

        Raises:
            ValidationError: If confidence is not a number
        """
        if not isinstance(confidence, (int, float)):
            raise ValidationError(
                f"{field_name} must be a number, got {type(confidence)}"
            )

        # Clamp to valid range
        return max(cls.MIN_CONFIDENCE, min(cls.MAX_CONFIDENCE, float(confidence)))

    @classmethod
    def validate_strength(cls, strength: float, field_name: str = "strength") -> float:
        """
        Validate and clamp strength value.

        Args:
            strength: Strength value to validate
            field_name: Name of field for error messages

        Returns:
            Clamped strength value [0.1, 10.0]

        Raises:
            ValidationError: If strength is not a number
        """
        if not isinstance(strength, (int, float)):
            raise ValidationError(f"{field_name} must be a number, got {type(strength)}")

        # Clamp to valid range
        return max(cls.MIN_STRENGTH, min(cls.MAX_STRENGTH, float(strength)))

    @classmethod
    def validate_depth(cls, depth: int, field_name: str = "depth") -> int:
        """
        Validate depth parameter.

        Args:
            depth: Depth value to validate
            field_name: Name of field for error messages

        Returns:
            Validated depth value

        Raises:
            ValidationError: If depth is invalid
        """
        if not isinstance(depth, int):
            raise ValidationError(f"{field_name} must be an integer, got {type(depth)}")

        if depth < 1:
            raise ValidationError(f"{field_name} must be at least 1, got {depth}")

        if depth > cls.MAX_DEPTH:
            raise ValidationError(
                f"{field_name} exceeds maximum of {cls.MAX_DEPTH}, got {depth}"
            )

        return depth

    @classmethod
    def validate_num_paths(cls, num_paths: int, field_name: str = "num_paths") -> int:
        """
        Validate number of paths parameter.

        Args:
            num_paths: Number of paths to validate
            field_name: Name of field for error messages

        Returns:
            Validated num_paths value

        Raises:
            ValidationError: If num_paths is invalid
        """
        if not isinstance(num_paths, int):
            raise ValidationError(
                f"{field_name} must be an integer, got {type(num_paths)}"
            )

        if num_paths < 1:
            raise ValidationError(f"{field_name} must be at least 1, got {num_paths}")

        if num_paths > cls.MAX_NUM_PATHS:
            raise ValidationError(
                f"{field_name} exceeds maximum of {cls.MAX_NUM_PATHS}, got {num_paths}"
            )

        return num_paths

    @classmethod
    def validate_filepath(cls, filepath: str) -> str:
        """
        Validate and sanitize filepath.

        Args:
            filepath: Filepath to validate

        Returns:
            Sanitized filepath

        Raises:
            ValidationError: If filepath is invalid
        """
        import os
        from pathlib import Path

        if not isinstance(filepath, str):
            raise ValidationError(f"Filepath must be a string, got {type(filepath)}")

        if not filepath or not filepath.strip():
            raise ValidationError("Filepath cannot be empty")

        filepath = filepath.strip()

        # Prevent path traversal attacks
        filepath = os.path.normpath(filepath)
        if ".." in filepath:
            raise ValidationError("Filepath cannot contain parent directory references")

        # Check if path is absolute or relative to safe location
        path = Path(filepath)
        if path.is_absolute():
            # Allow absolute paths but validate they're not system directories
            forbidden_dirs = ["/etc", "/sys", "/proc", "/dev", "/bin", "/sbin"]
            for forbidden in forbidden_dirs:
                if str(path).startswith(forbidden):
                    raise ValidationError(f"Cannot write to system directory: {forbidden}")

        return filepath

    @classmethod
    def validate_concept_id(cls, concept_id: str) -> str:
        """
        Validate concept ID format.

        Args:
            concept_id: Concept ID to validate

        Returns:
            Validated concept ID

        Raises:
            ValidationError: If concept ID is invalid
        """
        if not isinstance(concept_id, str):
            raise ValidationError(f"Concept ID must be a string, got {type(concept_id)}")

        if not concept_id or not concept_id.strip():
            raise ValidationError("Concept ID cannot be empty")

        # Check format (MD5 hash prefix)
        import re

        if not re.match(r"^[a-f0-9]{16}$", concept_id):
            raise ValidationError(
                f"Invalid concept ID format: {concept_id} "
                "(expected 16-character hex string)"
            )

        return concept_id

    @classmethod
    def validate_batch_size(cls, size: int, field_name: str = "batch_size") -> int:
        """
        Validate batch size parameter.

        Args:
            size: Batch size to validate
            field_name: Name of field for error messages

        Returns:
            Validated batch size

        Raises:
            ValidationError: If batch size is invalid
        """
        if not isinstance(size, int):
            raise ValidationError(f"{field_name} must be an integer, got {type(size)}")

        if size < 1:
            raise ValidationError(f"{field_name} must be at least 1, got {size}")

        if size > cls.MAX_BATCH_SIZE:
            raise ValidationError(
                f"{field_name} exceeds maximum of {cls.MAX_BATCH_SIZE}, got {size}"
            )

        return size

    @classmethod
    def validate_metadata(cls, metadata: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Validate metadata dictionary.

        Args:
            metadata: Metadata to validate

        Returns:
            Validated metadata (empty dict if None)

        Raises:
            ValidationError: If metadata is invalid
        """
        if metadata is None:
            return {}

        if not isinstance(metadata, dict):
            raise ValidationError(f"Metadata must be a dict, got {type(metadata)}")

        # Validate all keys are strings
        for key in metadata.keys():
            if not isinstance(key, str):
                raise ValidationError(
                    f"Metadata keys must be strings, got {type(key)} for key {key}"
                )

        return metadata
