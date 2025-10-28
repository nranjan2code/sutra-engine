"""
Sutra Control Center - Secure Gateway
Acts as the only interface between external clients and internal Sutra AI systems.
All internal communication via gRPC is abstracted from clients.
"""

import asyncio
import logging
import os
from datetime import datetime
from enum import Enum
from typing import Dict, List, Optional, Any
import json

import grpc
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, HTTPException
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Import gRPC client for internal communication
# from sutra_storage_client.client import StorageClient  # TODO: Add when available
# Import Grid API integration
from backend.grid_api import GridManager
# Import dependency scanner
from backend.dependency_scanner import DependencyScanner, PackageHealth, VulnerabilitySeverity

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class ServiceStatus(str, Enum):
    """Service status - no internal details exposed"""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNAVAILABLE = "unavailable"


class SystemHealth(BaseModel):
    """System health response - abstracts internal details"""
    status: ServiceStatus
    uptime: str
    last_update: str


class SystemMetrics(BaseModel):
    """System metrics - no internal service names or details"""
    knowledge_items: int = 0
    connections: int = 0
    activity_score: float = 0.0
    response_time_ms: float = 0.0
    system_load: float = 0.0
    memory_usage: float = 0.0
    timestamp: str


class ControlGateway:
    """Gateway that manages all internal system communication via gRPC"""
    
    def __init__(self):
        self.storage_server = os.getenv("SUTRA_STORAGE_SERVER", "localhost:50051")
        self.grid_master_addr = os.getenv("SUTRA_GRID_MASTER", "localhost:7000")
        self._storage_client = None
        self._grid_manager = None
        self.start_time = datetime.utcnow()
        
    async def get_storage_client(self) -> Optional[Any]:
        """Get storage client with connection handling"""
        try:
            # TODO: Implement when StorageClient is available
            logger.info(f"Storage client requested for {self.storage_server}")
            return None
        except Exception as e:
            logger.error(f"Storage connection failed: {e}")
            return None
    
    def get_grid_manager(self) -> GridManager:
        """Get Grid manager instance"""
        if not self._grid_manager:
            self._grid_manager = GridManager(self.storage_server, self.grid_master_addr)
        return self._grid_manager
    
    async def get_system_health(self) -> SystemHealth:
        """Get overall system health - abstracts internal components"""
        try:
            # For testing, assume system is healthy
            uptime = str(datetime.utcnow() - self.start_time).split('.')[0]
            
            return SystemHealth(
                status=ServiceStatus.HEALTHY,
                uptime=uptime,
                last_update=datetime.utcnow().isoformat()
            )
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return SystemHealth(
                status=ServiceStatus.UNAVAILABLE,
                uptime="unknown",
                last_update=datetime.utcnow().isoformat()
            )
    
    async def get_system_metrics(self) -> SystemMetrics:
        """Get system metrics - no internal details exposed"""
        try:
            # For testing, return mock metrics
            return SystemMetrics(
                knowledge_items=1250,
                connections=3420,
                activity_score=75.0,
                response_time_ms=1.0,
                system_load=25.0,
                memory_usage=60.0,
                timestamp=datetime.utcnow().isoformat()
            )
        except Exception as e:
            logger.error(f"Metrics collection failed: {e}")
            return SystemMetrics(timestamp=datetime.utcnow().isoformat())
    
    async def execute_query(self, query_text: str) -> Dict[str, Any]:
        """Execute query against internal systems - abstract response"""
        try:
            # For testing, return a mock successful response
            return {
                "success": True,
                "response": f"Mock query processed: '{query_text[:50]}...' ({len(query_text)} characters)",
                "processing_time_ms": 150,
                "confidence": 0.85
            }
        except Exception as e:
            logger.error(f"Query execution failed: {e}")
            return {"success": False, "error": "Query processing failed"}


# Initialize FastAPI app
app = FastAPI(
    title="Sutra AI Control Center",
    description="Secure gateway to Sutra AI system",
    version="2.0.0",
    # Remove docs in production to prevent API discovery
    docs_url=None if os.getenv("ENVIRONMENT") == "production" else "/docs",
    redoc_url=None if os.getenv("ENVIRONMENT") == "production" else "/redoc"
)

# Configure CORS - restrict in production
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000"] if os.getenv("ENVIRONMENT") != "production" else [],
    allow_credentials=True,
    allow_methods=["GET", "POST"],
    allow_headers=["*"],
)

# Mount static files for React app if available
import os
from fastapi.responses import FileResponse

# Determine static files location (Docker vs local dev)
STATIC_DIR = "/app/static" if os.path.exists("/app/static") else "/app/build"

# Mount assets directory if it exists
assets_dir = os.path.join(STATIC_DIR, "assets")
if os.path.exists(assets_dir):
    app.mount("/assets", StaticFiles(directory=assets_dir), name="assets")

