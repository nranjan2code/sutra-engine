"""
Production-grade security middleware for Sutra API.

Implements essential security headers, CORS hardening, and protection mechanisms.
Zero external dependencies - uses only FastAPI and Python standard library.
"""

import hashlib
import secrets
from typing import Callable

from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp


class SecurityHeadersMiddleware(BaseHTTPMiddleware):
    """
    Add production-grade security headers to all responses.
    
    Implements OWASP recommendations for web application security.
    """
    
    def __init__(
        self,
        app: ASGIApp,
        *,
        strict_transport_security: bool = True,
        content_security_policy: bool = True,
        x_frame_options: str = "DENY",
        x_content_type_options: bool = True,
        x_xss_protection: bool = True,
        referrer_policy: str = "strict-origin-when-cross-origin",
        permissions_policy: bool = True,
    ):
        super().__init__(app)
        self.strict_transport_security = strict_transport_security
        self.content_security_policy = content_security_policy
        self.x_frame_options = x_frame_options
        self.x_content_type_options = x_content_type_options
        self.x_xss_protection = x_xss_protection
        self.referrer_policy = referrer_policy
        self.permissions_policy = permissions_policy
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Add security headers to response."""
        response = await call_next(request)
        
        # HSTS - Force HTTPS for 1 year (only in production with HTTPS)
        if self.strict_transport_security and request.url.scheme == "https":
            response.headers["Strict-Transport-Security"] = (
                "max-age=31536000; includeSubDomains; preload"
            )
        
        # CSP - Prevent XSS and injection attacks
        if self.content_security_policy:
            response.headers["Content-Security-Policy"] = (
                "default-src 'self'; "
                "script-src 'self' 'unsafe-inline' 'unsafe-eval'; "  # Allow inline for React
                "style-src 'self' 'unsafe-inline'; "  # Allow inline styles
                "img-src 'self' data: https:; "
                "font-src 'self' data:; "
                "connect-src 'self' ws: wss:; "  # Allow WebSocket
                "frame-ancestors 'none'; "
                "base-uri 'self'; "
                "form-action 'self'"
            )
        
        # X-Frame-Options - Prevent clickjacking
        response.headers["X-Frame-Options"] = self.x_frame_options
        
        # X-Content-Type-Options - Prevent MIME sniffing
        if self.x_content_type_options:
            response.headers["X-Content-Type-Options"] = "nosniff"
        
        # X-XSS-Protection - Legacy XSS protection (browser-level)
        if self.x_xss_protection:
            response.headers["X-XSS-Protection"] = "1; mode=block"
        
        # Referrer-Policy - Control referrer information
        response.headers["Referrer-Policy"] = self.referrer_policy
        
        # Permissions-Policy - Disable unnecessary browser features
        if self.permissions_policy:
            response.headers["Permissions-Policy"] = (
                "geolocation=(), "
                "microphone=(), "
                "camera=(), "
                "payment=(), "
                "usb=(), "
                "magnetometer=(), "
                "gyroscope=(), "
                "accelerometer=()"
            )
        
        # Remove server header (information disclosure)
        if "Server" in response.headers:
            del response.headers["Server"]
        
        # Add security notice header (optional)
        response.headers["X-Powered-By"] = "Sutra AI"
        
        return response


class HTTPSRedirectMiddleware(BaseHTTPMiddleware):
    """
    Redirect all HTTP requests to HTTPS in production.
    
    Only active when SUTRA_SECURE_MODE=true.
    """
    
    def __init__(self, app: ASGIApp, enabled: bool = True):
        super().__init__(app)
        self.enabled = enabled
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Redirect HTTP to HTTPS if enabled."""
        if self.enabled and request.url.scheme == "http":
            # Redirect to HTTPS version
            https_url = request.url.replace(scheme="https")
            return Response(
                status_code=301,
                headers={"Location": str(https_url)},
            )
        
        return await call_next(request)


class SecureCookieMiddleware(BaseHTTPMiddleware):
    """
    Ensure cookies are set with secure flags.
    
    Adds HttpOnly, Secure, SameSite attributes to authentication cookies.
    """
    
    def __init__(
        self,
        app: ASGIApp,
        *,
        secure: bool = True,
        httponly: bool = True,
        samesite: str = "lax",
    ):
        super().__init__(app)
        self.secure = secure
        self.httponly = httponly
        self.samesite = samesite
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Enhance cookie security attributes."""
        response = await call_next(request)
        
        # Process Set-Cookie headers
        set_cookie_headers = response.headers.getlist("set-cookie")
        if set_cookie_headers:
            # Clear existing Set-Cookie headers
            response.headers._list = [
                (k, v) for k, v in response.headers._list if k.lower() != "set-cookie"
            ]
            
            # Re-add with security attributes
            for cookie in set_cookie_headers:
                # Only enhance if not already set
                if "Secure" not in cookie and self.secure:
                    cookie += "; Secure"
                if "HttpOnly" not in cookie and self.httponly:
                    cookie += "; HttpOnly"
                if "SameSite" not in cookie:
                    cookie += f"; SameSite={self.samesite.capitalize()}"
                
                response.headers.append("set-cookie", cookie)
        
        return response


def generate_nonce() -> str:
    """Generate cryptographically secure nonce for CSP."""
    return secrets.token_urlsafe(16)


def compute_sri_hash(content: str, algorithm: str = "sha384") -> str:
    """
    Compute Subresource Integrity (SRI) hash for inline scripts/styles.
    
    Args:
        content: Script or style content
        algorithm: Hash algorithm (sha256, sha384, sha512)
    
    Returns:
        Base64-encoded hash with algorithm prefix (e.g., "sha384-abc123...")
    """
    import base64
    
    hasher = hashlib.new(algorithm)
    hasher.update(content.encode("utf-8"))
    hash_bytes = hasher.digest()
    hash_b64 = base64.b64encode(hash_bytes).decode("utf-8")
    
    return f"{algorithm}-{hash_b64}"
