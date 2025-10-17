"""
Sutra Control Center - Web-based lifecycle management and monitoring.
"""

import asyncio
import logging
import os
import subprocess
import sys
import time
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional

import psutil
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


class ComponentState(str, Enum):
    """Component lifecycle states."""

    STOPPED = "stopped"
    STARTING = "starting"
    RUNNING = "running"
    STOPPING = "stopping"
    ERROR = "error"


class ComponentStatus(BaseModel):
    """Status of a system component."""

    name: str
    state: ComponentState
    pid: Optional[int] = None
    uptime: Optional[float] = None
    cpu_percent: Optional[float] = None
    memory_mb: Optional[float] = None
    error: Optional[str] = None
    last_updated: str


class SystemMetrics(BaseModel):
    """System-wide metrics."""

    timestamp: str
    total_concepts: int = 0
    total_associations: int = 0
    storage_size_mb: float = 0.0
    queries_per_second: float = 0.0
    avg_latency_ms: float = 0.0
    cpu_percent: float = 0.0
    memory_percent: float = 0.0


class LifecycleManager:
    """Manages lifecycle of Sutra AI components."""

    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.components: Dict[str, ComponentStatus] = {
            "storage": ComponentStatus(
                name="Storage Engine",
                state=ComponentState.STOPPED,
                last_updated=datetime.utcnow().isoformat(),
            ),
            "api": ComponentStatus(
                name="Sutra API",
                state=ComponentState.STOPPED,
                last_updated=datetime.utcnow().isoformat(),
            ),
        }
        self.processes: Dict[str, subprocess.Popen] = {}
        self.start_times: Dict[str, float] = {}

    async def start_component(self, component: str) -> bool:
        """Start a system component."""
        try:
            self.components[component].state = ComponentState.STARTING
            self.components[component].last_updated = datetime.utcnow().isoformat()

            if component == "api":
                # Start FastAPI server
                api_path = self.project_root / "packages" / "sutra-api"
                process = subprocess.Popen(
                    [
                        sys.executable,
                        "-m",
                        "uvicorn",
                        "sutra_api.main:app",
                        "--host",
                        "0.0.0.0",
                        "--port",
                        "8000",
                    ],
                    cwd=str(api_path),
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                self.processes[component] = process
                self.start_times[component] = time.time()

                # Wait a bit for startup
                await asyncio.sleep(2)

                if process.poll() is None:
                    self.components[component].state = ComponentState.RUNNING
                    self.components[component].pid = process.pid
                    logger.info(f"Started {component} with PID {process.pid}")
                    return True
                else:
                    self.components[component].state = ComponentState.ERROR
                    self.components[component].error = "Failed to start"
                    return False

            elif component == "storage":
                # Storage is embedded, just mark as running
                self.components[component].state = ComponentState.RUNNING
                self.start_times[component] = time.time()
                return True

        except Exception as e:
            logger.error(f"Failed to start {component}: {e}")
            self.components[component].state = ComponentState.ERROR
            self.components[component].error = str(e)
            return False

        return False

    async def stop_component(self, component: str) -> bool:
        """Stop a system component."""
        try:
            self.components[component].state = ComponentState.STOPPING
            self.components[component].last_updated = datetime.utcnow().isoformat()

            if component in self.processes:
                process = self.processes[component]
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                del self.processes[component]

            if component in self.start_times:
                del self.start_times[component]

            self.components[component].state = ComponentState.STOPPED
            self.components[component].pid = None
            self.components[component].uptime = None
            logger.info(f"Stopped {component}")
            return True

        except Exception as e:
            logger.error(f"Failed to stop {component}: {e}")
            self.components[component].error = str(e)
            return False

    async def restart_component(self, component: str) -> bool:
        """Restart a component."""
        await self.stop_component(component)
        await asyncio.sleep(1)
        return await self.start_component(component)

    def update_status(self):
        """Update component status with current metrics."""
        for name, component in self.components.items():
            if component.pid:
                try:
                    proc = psutil.Process(component.pid)
                    component.cpu_percent = proc.cpu_percent(interval=0.1)
                    component.memory_mb = proc.memory_info().rss / 1024 / 1024

                    if name in self.start_times:
                        component.uptime = time.time() - self.start_times[name]

                except psutil.NoSuchProcess:
                    component.state = ComponentState.ERROR
                    component.error = "Process died unexpectedly"
                    component.pid = None

            component.last_updated = datetime.utcnow().isoformat()

    def get_system_metrics(self) -> SystemMetrics:
        """Get current system metrics."""
        storage_path = Path(os.getenv("SUTRA_STORAGE_PATH", "./knowledge"))
        storage_size = 0.0
        if storage_path.exists():
            storage_size = sum(
                f.stat().st_size for f in storage_path.rglob("*") if f.is_file()
            ) / (1024 * 1024)

        return SystemMetrics(
            timestamp=datetime.utcnow().isoformat(),
            storage_size_mb=round(storage_size, 2),
            cpu_percent=round(psutil.cpu_percent(interval=0.1), 2),
            memory_percent=round(psutil.virtual_memory().percent, 2),
        )


# Initialize FastAPI app
app = FastAPI(title="Sutra Control Center", version="0.1.0")

# Mount static files
static_path = Path(__file__).parent.parent / "static"
if static_path.exists():
    app.mount("/static", StaticFiles(directory=str(static_path)), name="static")

# Initialize lifecycle manager
project_root = Path(__file__).parent.parent.parent.parent
manager = LifecycleManager(str(project_root))

# WebSocket connections for real-time updates
active_connections: List[WebSocket] = []


@app.get("/", response_class=HTMLResponse)
async def read_root():
    """Serve the control center dashboard."""
    template_path = Path(__file__).parent.parent / "templates" / "dashboard.html"
    if template_path.exists():
        return template_path.read_text()
    return "<h1>Sutra Control Center</h1><p>Dashboard template not found</p>"


@app.get("/api/status")
async def get_status():
    """Get current status of all components."""
    manager.update_status()
    return {
        "components": manager.components,
        "metrics": manager.get_system_metrics(),
    }


@app.post("/api/components/{component}/start")
async def start_component(component: str):
    """Start a component."""
    if component not in manager.components:
        return {"success": False, "error": "Unknown component"}

    success = await manager.start_component(component)
    return {"success": success, "component": manager.components[component]}


@app.post("/api/components/{component}/stop")
async def stop_component(component: str):
    """Stop a component."""
    if component not in manager.components:
        return {"success": False, "error": "Unknown component"}

    success = await manager.stop_component(component)
    return {"success": success, "component": manager.components[component]}


@app.post("/api/components/{component}/restart")
async def restart_component(component: str):
    """Restart a component."""
    if component not in manager.components:
        return {"success": False, "error": "Unknown component"}

    success = await manager.restart_component(component)
    return {"success": success, "component": manager.components[component]}


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket for real-time status updates."""
    await websocket.accept()
    active_connections.append(websocket)
    logger.info(f"Client connected. Total connections: {len(active_connections)}")

    try:
        while True:
            # Send status updates every 2 seconds
            manager.update_status()
            data = {
                "components": {
                    k: v.model_dump() for k, v in manager.components.items()
                },
                "metrics": manager.get_system_metrics().model_dump(),
            }
            await websocket.send_json(data)
            await asyncio.sleep(2)

    except WebSocketDisconnect:
        active_connections.remove(websocket)
        logger.info(f"Client disconnected. Total connections: {len(active_connections)}")
    except Exception as e:
        logger.error(f"WebSocket error: {e}")
        if websocket in active_connections:
            active_connections.remove(websocket)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=9000)
