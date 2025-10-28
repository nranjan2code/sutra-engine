"""
Production-grade monitoring, logging, and error handling for ML-Base service
"""

import asyncio
import functools
import logging
import time
import traceback
from contextlib import asynccontextmanager
from typing import Any, Callable, Dict, Optional

import psutil
from fastapi import HTTPException, Request
from fastapi.responses import JSONResponse


# ================================
# Structured Logging Setup
# ================================

def setup_production_logging(service_name: str, log_level: str = "INFO", format_type: str = "json") -> logging.Logger:
    """Setup production-grade structured logging"""
    
    # Create logger
    logger = logging.getLogger(service_name)
    logger.setLevel(getattr(logging, log_level.upper()))
    
    # Remove existing handlers
    for handler in logger.handlers[:]:
        logger.removeHandler(handler)
    
    # Create handler
    handler = logging.StreamHandler()
    
    # Set format
    if format_type == "json":
        import json
        
        class JSONFormatter(logging.Formatter):
            def format(self, record):
                log_entry = {
                    "timestamp": self.formatTime(record, self.datefmt),
                    "level": record.levelname,
                    "service": service_name,
                    "logger": record.name,
                    "message": record.getMessage(),
                    "module": record.module,
                    "function": record.funcName,
                    "line": record.lineno
                }
                
                # Add exception info if present
                if record.exc_info:
                    log_entry["exception"] = self.formatException(record.exc_info)
                
                # Add extra fields
                for key, value in record.__dict__.items():
                    if key not in ['name', 'msg', 'args', 'levelname', 'levelno', 'pathname', 'filename',
                                   'module', 'exc_info', 'exc_text', 'stack_info', 'lineno', 'funcName',
                                   'created', 'msecs', 'relativeCreated', 'thread', 'threadName',
                                   'processName', 'process', 'message']:
                        log_entry[key] = value
                
                return json.dumps(log_entry, default=str)
        
        formatter = JSONFormatter()
    else:
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(funcName)s:%(lineno)d - %(message)s'
        )
    
    handler.setFormatter(formatter)
    logger.addHandler(handler)
    
    return logger


# ================================
# Performance Monitoring
# ================================

class PerformanceMonitor:
    """Monitor system and application performance"""
    
    def __init__(self):
        self.start_time = time.time()
        self.request_count = 0
        self.error_count = 0
        self.total_processing_time = 0.0
        self.model_stats = {}
    
    def get_system_metrics(self) -> Dict[str, Any]:
        """Get current system metrics"""
        try:
            return {
                "cpu_percent": psutil.cpu_percent(interval=0.1),
                "memory_percent": psutil.virtual_memory().percent,
                "memory_used_mb": psutil.virtual_memory().used / (1024 * 1024),
                "memory_available_mb": psutil.virtual_memory().available / (1024 * 1024),
                "disk_usage_percent": psutil.disk_usage('/').percent,
                "uptime_seconds": time.time() - self.start_time,
                "load_average": psutil.getloadavg()[:3] if hasattr(psutil, 'getloadavg') else [0, 0, 0]
            }
        except Exception as e:
            return {"error": str(e)}
    
    def get_application_metrics(self) -> Dict[str, Any]:
        """Get application-specific metrics"""
        uptime = time.time() - self.start_time
        avg_response_time = (
            self.total_processing_time / self.request_count 
            if self.request_count > 0 else 0
        )
        
        return {
            "uptime_seconds": uptime,
            "total_requests": self.request_count,
            "error_count": self.error_count,
            "error_rate": self.error_count / max(self.request_count, 1),
            "requests_per_second": self.request_count / max(uptime, 1),
            "average_response_time_ms": avg_response_time * 1000,
            "model_stats": self.model_stats.copy()
        }
    
    def record_request(self, processing_time: float, model_id: Optional[str] = None, success: bool = True):
        """Record request metrics"""
        self.request_count += 1
        self.total_processing_time += processing_time
        
        if not success:
            self.error_count += 1
        
        if model_id:
            if model_id not in self.model_stats:
                self.model_stats[model_id] = {
                    "requests": 0,
                    "total_time": 0.0,
                    "errors": 0
                }
            
            self.model_stats[model_id]["requests"] += 1
            self.model_stats[model_id]["total_time"] += processing_time
            if not success:
                self.model_stats[model_id]["errors"] += 1


