"""
Base ML Service Framework for Sutra

Provides FastAPI-based service scaffolding with:
- Edition-aware feature gating and resource management
- Standardized health checks, metrics, and observability
- Consistent API patterns and response schemas
- Security, authentication, and access control
- Performance optimization and caching
"""

import asyncio
import logging
import os
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, Optional, List, Callable, Union
from datetime import datetime, timedelta

try:
    from fastapi import FastAPI, HTTPException, Request, Response
    from fastapi.middleware.cors import CORSMiddleware
    from pydantic import BaseModel, Field
    import uvicorn
    HAS_FASTAPI = True
except ImportError:
    HAS_FASTAPI = False
    logging.warning("FastAPI not available - web service features disabled")

from .edition import EditionManager, Edition
from .metrics import MetricsCollector, ServiceMetrics

logger = logging.getLogger(__name__)


class HealthStatus(Enum):
    """Service health status levels"""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    LOADING = "loading"


@dataclass
class ServiceConfig:
    """Configuration for ML service"""
    # Service identity
    service_name: str
    service_version: str = "2.0.0"
    instance_id: Optional[str] = None
    
    # Network settings
    host: str = "0.0.0.0"
    port: int = 8888
    workers: int = 1
    
    # API settings
    api_prefix: str = ""
    enable_cors: bool = True
    cors_origins: List[str] = None
    
    # Performance settings
    request_timeout: float = 30.0
    max_concurrent_requests: int = 100
    
    # Security settings
    require_auth: bool = False
    api_keys: List[str] = None
    rate_limit_per_minute: int = 1000
    
    # Observability
    enable_metrics: bool = True
    log_level: str = "INFO"
    
    def __post_init__(self):
        """Validate and set defaults"""
        if self.instance_id is None:
            self.instance_id = f"{self.service_name}-{os.getpid()}"
        
        if self.cors_origins is None:
            self.cors_origins = ["*"] if not self.require_auth else []
        
        if self.api_keys is None:
            self.api_keys = []


# Standard response models
class HealthResponse(BaseModel):
    """Standard health check response"""
    status: str = Field(..., description="Service health status")
    service: str = Field(..., description="Service name")
    version: str = Field(..., description="Service version")
    instance_id: str = Field(..., description="Service instance ID")
    timestamp: str = Field(..., description="Response timestamp")
    uptime_seconds: float = Field(..., description="Service uptime in seconds")
    edition: str = Field(..., description="Sutra edition")
    
    # Service-specific health data
    model_loaded: bool = Field(default=False, description="Whether model is loaded")
    model_name: Optional[str] = Field(default=None, description="Loaded model name")
    device: str = Field(default="unknown", description="Model device")
    memory_usage_gb: float = Field(default=0.0, description="Memory usage in GB")
    
    # Performance indicators
    total_requests: int = Field(default=0, description="Total requests processed")
    avg_response_time_ms: float = Field(default=0.0, description="Average response time")
    error_rate: float = Field(default=0.0, description="Error rate percentage")


class InfoResponse(BaseModel):
    """Service information response"""
    service: str = Field(..., description="Service name")
    version: str = Field(..., description="Service version")
    description: str = Field(..., description="Service description")
    edition: Dict[str, Any] = Field(..., description="Edition configuration")
    capabilities: List[str] = Field(..., description="Available capabilities")
    endpoints: List[str] = Field(..., description="Available endpoints")
    
    # Resource information
    max_batch_size: int = Field(..., description="Maximum batch size")
    max_sequence_length: int = Field(..., description="Maximum sequence length")
    supported_models: List[str] = Field(..., description="Supported model types")


class ErrorResponse(BaseModel):
    """Standard error response"""
    error: str = Field(..., description="Error message")
    error_code: str = Field(..., description="Error code")
    instance_id: str = Field(..., description="Service instance ID")
    timestamp: str = Field(..., description="Error timestamp")
    request_id: Optional[str] = Field(default=None, description="Request ID for tracing")


