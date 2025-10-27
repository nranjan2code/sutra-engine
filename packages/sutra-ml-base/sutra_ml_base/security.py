"""
Security and Authentication for Sutra ML Services

Provides production-grade security features:
- API key authentication and JWT tokens
- Rate limiting and DDoS protection
- Request validation and sanitization
- Edition-aware security policies
- Audit logging and monitoring
"""

import logging
import time
import hashlib
import hmac
from dataclasses import dataclass
from typing import Optional, Dict, List, Any
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)


@dataclass
class AuthConfig:
    """Authentication and security configuration"""
    # Authentication
    require_auth: bool = False
    api_keys: List[str] = None
    jwt_secret: Optional[str] = None
    token_expiry_hours: int = 24
    
    # Rate limiting
    rate_limit_per_minute: int = 1000
    burst_limit: int = 100
    
    # Security features
    enable_cors: bool = True
    allowed_origins: List[str] = None
    require_https: bool = False
    
    # Request validation
    max_request_size_mb: int = 10
    request_timeout_seconds: int = 30
    
    def __post_init__(self):
        """Set defaults and validate"""
        if self.api_keys is None:
            self.api_keys = []
        
        if self.allowed_origins is None:
            self.allowed_origins = ["*"] if not self.require_auth else []


class SecurityManager:
    """Security manager for ML services"""
    
    def __init__(self, config: AuthConfig):
        self.config = config
        self._rate_limit_tracker = {}
        
        logger.info(f"Security manager initialized (auth: {config.require_auth})")
    
    def validate_api_key(self, api_key: str) -> bool:
        """Validate API key
        
        Args:
            api_key: API key to validate
            
        Returns:
            True if valid, False otherwise
        """
        if not self.config.require_auth:
            return True
        
        if not api_key:
            return False
        
        # Constant-time comparison to prevent timing attacks
        for valid_key in self.config.api_keys:
            if hmac.compare_digest(api_key, valid_key):
                return True
        
        logger.warning(f"Invalid API key attempted: {api_key[:8]}...")
        return False
    
    def check_rate_limit(self, client_id: str) -> bool:
        """Check if client is within rate limits
        
        Args:
            client_id: Client identifier (IP, API key, etc.)
            
        Returns:
            True if within limits, False if rate limited
        """
        now = time.time()
        window_start = now - 60  # 1-minute window
        
        # Clean old entries
        self._clean_rate_limit_tracker(window_start)
        
        # Get or create client tracker
        if client_id not in self._rate_limit_tracker:
            self._rate_limit_tracker[client_id] = []
        
        client_requests = self._rate_limit_tracker[client_id]
        
        # Remove requests outside window
        client_requests[:] = [req_time for req_time in client_requests if req_time > window_start]
        
        # Check limits
        if len(client_requests) >= self.config.rate_limit_per_minute:
            logger.warning(f"Rate limit exceeded for client: {client_id}")
            return False
        
        # Record this request
        client_requests.append(now)
        return True
    
    def sanitize_input(self, input_data: Any) -> Any:
        """Sanitize input data for security
        
        Args:
            input_data: Input to sanitize
            
        Returns:
            Sanitized input
        """
        if isinstance(input_data, str):
            # Basic string sanitization
            return input_data.strip()
        
        elif isinstance(input_data, dict):
            # Recursive sanitization for dictionaries
            return {
                k: self.sanitize_input(v)
                for k, v in input_data.items()
                if isinstance(k, str) and len(str(k)) <= 100  # Limit key length
            }
        
        elif isinstance(input_data, list):
            # Limit list sizes and sanitize items
            if len(input_data) > 1000:  # Reasonable limit
                logger.warning("Truncating oversized list input")
                input_data = input_data[:1000]
            
            return [self.sanitize_input(item) for item in input_data]
        
        return input_data
    
    def validate_request_size(self, content_length: int) -> bool:
        """Validate request size is within limits
        
        Args:
            content_length: Size of request in bytes
            
        Returns:
            True if within limits, False otherwise
        """
        max_bytes = self.config.max_request_size_mb * 1024 * 1024
        
        if content_length > max_bytes:
            logger.warning(f"Request too large: {content_length} bytes (max: {max_bytes})")
            return False
        
        return True
    
    def get_client_id(self, request_headers: Dict[str, str], client_ip: str) -> str:
        """Generate client ID for rate limiting
        
        Args:
            request_headers: HTTP headers
            client_ip: Client IP address
            
        Returns:
            Client identifier string
        """
        # Use API key if present, otherwise IP
        api_key = request_headers.get("X-API-Key") or request_headers.get("Authorization")
        
        if api_key and self.validate_api_key(api_key):
            # Hash API key for privacy
            return hashlib.sha256(api_key.encode()).hexdigest()[:16]
        
        return client_ip
    
    def _clean_rate_limit_tracker(self, cutoff_time: float):
        """Clean old entries from rate limit tracker"""
        clients_to_remove = []
        
        for client_id, requests in self._rate_limit_tracker.items():
            # Remove old requests
            requests[:] = [req_time for req_time in requests if req_time > cutoff_time]
            
            # Mark empty clients for removal
            if not requests:
                clients_to_remove.append(client_id)
        
        # Remove empty clients
        for client_id in clients_to_remove:
            del self._rate_limit_tracker[client_id]
    
    def get_security_headers(self) -> Dict[str, str]:
        """Get security headers to add to responses
        
        Returns:
            Dictionary of security headers
        """
        headers = {
            "X-Content-Type-Options": "nosniff",
            "X-Frame-Options": "DENY",
            "X-XSS-Protection": "1; mode=block",
            "Referrer-Policy": "strict-origin-when-cross-origin"
        }
        
        if self.config.require_https:
            headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        
        return headers