# Global performance monitor instance
performance_monitor = PerformanceMonitor()


# ================================
# Error Handling Decorators
# ================================

def handle_ml_errors(logger: logging.Logger):
    """Decorator for handling ML service errors with proper logging"""
    
    def decorator(func: Callable):
        @functools.wraps(func)
        async def async_wrapper(*args, **kwargs):
            start_time = time.time()
            model_id = kwargs.get('model_id') or (args[1].model_id if len(args) > 1 and hasattr(args[1], 'model_id') else None)
            
            try:
                result = await func(*args, **kwargs)
                
                # Record successful request
                processing_time = time.time() - start_time
                performance_monitor.record_request(processing_time, model_id, success=True)
                
                logger.info(
                    "ML request completed",
                    extra={
                        "function": func.__name__,
                        "model_id": model_id,
                        "processing_time_ms": processing_time * 1000,
                        "success": True
                    }
                )
                
                return result
                
            except HTTPException:
                # Re-raise HTTP exceptions (client errors)
                processing_time = time.time() - start_time
                performance_monitor.record_request(processing_time, model_id, success=False)
                raise
                
            except Exception as e:
                # Handle and log unexpected errors
                processing_time = time.time() - start_time
                performance_monitor.record_request(processing_time, model_id, success=False)
                
                error_id = f"ml_error_{int(time.time())}"
                logger.error(
                    "ML request failed",
                    extra={
                        "error_id": error_id,
                        "function": func.__name__,
                        "model_id": model_id,
                        "processing_time_ms": processing_time * 1000,
                        "error_type": type(e).__name__,
                        "error_message": str(e),
                        "traceback": traceback.format_exc()
                    }
                )
                
                # Convert to HTTP exception
                if "model" in str(e).lower() and "not" in str(e).lower():
                    raise HTTPException(status_code=404, detail=f"Model error: {str(e)}")
                elif "memory" in str(e).lower() or "cuda" in str(e).lower():
                    raise HTTPException(status_code=503, detail=f"Resource error: {str(e)}")
                elif "timeout" in str(e).lower():
                    raise HTTPException(status_code=504, detail=f"Timeout error: {str(e)}")
                else:
                    raise HTTPException(
                        status_code=500, 
                        detail=f"Internal ML error (ID: {error_id}). Check logs for details."
                    )
        
        @functools.wraps(func)
        def sync_wrapper(*args, **kwargs):
            start_time = time.time()
            
            try:
                result = func(*args, **kwargs)
                processing_time = time.time() - start_time
                performance_monitor.record_request(processing_time, success=True)
                return result
                
            except Exception as e:
                processing_time = time.time() - start_time
                performance_monitor.record_request(processing_time, success=False)
                
                logger.error(
                    "Sync ML operation failed",
                    extra={
                        "function": func.__name__,
                        "processing_time_ms": processing_time * 1000,
                        "error_type": type(e).__name__,
                        "error_message": str(e)
                    }
                )
                raise
        
        # Return appropriate wrapper based on function type
        return async_wrapper if asyncio.iscoroutinefunction(func) else sync_wrapper
    
    return decorator


# ================================
# Circuit Breaker Pattern
# ================================