# Initialize gateway
gateway = ControlGateway()

# Active WebSocket connections
active_connections: List[WebSocket] = []


@app.get("/", response_class=HTMLResponse)
async def serve_app():
    """Serve React application"""
    index_path = os.path.join(STATIC_DIR, "index.html")
    try:
        with open(index_path, "r") as f:
            return f.read()
    except FileNotFoundError:
        return HTMLResponse("<h1>Sutra AI Control Center</h1><p>System initializing...</p>")


@app.get("/health")
async def health_check():
    """Public health endpoint - no internal details"""
    health = await gateway.get_system_health()
    return health


@app.get("/api/metrics")
async def get_metrics():
    """Get system metrics - abstracted from internal services"""
    metrics = await gateway.get_system_metrics()
    return metrics


@app.post("/api/query")
async def execute_query(request: dict):
    """Execute query - abstracts internal processing"""
    query_text = request.get("query", "").strip()
    if not query_text:
        raise HTTPException(status_code=400, detail="Query text required")
    
    if len(query_text) > 1000:  # Prevent abuse
        raise HTTPException(status_code=400, detail="Query too long")
    
    result = await gateway.execute_query(query_text)
    return result


# ===== Grid Management Endpoints =====

@app.get("/api/grid/agents")
async def get_grid_agents():
    """Get all Grid agents"""
    try:
        grid_manager = gateway.get_grid_manager()
        agents = await grid_manager.get_agents()
        return {"agents": [agent.dict() for agent in agents]}
    except Exception as e:
        logger.error(f"Failed to get Grid agents: {e}")
        raise HTTPException(status_code=500, detail="Failed to retrieve agents")


@app.get("/api/grid/status")
async def get_grid_status():
    """Get Grid cluster status"""
    try:
        grid_manager = gateway.get_grid_manager()
        status = await grid_manager.get_cluster_status()
        return status.dict()
    except Exception as e:
        logger.error(f"Failed to get Grid status: {e}")
        raise HTTPException(status_code=500, detail="Failed to retrieve status")


@app.get("/api/grid/events")
async def get_grid_events(
    event_type: Optional[str] = None,
    entity_id: Optional[str] = None,
    hours: int = 24
):
    """Query Grid events from Sutra Storage"""
    try:
        grid_manager = gateway.get_grid_manager()
        events = await grid_manager.query_grid_events(event_type, entity_id, hours)
        return {"events": [event.dict() for event in events]}
    except Exception as e:
        logger.error(f"Failed to query Grid events: {e}")
        raise HTTPException(status_code=500, detail="Failed to query events")


@app.post("/api/grid/spawn")
async def spawn_storage_node(request: dict):
    """Spawn a storage node"""
    try:
        grid_manager = gateway.get_grid_manager()
        result = await grid_manager.spawn_storage_node(
            agent_id=request["agent_id"],
            port=request["port"],
            storage_path=request.get("storage_path", "/tmp/storage"),
            memory_limit_mb=request.get("memory_limit_mb", 512)
        )
        return result
    except KeyError as e:
        raise HTTPException(status_code=400, detail=f"Missing required field: {e}")
    except Exception as e:
        logger.error(f"Failed to spawn storage node: {e}")
        raise HTTPException(status_code=500, detail="Failed to spawn node")


@app.post("/api/grid/stop")
async def stop_storage_node(request: dict):
    """Stop a storage node"""
    try:
        grid_manager = gateway.get_grid_manager()
        result = await grid_manager.stop_storage_node(
            agent_id=request["agent_id"],
            node_id=request["node_id"]
        )
        return result
    except KeyError as e:
        raise HTTPException(status_code=400, detail=f"Missing required field: {e}")
    except Exception as e:
        logger.error(f"Failed to stop storage node: {e}")
        raise HTTPException(status_code=500, detail="Failed to stop node")


@app.post("/api/grid/query")
async def natural_language_grid_query(request: dict):
    """Execute natural language queries against Grid data"""
    try:
        query_text = request.get("query", "").strip()
        if not query_text:
            raise HTTPException(status_code=400, detail="Query text required")
        
        grid_manager = gateway.get_grid_manager()
        result = await grid_manager.natural_language_grid_query(query_text)
        return result
    except Exception as e:
        logger.error(f"Natural language Grid query failed: {e}")
        raise HTTPException(status_code=500, detail="Query failed")


# ===== Semantic Reasoning Gateway Endpoints =====

@app.post("/api/semantic/path")
async def semantic_path_gateway(request: dict):
    """Gateway to semantic pathfinding endpoint"""
    try:
        # Forward to sutra-api semantic endpoint
        import httpx
        api_url = os.getenv("SUTRA_API_URL", "http://sutra-api:8000")
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{api_url}/sutra/semantic/path",
                json=request,
            )
            return response.json()
    except Exception as e:
        logger.error(f"Semantic path gateway failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/semantic/temporal")
