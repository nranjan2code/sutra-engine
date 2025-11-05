"""
Production-grade metrics middleware for FastAPI applications.

Automatically collects API metrics without external dependencies.
Integrates with Sutra's internal monitoring system.
"""

import time
from datetime import datetime
from typing import Callable, Dict, Any
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp
import logging

from .internal_metrics import counter_inc, gauge_set, histogram_observe

logger = logging.getLogger(__name__)


class SutraMetricsMiddleware(BaseHTTPMiddleware):
    """
    Production FastAPI middleware for automatic metrics collection.
    
    Collects:
    - Request count by endpoint and method
    - Request duration by endpoint
    - Response status codes
    - Error rates
    - Concurrent requests
    """
    
    def __init__(self, app: ASGIApp, service_name: str = "sutra"):
        super().__init__(app)
        self.service_name = service_name
        self.active_requests = 0
        
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request and collect metrics."""
        start_time = time.time()
        
        # Track active requests
        self.active_requests += 1
        gauge_set(f"{self.service_name}_active_requests", self.active_requests)
        
        # Extract request info
        method = request.method
        path = self._get_route_path(request)
        
        try:
            # Process request
            response = await call_next(request)
            
            # Collect metrics
            duration = time.time() - start_time
            status_code = response.status_code
            
            # Request counter
            counter_inc(
                f"{self.service_name}_requests_total",
                labels={
                    "method": method,
                    "endpoint": path,
                    "status": str(status_code),
                }
            )
            
            # Request duration
            histogram_observe(
                f"{self.service_name}_request_duration_seconds",
                duration,
                labels={
                    "method": method,
                    "endpoint": path,
                }
            )
            
            # Status code counter
            counter_inc(
                f"{self.service_name}_responses_total",
                labels={
                    "status": str(status_code),
                    "status_class": f"{status_code // 100}xx",
                }
            )
            
            # Error rate (4xx, 5xx)
            if status_code >= 400:
                counter_inc(
                    f"{self.service_name}_errors_total",
                    labels={
                        "method": method,
                        "endpoint": path,
                        "status": str(status_code),
                    }
                )
            
            return response
            
        except Exception as e:
            # Track exceptions
            duration = time.time() - start_time
            
            counter_inc(
                f"{self.service_name}_exceptions_total",
                labels={
                    "method": method,
                    "endpoint": path,
                    "exception": e.__class__.__name__,
                }
            )
            
            histogram_observe(
                f"{self.service_name}_request_duration_seconds",
                duration,
                labels={
                    "method": method,
                    "endpoint": path,
                }
            )
            
            logger.error(f"Exception in {method} {path}: {e}")
            raise
            
        finally:
            # Update active requests
            self.active_requests -= 1
            gauge_set(f"{self.service_name}_active_requests", self.active_requests)
    
    def _get_route_path(self, request: Request) -> str:
        """Extract route path template for consistent labeling."""
        # Try to get the route path template
        route = request.scope.get("route")
        if route:
            return route.path
        
        # Fall back to raw path (less ideal for high-cardinality paths)
        path = request.url.path
        
        # Normalize common patterns to reduce cardinality
        if path.startswith("/api/v"):
            # Handle versioned APIs
            parts = path.split("/")
            if len(parts) >= 4:
                # /api/v1/users/123 -> /api/v1/users/{id}
                normalized = "/".join(parts[:3])
                if len(parts) > 3:
                    # Replace IDs with template
                    for i, part in enumerate(parts[3:], 3):
                        if part.isdigit() or len(part) > 20:  # Likely an ID
                            parts[i] = "{id}"
                return "/".join(parts)
        
        return path


class ProductionMetricsEndpoint:
    """
    Production metrics endpoint for health checks and monitoring.
    
    Provides Prometheus-compatible metrics endpoint without Prometheus dependency.
    """
    
    def __init__(self, metrics_collector=None):
        from .internal_metrics import get_metrics_collector
        self.collector = metrics_collector or get_metrics_collector()
    
    def get_metrics_response(self, format: str = "json") -> Dict[str, Any]:
        """Get metrics in requested format."""
        if format == "prometheus":
            return {
                "content": self.collector.export_prometheus_format(),
                "media_type": "text/plain",
            }
        
        # Default JSON format
        return {
            "content": {
                "system_stats": self.collector.get_system_stats(),
                "metrics": {
                    name: {
                        "latest_value": series.get_latest().value if series.get_latest() else 0,
                        "average_5min": series.get_average(300),
                        "data_points": len(series.points),
                    }
                    for name, series in self.collector.get_all_metrics().items()
                },
                "timestamp": datetime.utcnow().isoformat(),
            },
            "media_type": "application/json",
        }
    
    def get_health_response(self) -> Dict[str, Any]:
        """Get service health status."""
        stats = self.collector.get_system_stats()
        
        # Simple health checks
        healthy = True
        issues = []
        
        # Check if we're collecting metrics
        if stats["metrics_count"] == 0:
            healthy = False
            issues.append("No metrics being collected")
        
        # Check if system metrics are recent
        cpu_metric = self.collector.get_metric("sutra_system_cpu_usage_percent")
        if cpu_metric:
            latest = cpu_metric.get_latest()
            if not latest or (time.time() - latest.timestamp) > 120:  # 2 minutes
                healthy = False
                issues.append("System metrics stale")
        
        return {
            "status": "healthy" if healthy else "unhealthy",
            "timestamp": datetime.utcnow().isoformat(),
            "uptime_seconds": stats["uptime_seconds"],
            "issues": issues,
            "metrics_count": stats["metrics_count"],
            "data_points": stats["total_data_points"],
        }


def create_metrics_endpoints():
    """Create FastAPI routes for metrics and health endpoints."""
    from fastapi import APIRouter
    from fastapi.responses import JSONResponse, PlainTextResponse
    
    router = APIRouter()
    metrics_endpoint = ProductionMetricsEndpoint()
    
    @router.get("/metrics")
    async def get_metrics(format: str = "json"):
        """
        Get system metrics.
        
        - format=json: JSON format (default)
        - format=prometheus: Prometheus-compatible format
        """
        response_data = metrics_endpoint.get_metrics_response(format)
        
        if format == "prometheus":
            return PlainTextResponse(
                content=response_data["content"],
                media_type=response_data["media_type"]
            )
        
        return JSONResponse(
            content=response_data["content"],
            media_type=response_data["media_type"]
        )
    
    @router.get("/health")
    async def get_health():
        """Get service health status."""
        health_data = metrics_endpoint.get_health_response()
        
        status_code = 200 if health_data["status"] == "healthy" else 503
        
        return JSONResponse(
            content=health_data,
            status_code=status_code
        )
    
    @router.get("/metrics/query")
    async def query_metrics(q: str):
        """
        Query metrics using natural language.
        
        Examples:
        - ?q=Show API request rate
        - ?q=What's the memory usage?
        - ?q=How many storage concepts?
        """
        from .internal_metrics import query_metrics
        
        result = query_metrics(q)
        return JSONResponse(content={
            "query": q,
            "result": result,
            "timestamp": datetime.utcnow().isoformat(),
        })
    
    return router


# Convenience function for adding metrics to FastAPI apps
def add_production_monitoring(app, service_name: str = "sutra", include_endpoints: bool = True):
    """
    Add production monitoring to a FastAPI application.
    
    Args:
        app: FastAPI application instance
        service_name: Service name for metrics labeling
        include_endpoints: Whether to add /metrics and /health endpoints
    
    Returns:
        The modified FastAPI app
    """
    # Add metrics middleware
    app.add_middleware(SutraMetricsMiddleware, service_name=service_name)
    
    # Add metrics endpoints
    if include_endpoints:
        metrics_router = create_metrics_endpoints()
        app.include_router(metrics_router, prefix="/internal", tags=["Monitoring"])
    
    logger.info(f"Production monitoring enabled for {service_name} - zero external dependencies")
    
    return app