class CircuitBreaker:
    """Circuit breaker for ML model operations"""
    
    def __init__(self, failure_threshold: int = 5, timeout: float = 60.0):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failure_count = 0
        self.last_failure_time = None
        self.state = "closed"  # closed, open, half_open
    
    def call(self, func: Callable, *args, **kwargs):
        """Execute function with circuit breaker protection"""
        
        if self.state == "open":
            if time.time() - self.last_failure_time >= self.timeout:
                self.state = "half_open"
            else:
                raise HTTPException(
                    status_code=503, 
                    detail="Service temporarily unavailable (circuit breaker open)"
                )
        
        try:
            result = func(*args, **kwargs)
            
            # Success - reset circuit breaker
            if self.state == "half_open":
                self.state = "closed"
                self.failure_count = 0
            
            return result
            
        except Exception as e:
            self.failure_count += 1
            self.last_failure_time = time.time()
            
            if self.failure_count >= self.failure_threshold:
                self.state = "open"
            
            raise e


# ================================
# Health Check Manager
# ================================

class HealthCheckManager:
    """Manage health checks for ML models and system resources"""
    
    def __init__(self):
        self.checks = {}
        self.last_check_time = {}
        self.check_results = {}
    
    def register_check(self, name: str, check_func: Callable, interval: float = 30.0):
        """Register a health check"""
        self.checks[name] = {
            "func": check_func,
            "interval": interval
        }
        self.last_check_time[name] = 0
        self.check_results[name] = {"status": "unknown", "message": "Not yet checked"}
    
    async def run_checks(self) -> Dict[str, Any]:
        """Run all health checks"""
        current_time = time.time()
        results = {}
        
        for name, check_info in self.checks.items():
            # Check if it's time to run this check
            if current_time - self.last_check_time[name] >= check_info["interval"]:
                try:
                    if asyncio.iscoroutinefunction(check_info["func"]):
                        result = await check_info["func"]()
                    else:
                        result = check_info["func"]()
                    
                    self.check_results[name] = {
                        "status": "healthy",
                        "message": result if isinstance(result, str) else "OK",
                        "last_check": current_time
                    }
                    
                except Exception as e:
                    self.check_results[name] = {
                        "status": "unhealthy", 
                        "message": str(e),
                        "last_check": current_time
                    }
                
                self.last_check_time[name] = current_time
            
            results[name] = self.check_results[name]
        
        # Overall health status
        overall_status = "healthy"
        if any(result["status"] == "unhealthy" for result in results.values()):
            overall_status = "unhealthy"
        elif any(result["status"] == "unknown" for result in results.values()):
            overall_status = "degraded"
        
        return {
            "status": overall_status,
            "checks": results,
            "timestamp": current_time
        }


# ================================
# Global Exception Handler
# ================================

async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """Global exception handler for production error responses"""
    
    error_id = f"error_{int(time.time() * 1000)}"
    
    # Log the error
    logger = logging.getLogger("ml-base-service")
    logger.error(
        "Unhandled exception",
        extra={
            "error_id": error_id,
            "path": request.url.path,
            "method": request.method,
            "error_type": type(exc).__name__,
            "error_message": str(exc),
            "traceback": traceback.format_exc()
        }
    )
    
    # Return user-friendly error response
    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal server error",
            "error_id": error_id,
            "message": "An unexpected error occurred. Please contact support with this error ID.",
            "timestamp": time.time()
        }
    )


# ================================
# Request Context Manager
# ================================

@asynccontextmanager
async def request_context(request_id: str, operation: str, logger: logging.Logger):
    """Context manager for request tracking and logging"""
    
    start_time = time.time()
    logger.info(
        "Request started",
        extra={
            "request_id": request_id,
            "operation": operation,
            "start_time": start_time
        }
    )
    
    try:
        yield
        
        # Log successful completion
        processing_time = time.time() - start_time
        logger.info(
            "Request completed",
            extra={
                "request_id": request_id,
                "operation": operation,
                "processing_time_ms": processing_time * 1000,
                "success": True
            }
        )
        
    except Exception as e:
        # Log error completion
        processing_time = time.time() - start_time
        logger.error(
            "Request failed", 
            extra={
                "request_id": request_id,
                "operation": operation,
                "processing_time_ms": processing_time * 1000,
                "success": False,
                "error_type": type(e).__name__,
                "error_message": str(e)
            }
        )
        raise