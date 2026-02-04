"""
Circuit Breaker Pattern Implementation

Prevents cascading failures when external services (e.g., embedding service) fail.

States:
- CLOSED: Normal operation, requests pass through
- OPEN: Service failing, requests fail fast
- HALF_OPEN: Testing if service recovered
"""

import logging
import time
from enum import Enum
from typing import Callable, Optional, TypeVar, Generic
from dataclasses import dataclass
from threading import Lock

logger = logging.getLogger(__name__)

T = TypeVar('T')


class CircuitBreakerState(Enum):
    """Circuit breaker states"""
    CLOSED = "closed"  # Normal operation
    OPEN = "open"  # Failing, reject requests
    HALF_OPEN = "half_open"  # Testing recovery


@dataclass
class CircuitBreakerConfig:
    """Circuit breaker configuration"""
    failure_threshold: int = 5  # Failures before opening
    timeout_seconds: int = 60  # Time before attempting recovery
    half_open_max_requests: int = 3  # Test requests in half-open state


class CircuitBreakerError(Exception):
    """Raised when circuit breaker is open"""
    pass


class CircuitBreaker(Generic[T]):
    """
    Circuit breaker for protecting against cascading failures.

    Usage:
        >>> breaker = CircuitBreaker(name="embedding_service")
        >>>
        >>> def call_embedding_service():
        ...     # Network call that might fail
        ...     return embedding_client.generate(text)
        >>>
        >>> try:
        ...     result = breaker.call(call_embedding_service)
        ... except CircuitBreakerError:
        ...     # Circuit is open, service is failing
        ...     result = fallback_embedding()
    """

    def __init__(
        self,
        name: str,
        config: Optional[CircuitBreakerConfig] = None,
    ):
        """
        Initialize circuit breaker.

        Args:
            name: Circuit breaker identifier (for logging)
            config: Configuration (uses defaults if None)
        """
        self.name = name
        self.config = config or CircuitBreakerConfig()

        self._state = CircuitBreakerState.CLOSED
        self._failure_count = 0
        self._last_failure_time: Optional[float] = None
        self._half_open_requests = 0
        self._lock = Lock()

        logger.info(
            f"Circuit breaker '{name}' initialized: "
            f"threshold={self.config.failure_threshold}, "
            f"timeout={self.config.timeout_seconds}s"
        )

    @property
    def state(self) -> CircuitBreakerState:
        """Get current circuit breaker state (thread-safe)."""
        with self._lock:
            return self._state

    def call(self, func: Callable[[], T]) -> T:
        """
        Execute function with circuit breaker protection.

        Args:
            func: Function to execute (no arguments)

        Returns:
            Function return value

        Raises:
            CircuitBreakerError: If circuit is open
            Exception: If function raises (circuit may open)
        """
        with self._lock:
            current_state = self._state

            # Check if we should transition from OPEN -> HALF_OPEN
            if current_state == CircuitBreakerState.OPEN:
                if self._should_attempt_reset():
                    logger.info(f"Circuit breaker '{self.name}': OPEN -> HALF_OPEN")
                    self._state = CircuitBreakerState.HALF_OPEN
                    self._half_open_requests = 0
                    current_state = CircuitBreakerState.HALF_OPEN
                else:
                    # Circuit is open, fail fast
                    raise CircuitBreakerError(
                        f"Circuit breaker '{self.name}' is OPEN "
                        f"(failed {self._failure_count} times)"
                    )

            # HALF_OPEN: limit test requests
            if current_state == CircuitBreakerState.HALF_OPEN:
                if self._half_open_requests >= self.config.half_open_max_requests:
                    raise CircuitBreakerError(
                        f"Circuit breaker '{self.name}' is HALF_OPEN "
                        f"(max test requests reached)"
                    )
                self._half_open_requests += 1

        # Execute function (outside lock to allow parallelism)
        try:
            result = func()
            self._on_success()
            return result

        except Exception as e:
            self._on_failure(e)
            raise

    def _on_success(self) -> None:
        """Handle successful request."""
        with self._lock:
            if self._state == CircuitBreakerState.HALF_OPEN:
                # Recovery successful
                logger.info(
                    f"Circuit breaker '{self.name}': HALF_OPEN -> CLOSED (recovered)"
                )
                self._state = CircuitBreakerState.CLOSED
                self._failure_count = 0
                self._last_failure_time = None

            elif self._state == CircuitBreakerState.CLOSED:
                # Reset failure count on success
                if self._failure_count > 0:
                    logger.debug(
                        f"Circuit breaker '{self.name}': resetting failure count "
                        f"({self._failure_count} -> 0)"
                    )
                    self._failure_count = 0

    def _on_failure(self, exception: Exception) -> None:
        """Handle failed request."""
        with self._lock:
            self._failure_count += 1
            self._last_failure_time = time.time()

            if self._state == CircuitBreakerState.HALF_OPEN:
                # Recovery failed, reopen circuit
                logger.warning(
                    f"Circuit breaker '{self.name}': HALF_OPEN -> OPEN "
                    f"(recovery failed: {exception})"
                )
                self._state = CircuitBreakerState.OPEN

            elif self._state == CircuitBreakerState.CLOSED:
                if self._failure_count >= self.config.failure_threshold:
                    # Too many failures, open circuit
                    logger.error(
                        f"Circuit breaker '{self.name}': CLOSED -> OPEN "
                        f"(threshold reached: {self._failure_count} failures)"
                    )
                    self._state = CircuitBreakerState.OPEN
                else:
                    logger.warning(
                        f"Circuit breaker '{self.name}': failure {self._failure_count}/"
                        f"{self.config.failure_threshold} ({exception})"
                    )

    def _should_attempt_reset(self) -> bool:
        """Check if enough time has passed to attempt recovery."""
        if self._last_failure_time is None:
            return True

        elapsed = time.time() - self._last_failure_time
        return elapsed >= self.config.timeout_seconds

    def reset(self) -> None:
        """Manually reset circuit breaker to CLOSED state."""
        with self._lock:
            logger.info(f"Circuit breaker '{self.name}': manual reset to CLOSED")
            self._state = CircuitBreakerState.CLOSED
            self._failure_count = 0
            self._last_failure_time = None

    def get_stats(self) -> dict:
        """Get circuit breaker statistics."""
        with self._lock:
            return {
                "name": self.name,
                "state": self._state.value,
                "failure_count": self._failure_count,
                "last_failure_time": self._last_failure_time,
                "config": {
                    "failure_threshold": self.config.failure_threshold,
                    "timeout_seconds": self.config.timeout_seconds,
                    "half_open_max_requests": self.config.half_open_max_requests,
                },
            }
