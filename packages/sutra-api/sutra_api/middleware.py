"""
Rate limiting middleware for Sutra API.

Implements simple in-memory rate limiting to prevent API abuse.
For production, consider using Redis-backed rate limiting (slowapi + redis).
"""

import time
from collections import defaultdict
from typing import Callable, Dict, Tuple

from fastapi import HTTPException, Request, Response, status
from starlette.middleware.base import BaseHTTPMiddleware


class RateLimitMiddleware(BaseHTTPMiddleware):
    """
    Simple in-memory rate limiting middleware.

    Tracks requests per IP address and endpoint combination.
    Uses sliding window algorithm for accurate rate limiting.

    Note: This is a basic implementation for single-server deployments.
    For multi-server production, use Redis-backed rate limiting (slowapi).
    """

    def __init__(
        self,
        app,
        default_limit: int = 60,
        window_seconds: int = 60,
        endpoint_limits: Dict[str, int] = None,
    ):
        """
        Initialize rate limiting middleware.

        Args:
            app: FastAPI application
            default_limit: Default requests per window (default: 60/min)
            window_seconds: Time window in seconds (default: 60)
            endpoint_limits: Per-endpoint limits override
                Example: {"/learn": 30, "/reason": 20}
        """
        super().__init__(app)
        self.default_limit = default_limit
        self.window_seconds = window_seconds
        self.endpoint_limits = endpoint_limits or {}

        # Track requests: {(ip, endpoint): [(timestamp, ...)]}
        self.request_log: Dict[Tuple[str, str], list] = defaultdict(list)

        # Cleanup old entries periodically
        self.last_cleanup = time.time()
        self.cleanup_interval = 300  # 5 minutes

    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request with rate limiting."""

        # Skip rate limiting for health check and root endpoints
        if request.url.path in ["/health", "/", "/docs", "/openapi.json"]:
            return await call_next(request)

        # Get client IP
        client_ip = self._get_client_ip(request)
        endpoint = request.url.path

        # Get rate limit for this endpoint
        limit = self.endpoint_limits.get(endpoint, self.default_limit)

        # Check rate limit
        if not self._is_allowed(client_ip, endpoint, limit):
            raise HTTPException(
                status_code=status.HTTP_429_TOO_MANY_REQUESTS,
                detail={
                    "error": "RateLimitExceeded",
                    "message": f"Rate limit exceeded: {limit} requests per {self.window_seconds} seconds",
                    "retry_after": self.window_seconds,
                },
                headers={"Retry-After": str(self.window_seconds)},
            )

        # Record this request
        self._record_request(client_ip, endpoint)

        # Periodic cleanup
        self._maybe_cleanup()

        return await call_next(request)

    def _get_client_ip(self, request: Request) -> str:
        """Extract client IP from request."""
        # Check for proxy headers first
        forwarded = request.headers.get("X-Forwarded-For")
        if forwarded:
            return forwarded.split(",")[0].strip()

        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip

        # Fallback to direct client
        if request.client:
            return request.client.host

        return "unknown"

    def _is_allowed(self, client_ip: str, endpoint: str, limit: int) -> bool:
        """Check if request is within rate limit."""
        key = (client_ip, endpoint)
        now = time.time()
        window_start = now - self.window_seconds

        # Get requests within current window
        requests = self.request_log.get(key, [])
        recent_requests = [ts for ts in requests if ts > window_start]

        return len(recent_requests) < limit

    def _record_request(self, client_ip: str, endpoint: str) -> None:
        """Record a request timestamp."""
        key = (client_ip, endpoint)
        now = time.time()
        window_start = now - self.window_seconds

        # Clean old timestamps and add new one
        requests = self.request_log.get(key, [])
        requests = [ts for ts in requests if ts > window_start]
        requests.append(now)
        self.request_log[key] = requests

    def _maybe_cleanup(self) -> None:
        """Periodically cleanup old request logs to prevent memory bloat."""
        now = time.time()

        if now - self.last_cleanup < self.cleanup_interval:
            return

        window_start = now - self.window_seconds

        # Remove expired entries
        keys_to_delete = []
        for key, timestamps in self.request_log.items():
            # Keep only recent timestamps
            recent = [ts for ts in timestamps if ts > window_start]
            if recent:
                self.request_log[key] = recent
            else:
                keys_to_delete.append(key)

        # Delete empty entries
        for key in keys_to_delete:
            del self.request_log[key]

        self.last_cleanup = now


def create_rate_limit_middleware(
    default_limit: int = 60,
    learn_limit: int = 30,
    reason_limit: int = 20,
    search_limit: int = 100,
) -> RateLimitMiddleware:
    """
    Create rate limit middleware with sensible defaults.

    Args:
        default_limit: Default requests per minute
        learn_limit: Learning endpoint limit
        reason_limit: Reasoning endpoint limit
        search_limit: Search endpoint limit

    Returns:
        Configured RateLimitMiddleware
    """
    return RateLimitMiddleware(
        app=None,  # Will be set by add_middleware
        default_limit=default_limit,
        window_seconds=60,
        endpoint_limits={
            "/learn": learn_limit,
            "/learn/batch": learn_limit // 2,  # More restrictive for batch
            "/reason": reason_limit,
            "/search": search_limit,
        },
    )