class BaseMlService(ABC):
    """Abstract base class for Sutra ML services"""
    
    def __init__(self, config: ServiceConfig, edition_manager: Optional[EditionManager] = None):
        """Initialize ML service
        
        Args:
            config: Service configuration
            edition_manager: Edition manager (auto-created if None)
        """
        if not HAS_FASTAPI:
            raise RuntimeError("FastAPI required for ML services")
        
        self.config = config
        self.edition_manager = edition_manager or EditionManager()
        self.start_time = time.time()
        self.metrics = MetricsCollector() if config.enable_metrics else None
        
        # Service state
        self._health_status = HealthStatus.LOADING
        self._model_loaded = False
        self._model_info = {}
        
        # Create FastAPI app
        self.app = self._create_fastapi_app()
        
        # Setup routes
        self._setup_standard_routes()
        self._setup_service_routes()
        
        logger.info(f"Initialized {config.service_name} service (edition: {self.edition_manager.edition.value})")
    
    @abstractmethod
    async def load_model(self) -> bool:
        """Load the ML model
        
        Returns:
            True if model loaded successfully, False otherwise
        """
        pass
    
    @abstractmethod
    async def process_request(self, request: Any) -> Any:
        """Process a service-specific request
        
        Args:
            request: Request data (service-specific)
            
        Returns:
            Response data (service-specific)
        """
        pass
    
    @abstractmethod
    def get_service_info(self) -> Dict[str, Any]:
        """Get service-specific information
        
        Returns:
            Dictionary with service information
        """
        pass
    
    def _create_fastapi_app(self) -> FastAPI:
        """Create and configure FastAPI application"""
        app = FastAPI(
            title=self.config.service_name,
            version=self.config.service_version,
            description=f"Sutra AI {self.config.service_name} Service"
        )
        
        # Add CORS middleware
        if self.config.enable_cors:
            app.add_middleware(
                CORSMiddleware,
                allow_origins=self.config.cors_origins,
                allow_credentials=True,
                allow_methods=["*"],
                allow_headers=["*"],
            )
        
        # Add request timing middleware
        @app.middleware("http")
        async def add_timing_header(request: Request, call_next: Callable):
            start_time = time.time()
            response = await call_next(request)
            process_time = time.time() - start_time
            
            response.headers["X-Process-Time"] = str(process_time)
            response.headers["X-Instance-ID"] = self.config.instance_id
            
            # Record metrics
            if self.metrics:
                self.metrics.record_request(
                    endpoint=str(request.url.path),
                    method=request.method,
                    status_code=response.status_code,
                    processing_time_ms=process_time * 1000
                )
            
            return response
        
        # Add error handling middleware
        @app.exception_handler(Exception)
        async def global_exception_handler(request: Request, exc: Exception) -> Response:
            logger.error(f"Unhandled exception in {request.method} {request.url.path}: {exc}")
            
            error_response = ErrorResponse(
                error=str(exc),
                error_code="INTERNAL_ERROR",
                instance_id=self.config.instance_id,
                timestamp=datetime.utcnow().isoformat()
            )
            
            return Response(
                content=error_response.model_dump_json(),
                status_code=500,
                media_type="application/json"
            )
        
        return app
    
    def _setup_standard_routes(self):
        """Setup standard routes available in all ML services"""
        
        @self.app.get("/", response_model=Dict[str, Any])
        async def root():
            """Root endpoint with service information"""
            return {
                "service": self.config.service_name,
                "version": self.config.service_version,
                "status": self._health_status.value,
                "edition": self.edition_manager.edition.value,
                "instance_id": self.config.instance_id,
                "capabilities": self._get_capabilities()
            }
        
        @self.app.get("/health", response_model=HealthResponse)
        async def health():
            """Comprehensive health check endpoint"""
            uptime = time.time() - self.start_time
            
            # Get metrics
            total_requests = 0
            avg_response_time = 0.0
            error_rate = 0.0
            
            if self.metrics:
                stats = self.metrics.get_stats()
                total_requests = stats.total_requests
                avg_response_time = stats.avg_response_time_ms
                error_rate = stats.error_rate
            
            return HealthResponse(
                status=self._health_status.value,
                service=self.config.service_name,
                version=self.config.service_version,
                instance_id=self.config.instance_id,
                timestamp=datetime.utcnow().isoformat(),
                uptime_seconds=uptime,
                edition=self.edition_manager.edition.value,
                model_loaded=self._model_loaded,
                model_name=self._model_info.get("model_name"),
                device=self._model_info.get("device", "unknown"),
                memory_usage_gb=self._model_info.get("memory_usage_gb", 0.0),
                total_requests=total_requests,
                avg_response_time_ms=avg_response_time,
                error_rate=error_rate
            )
        
        @self.app.get("/info", response_model=InfoResponse)
        async def info():
            """Service information and capabilities"""
            if not self._model_loaded:
                raise HTTPException(status_code=503, detail="Service not ready - model not loaded")
            
            service_info = self.get_service_info()
            
            return InfoResponse(
                service=self.config.service_name,
                version=self.config.service_version,
                description=service_info.get("description", "Sutra AI ML Service"),
                edition=self.edition_manager.get_edition_info(),
                capabilities=self._get_capabilities(),
                endpoints=self._get_endpoints(),
                max_batch_size=self.edition_manager.get_batch_size_limit(),
                max_sequence_length=self.edition_manager.get_sequence_length_limit(),
                supported_models=service_info.get("supported_models", [])
            )
        
        @self.app.get("/metrics", response_model=Dict[str, Any])
        async def metrics():
            """Service metrics and performance data"""
            if not self.config.enable_metrics or not self.metrics:
                raise HTTPException(status_code=404, detail="Metrics not enabled")
            
            stats = self.metrics.get_stats()
            
            return {
                "service": self.config.service_name,
                "instance_id": self.config.instance_id,
                "edition": self.edition_manager.edition.value,
                "uptime_seconds": time.time() - self.start_time,
                "requests": {
                    "total": stats.total_requests,
                    "successful": stats.successful_requests,
                    "failed": stats.failed_requests,
                    "error_rate": stats.error_rate
                },
                "performance": {
                    "avg_response_time_ms": stats.avg_response_time_ms,
                    "min_response_time_ms": stats.min_response_time_ms,
                    "max_response_time_ms": stats.max_response_time_ms,
                    "requests_per_second": stats.requests_per_second
                },
                "resources": {
                    "model_loaded": self._model_loaded,
                    "memory_usage_gb": self._model_info.get("memory_usage_gb", 0.0),
                    "model_parameters": self._model_info.get("parameters", 0)
                }
            }
    
    @abstractmethod
    def _setup_service_routes(self):
        """Setup service-specific routes (implemented by subclasses)"""
        pass
    
    def _get_capabilities(self) -> List[str]:
        """Get list of service capabilities based on edition"""
        capabilities = ["health_check", "metrics", "info"]
        
        # Add edition-specific capabilities
        if self.edition_manager.supports_custom_models():
            capabilities.append("custom_models")
        
        if self.edition_manager.supports_advanced_caching():
            capabilities.append("advanced_caching")
        
        if self.edition_manager.supports_multi_gpu():
            capabilities.append("multi_gpu")
        
        return capabilities
    
    def _get_endpoints(self) -> List[str]:
        """Get list of available endpoints"""
        return [route.path for route in self.app.routes if hasattr(route, 'path')]
    
    def set_model_loaded(self, model_info: Dict[str, Any]):
        """Mark model as loaded and update service status
        
        Args:
            model_info: Information about the loaded model
        """
        self._model_loaded = True
        self._model_info = model_info
        self._health_status = HealthStatus.HEALTHY
        
        logger.info(f"Model loaded: {model_info.get('model_name', 'unknown')}")
    
    def set_health_status(self, status: HealthStatus):
        """Update service health status
        
        Args:
            status: New health status
        """
        self._health_status = status
        logger.info(f"Health status updated: {status.value}")
    
    async def startup(self):
        """Startup sequence for the service"""
        logger.info(f"Starting {self.config.service_name} service...")
        
        # Load model
        success = await self.load_model()
        if not success:
            self._health_status = HealthStatus.UNHEALTHY
            raise RuntimeError("Failed to load model during startup")
        
        logger.info(f"{self.config.service_name} service ready")
    
    def run(self):
        """Run the service with uvicorn"""
        if not HAS_FASTAPI:
            raise RuntimeError("FastAPI required to run service")
        
        # Setup startup event
        @self.app.on_event("startup")
        async def startup_event():
            await self.startup()
        
        # Run with uvicorn
        uvicorn.run(
            self.app,
            host=self.config.host,
            port=self.config.port,
            workers=1,  # Single worker for model services
            log_level=self.config.log_level.lower(),
            access_log=True
        )