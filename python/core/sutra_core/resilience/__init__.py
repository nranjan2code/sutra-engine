"""
Resilience patterns for production deployments.

Provides:
- Circuit breakers for external service calls
- Retry mechanisms with exponential backoff
- Graceful degradation
"""

from .circuit_breaker import CircuitBreaker, CircuitBreakerState

__all__ = ["CircuitBreaker", "CircuitBreakerState"]