async def temporal_chain_gateway(request: dict):
    """Gateway to temporal chain endpoint"""
    try:
        import httpx
        api_url = os.getenv("SUTRA_API_URL", "http://sutra-api:8000")
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{api_url}/sutra/semantic/temporal",
                json=request,
            )
            return response.json()
    except Exception as e:
        logger.error(f"Temporal chain gateway failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/semantic/causal")
async def causal_chain_gateway(request: dict):
    """Gateway to causal chain endpoint"""
    try:
        import httpx
        api_url = os.getenv("SUTRA_API_URL", "http://sutra-api:8000")
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{api_url}/sutra/semantic/causal",
                json=request,
            )
            return response.json()
    except Exception as e:
        logger.error(f"Causal chain gateway failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/semantic/contradictions")
async def contradictions_gateway(request: dict):
    """Gateway to contradiction detection endpoint"""
    try:
        import httpx
        api_url = os.getenv("SUTRA_API_URL", "http://sutra-api:8000")
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{api_url}/sutra/semantic/contradictions",
                json=request,
            )
            return response.json()
    except Exception as e:
        logger.error(f"Contradiction detection gateway failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/semantic/query")
async def semantic_query_gateway(request: dict):
    """Gateway to semantic query endpoint"""
    try:
        import httpx
        api_url = os.getenv("SUTRA_API_URL", "http://sutra-api:8000")
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{api_url}/sutra/semantic/query",
                json=request,
            )
            return response.json()
    except Exception as e:
        logger.error(f"Semantic query gateway failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ===== Dependency Management Endpoints =====

@app.get("/api/dependencies/scan")
async def scan_dependencies():
    """Scan all packages for dependencies and vulnerabilities"""
    try:
        from pathlib import Path
        project_root = Path("/app").parent.parent  # Adjust based on Docker setup
        scanner = DependencyScanner(project_root)
        health_reports = await scanner.scan_all_packages()
        
        # Convert to JSON-serializable format
        results = {}
        for path, health in health_reports.items():
            results[path] = {
                "package_type": health.package_type.value,
                "total_dependencies": health.total_dependencies,
                "outdated_count": health.outdated_count,
                "vulnerable_count": health.vulnerable_count,
                "critical_vulns": health.critical_vulns,
                "high_vulns": health.high_vulns,
                "last_scanned": health.last_scanned.isoformat(),
                "dependencies": [
                    {
                        "name": dep.name,
                        "version": dep.version,
                        "latest_version": dep.latest_version,
                        "outdated": dep.outdated,
                        "vulnerabilities": [
                            {
                                "severity": vuln.severity.value,
                                "cve": vuln.cve,
                                "description": vuln.description,
                                "fixed_version": vuln.fixed_version
                            }
                            for vuln in dep.vulnerabilities
                        ]
                    }
                    for dep in health.dependencies
                ]
            }
        
        return {"packages": results}
    except Exception as e:
        logger.error(f"Dependency scan failed: {e}")
        raise HTTPException(status_code=500, detail="Dependency scan failed")


@app.get("/api/dependencies/summary")
async def get_dependency_summary():
    """Get summary of dependency health across all packages"""
    try:
        from pathlib import Path
        project_root = Path("/app").parent.parent
        scanner = DependencyScanner(project_root)
        health_reports = await scanner.scan_all_packages()
        
        # Calculate totals
        total_deps = 0
        total_outdated = 0
        total_vulnerable = 0
        total_critical = 0
        total_high = 0
        package_count = len(health_reports)
        
        for health in health_reports.values():
            total_deps += health.total_dependencies
            total_outdated += health.outdated_count
            total_vulnerable += health.vulnerable_count
            total_critical += health.critical_vulns
            total_high += health.high_vulns
        
        # Get unique dependency counts by type
        python_deps = set()
        rust_deps = set()
        node_deps = set()
        
        for health in health_reports.values():
            for dep in health.dependencies:
                if dep.package_type.value == "python":
                    python_deps.add(dep.name)
                elif dep.package_type.value == "rust":
                    rust_deps.add(dep.name)
                elif dep.package_type.value == "node":
                    node_deps.add(dep.name)
        
        return {
            "summary": {
                "total_packages": package_count,
                "total_dependencies": total_deps,
                "unique_python": len(python_deps),
                "unique_rust": len(rust_deps),
                "unique_node": len(node_deps),
                "outdated_dependencies": total_outdated,
                "vulnerable_dependencies": total_vulnerable,
                "critical_vulnerabilities": total_critical,
                "high_vulnerabilities": total_high,
                "health_score": max(0, 100 - (total_critical * 10 + total_high * 5 + total_outdated))
            }
        }
    except Exception as e:
        logger.error(f"Failed to get dependency summary: {e}")
        raise HTTPException(status_code=500, detail="Failed to get summary")


@app.get("/api/dependencies/sbom")
async def get_sbom():
    """Generate Software Bill of Materials (SBOM)"""
    try:
        from pathlib import Path
        project_root = Path("/app").parent.parent
        scanner = DependencyScanner(project_root)
        sbom = await scanner.generate_sbom()
        return sbom
    except Exception as e:
        logger.error(f"SBOM generation failed: {e}")
        raise HTTPException(status_code=500, detail="SBOM generation failed")


@app.get("/api/dependencies/vulnerabilities")
async def get_vulnerabilities(
    severity: Optional[str] = None,
    package_type: Optional[str] = None
):
    """Get all vulnerabilities, optionally filtered"""
    try:
        from pathlib import Path
        project_root = Path("/app").parent.parent
        scanner = DependencyScanner(project_root)
        health_reports = await scanner.scan_all_packages()
        
        vulnerabilities = []
        for path, health in health_reports.items():
            for dep in health.dependencies:
                # Filter by package type if specified
                if package_type and dep.package_type.value != package_type:
                    continue
                
                for vuln in dep.vulnerabilities:
                    # Filter by severity if specified
                    if severity and vuln.severity.value != severity:
                        continue
                    
                    vulnerabilities.append({
                        "package": dep.name,
                        "version": dep.version,
                        "package_type": dep.package_type.value,
                        "severity": vuln.severity.value,
                        "cve": vuln.cve,
                        "description": vuln.description,
                        "fixed_version": vuln.fixed_version,
                        "package_path": path
                    })
        
        # Sort by severity (critical first)
        severity_order = {"critical": 0, "high": 1, "moderate": 2, "low": 3, "info": 4}
        vulnerabilities.sort(key=lambda x: severity_order.get(x["severity"], 5))
        
        return {"vulnerabilities": vulnerabilities}
    except Exception as e:
        logger.error(f"Failed to get vulnerabilities: {e}")
        raise HTTPException(status_code=500, detail="Failed to get vulnerabilities")


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket for real-time updates - no internal details exposed"""
    await websocket.accept()
    active_connections.append(websocket)
    logger.info(f"Client connected. Active connections: {len(active_connections)}")
    
    try:
        while True:
            # Send abstracted system status
            health = await gateway.get_system_health()
            metrics = await gateway.get_system_metrics()
            
            data = {
                "health": health.dict(),
                "metrics": metrics.dict(),
                "timestamp": datetime.utcnow().isoformat()
            }
            
            await websocket.send_json(data)
            await asyncio.sleep(2)  # Update every 2 seconds
            
    except WebSocketDisconnect:
        active_connections.remove(websocket)
        logger.info(f"Client disconnected. Active connections: {len(active_connections)}")
    except Exception as e:
        logger.error(f"WebSocket error: {e}")
        if websocket in active_connections:
            active_connections.remove(websocket)


# Serve PWA files explicitly before catch-all route
@app.get("/manifest.webmanifest")
async def serve_manifest():
    """Serve PWA manifest"""
    manifest_path = os.path.join(STATIC_DIR, "manifest.webmanifest")
    if os.path.exists(manifest_path):
        return FileResponse(manifest_path, media_type="application/manifest+json")
    raise HTTPException(status_code=404, detail="Manifest not found")


@app.get("/registerSW.js")
async def serve_register_sw():
    """Serve service worker registration"""
    sw_path = os.path.join(STATIC_DIR, "registerSW.js")
    if os.path.exists(sw_path):
        return FileResponse(sw_path, media_type="application/javascript")
    raise HTTPException(status_code=404, detail="Service worker registration not found")


@app.get("/sw.js")
async def serve_sw():
    """Serve service worker"""
    sw_path = os.path.join(STATIC_DIR, "sw.js")
    if os.path.exists(sw_path):
        return FileResponse(sw_path, media_type="application/javascript")
    raise HTTPException(status_code=404, detail="Service worker not found")


@app.get("/workbox-{filename}")
async def serve_workbox(filename: str):
    """Serve workbox files"""
    workbox_path = os.path.join(STATIC_DIR, f"workbox-{filename}")
    if os.path.exists(workbox_path):
        return FileResponse(workbox_path, media_type="application/javascript")
    raise HTTPException(status_code=404, detail="Workbox file not found")


# Catch-all route for React Router
@app.get("/{path:path}")
async def serve_react_app(path: str):
    """Serve React app for any route - SPA support"""
    index_path = os.path.join(STATIC_DIR, "index.html")
    try:
        with open(index_path, "r") as f:
            return HTMLResponse(f.read())
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail="Application not found")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=int(os.getenv("PORT", 9000)),
        log_level="info"
    )