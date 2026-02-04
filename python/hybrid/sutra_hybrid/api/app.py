"""Main FastAPI application for Sutra AI.

Production-ready API with middleware, error handling, and CORS.
"""

import logging
from contextlib import asynccontextmanager
from typing import Optional
import os

from fastapi import FastAPI, Request, status
from fastapi.exceptions import RequestValidationError
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import ValidationError

from ..engine import SutraAI
from . import openai_endpoints, sutra_endpoints, streaming_endpoints

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Global AI instance
_ai_instance: Optional[SutraAI] = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown."""
    # Startup
    logger.info("Starting Sutra AI API...")
    global _ai_instance

    # Initialize AI instance if not already set
    if _ai_instance is None:
        logger.info("Creating new SutraAI instance (gRPC)...")
        storage_server = os.getenv("SUTRA_STORAGE_SERVER", "storage-server:50051")
        _ai_instance = SutraAI(storage_server=storage_server)

    # Set AI instance in endpoint modules
    openai_endpoints.set_ai_instance(_ai_instance)
    sutra_endpoints.set_ai_instance(_ai_instance)
    streaming_endpoints.set_ai_instance(_ai_instance)

    logger.info("Sutra AI API started successfully")

    yield

    # Shutdown
    logger.info("Shutting down Sutra AI API...")
    if _ai_instance:
        _ai_instance.save()
        logger.info("Knowledge base saved")


def create_app(ai_instance: Optional[SutraAI] = None) -> FastAPI:
    """Create and configure FastAPI application.

    Args:
        ai_instance: Optional pre-configured SutraAI instance

    Returns:
        Configured FastAPI application
    """
    global _ai_instance
    if ai_instance:
        _ai_instance = ai_instance

    # Create FastAPI app
    app = FastAPI(
        title="Sutra AI API",
        description="OpenAI-compatible API with full explainability and audit trails",
        version="1.0.0",
        lifespan=lifespan,
    )

    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Add request logging middleware
    @app.middleware("http")
    async def log_requests(request: Request, call_next):
        """Log all requests."""
        logger.info(f"{request.method} {request.url.path}")
        response = await call_next(request)
        logger.info(f"Response status: {response.status_code}")
        return response

    # Exception handlers
    @app.exception_handler(RequestValidationError)
    async def validation_exception_handler(
        request: Request, exc: RequestValidationError
    ):
        """Handle validation errors."""
        logger.error(f"Validation error: {exc}")
        return JSONResponse(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            content={
                "error": "ValidationError",
                "message": "Invalid request parameters",
                "details": exc.errors(),
            },
        )

    @app.exception_handler(ValidationError)
    async def pydantic_exception_handler(request: Request, exc: ValidationError):
        """Handle Pydantic validation errors."""
        logger.error(f"Pydantic validation error: {exc}")
        return JSONResponse(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            content={
                "error": "ValidationError",
                "message": "Invalid data format",
                "details": exc.errors(),
            },
        )

    @app.exception_handler(Exception)
    async def general_exception_handler(request: Request, exc: Exception):
        """Handle general exceptions."""
        logger.error(f"Unhandled exception: {exc}", exc_info=True)
        return JSONResponse(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            content={
                "error": "InternalServerError",
                "message": "An unexpected error occurred",
                "details": str(exc),
            },
        )

    # Include routers
    app.include_router(openai_endpoints.router)
    app.include_router(sutra_endpoints.router)
    app.include_router(streaming_endpoints.router)

    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint."""
        return {
            "message": "Sutra AI API",
            "version": "1.0.0",
            "docs": "/docs",
            "openapi_endpoints": [
                "/v1/chat/completions",
                "/v1/models",
            ],
            "sutra_endpoints": [
                "/sutra/learn",
                "/sutra/query",
                "/sutra/multi-strategy",
                "/sutra/health",
                "/sutra/audit",
                "/sutra/stats",
            ],
        }

    # Health check endpoint
    @app.get("/ping")
    async def ping():
        """Simple ping endpoint."""
        return {"status": "ok"}

    return app


# Create app instance for uvicorn
app = create_app()
