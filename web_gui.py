#!/usr/bin/env python3
"""
🌐 BIOLOGICAL INTELLIGENCE WEB GUI
FastAPI-based web interface for remote control and monitoring.

Provides all functionality of the terminal GUI through a web browser:
- Service control (start/stop/restart)
- Real-time monitoring dashboard
- Knowledge feeding interface
- Curriculum management
- 7-agent swarm visualization
- Pi hardware monitoring
"""

import asyncio
import json
import time
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List
import logging
import os

from fastapi import FastAPI, WebSocket, Request, Form
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.middleware.cors import CORSMiddleware
import logging

try:
    import httpx
    HAS_HTTPX = True
except ImportError:
    HAS_HTTPX = False

# Import our components
try:
    from pi_config import PiConfig
    PI_MODE = True
except:
    PI_MODE = False


# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(
    title="Biological Intelligence Control Center",
    description="Web-based interface for biological intelligence management",
    version="1.0.0"
)

# Setup static files and templates
app.mount("/static", StaticFiles(directory="web_static"), name="static")
templates = Jinja2Templates(directory="web_templates")

# Configure CORS for API access from browser
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allow all origins for development
    allow_credentials=True,
    allow_methods=["GET", "POST"],
    allow_headers=["*"],
)

# Add Python builtin functions to template globals
def format_number(num, decimals=0):
    """Format numbers with K/M suffixes."""
    if num >= 1000000:
        return f"{num/1000000:.1f}M"
    elif num >= 1000:
        return f"{num/1000:.1f}K"
    else:
        return str(int(num))

templates.env.globals.update({
    'min': min,
    'max': max,
    'round': round,
    'int': int,
    'float': float,
    'formatNumber': format_number
})

# Global state
current_mode = "general"
websocket_connections = []


class WebGUIManager:
    """Manager for web-based biological intelligence control via API."""
    
    def __init__(self):
        self.core_service_url = os.getenv("CORE_SERVICE_URL", "http://localhost:8000")
        self.workspace = Path("./biological_workspace")
        self.english_workspace = Path("./english_biological_workspace")
        
        # Use Pi config if available
        if PI_MODE:
            self.workspace = Path(PiConfig.HDD_WORKSPACE) / "biological_workspace"
            self.english_workspace = Path(PiConfig.HDD_WORKSPACE) / "english_biological_workspace"
    
    def get_workspace(self) -> Path:
        """Get current workspace based on mode."""
        return self.english_workspace if current_mode == "english" else self.workspace
    
    async def check_core_service_health(self) -> bool:
        """Check if core service is healthy."""
        if not HAS_HTTPX:
            logger.warning("httpx not available for health check")
            return False
            
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                response = await client.get(f"{self.core_service_url}/api/health")
                if response.status_code == 200:
                    data = response.json()
                    return data.get("status") == "alive"
                return False
        except Exception as e:
            logger.warning(f"Health check failed: {e}")
            return False
    
    async def get_core_service_status(self) -> Dict[str, Any]:
        """Get status from core service API."""
        if not HAS_HTTPX:
            return {}
            
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                response = await client.get(f"{self.core_service_url}/api/status")
                if response.status_code == 200:
                    return response.json()
        except:
            pass
        return {}
    
    async def get_system_status(self) -> Dict[str, Any]:
        """Get comprehensive system status via API."""
        # Check if core service is running
        service_running = await self.check_core_service_health()
        
        # Get status from core service if available
        core_status = await self.get_core_service_status()
        
        # Extract data from core service response
        service_state = core_status.get('service_state', {})
        metrics = {
            'total_concepts': core_status.get('total_concepts', 0),
            'total_associations': core_status.get('total_associations', 0),
            'total_training_cycles': core_status.get('training_cycles', 0),
            'total_dreams': core_status.get('dreams_completed', 0),
            'consciousness_score': core_status.get('consciousness_score', 0),
            'emergence_factor': core_status.get('emergence_factor', 1),
            'uptime_seconds': core_status.get('uptime_seconds', 0),
            'last_update': core_status.get('timestamp', datetime.now().isoformat())
        }
        
        # Process info - show Docker container info instead
        process_info = {
            'pid': 'Docker Container',
            'memory_mb': 'Container Mode',
            'cpu_percent': 'Distributed',
            'start_time': 'Container Runtime'
        }
        
        # Pi-specific hardware info
        hardware_info = {}
        if PI_MODE:
            try:
                pi_status = PiConfig.get_system_status()
                hardware_info = {
                    'cpu_temp': pi_status['hardware']['cpu_temp'],
                    'memory_info': pi_status['memory'],
                    'disk_info': pi_status['disk'],
                    'thermal_status': pi_status['thermal'],
                    'optimal_batch_size': pi_status['performance']['optimal_batch_size']
                }
            except:
                pass
        
        return {
            'service_running': service_running,
            'service_state': service_state,
            'metrics': metrics,
            'process_info': process_info,
            'hardware_info': hardware_info,
            'mode': current_mode,
            'workspace': str(self.get_workspace()),
            'pi_mode': PI_MODE,
            'timestamp': datetime.now().isoformat()
        }
    
    def get_system_status_sync(self) -> Dict[str, Any]:
        """Synchronous wrapper for get_system_status."""
        import asyncio
        try:
            loop = asyncio.get_event_loop()
            return loop.run_until_complete(self.get_system_status())
        except:
            return asyncio.run(self.get_system_status())


# Global manager instance
web_manager = WebGUIManager()


# WebSocket manager for real-time updates
class ConnectionManager:
    def __init__(self):
        self.active_connections: List[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)

    def disconnect(self, websocket: WebSocket):
        self.active_connections.remove(websocket)

    async def broadcast(self, message: dict):
        for connection in self.active_connections:
            try:
                await connection.send_json(message)
            except:
                pass  # Connection might be closed

manager = ConnectionManager()


# Routes

@app.get("/", response_class=HTMLResponse)
async def dashboard(request: Request):
    """Main dashboard page - Material Design 3."""
    status = await web_manager.get_system_status()
    return templates.TemplateResponse("dashboard_md3.html", {
        "request": request,
        "status": status,
        "pi_mode": PI_MODE,
        "active_section": "dashboard"
    })

@app.get("/dashboard", response_class=HTMLResponse)
async def dashboard_redirect(request: Request):
    """Dashboard page redirect."""
    return await dashboard(request)

@app.get("/intelligence", response_class=HTMLResponse)
async def intelligence_page(request: Request):
    """Intelligence chat interface."""
    status = await web_manager.get_system_status()
    return templates.TemplateResponse("intelligence_md3.html", {
        "request": request,
        "status": status,
        "pi_mode": PI_MODE,
        "active_section": "intelligence"
    })

@app.get("/knowledge", response_class=HTMLResponse)
async def knowledge_page(request: Request):
    """Knowledge management interface."""
    status = await web_manager.get_system_status()
    return templates.TemplateResponse("knowledge_md3.html", {
        "request": request,
        "status": status,
        "pi_mode": PI_MODE,
        "active_section": "knowledge"
    })

@app.get("/health", response_class=HTMLResponse)
async def health_page(request: Request):
    """System health monitoring interface."""
    status = await web_manager.get_system_status()
    return templates.TemplateResponse("health_md3.html", {
        "request": request,
        "status": status,
        "pi_mode": PI_MODE,
        "active_section": "health"
    })

@app.get("/settings", response_class=HTMLResponse)
async def settings_page(request: Request):
    """Settings and configuration interface."""
    status = await web_manager.get_system_status()
    return templates.TemplateResponse("settings_md3.html", {
        "request": request,
        "status": status,
        "pi_mode": PI_MODE,
        "active_section": "settings"
    })


@app.get("/api/status")
async def get_status():
    """Get current system status API."""
    status = await web_manager.get_system_status()
    return JSONResponse(status)


@app.post("/api/service/start")
async def start_service_api(mode: str = Form(...)):
    """Start biological service API - Docker mode communicates with core service."""
    global current_mode
    current_mode = mode
    
    # In Docker architecture, the core service is always running
    # We just need to check if it's healthy
    try:
        success = await web_manager.check_core_service_health()
        message = f"Core service is {'healthy' if success else 'not responding'} - {mode} mode selected"
        
        await manager.broadcast({
            "type": "service_status",
            "message": message,
            "success": success
        })
        return {"success": success, "mode": mode, "message": message}
    except Exception as e:
        logger.error(f"Error checking service: {e}")
        return {"success": False, "error": str(e)}


@app.post("/api/service/stop")
async def stop_service_api():
    """Stop biological service API - In Docker mode, core service runs independently."""
    try:
        # In Docker architecture, we can't stop the core service from web GUI
        # The core service runs in its own container
        message = "Core service runs independently in Docker mode - use 'docker-compose down' to stop"
        
        await manager.broadcast({
            "type": "service_status", 
            "message": message,
            "success": False
        })
        return {"success": False, "message": message}
    except Exception as e:
        logger.error(f"Error: {e}")
        return {"success": False, "error": str(e)}


@app.post("/api/knowledge/feed")
async def feed_knowledge_api(knowledge: str = Form(...)):
    """Feed knowledge to the system via core service API."""
    status = await web_manager.get_system_status()
    if not status['service_running']:
        return {"success": False, "error": "Core service must be running"}
    
    if not HAS_HTTPX:
        return {"success": False, "error": "HTTP client not available"}
    
    try:
        # Feed knowledge directly to core service API
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{web_manager.core_service_url}/api/feed",
                json={"content": knowledge}
            )
            success = response.status_code == 200
            result_data = response.json() if success else {}
        
        await manager.broadcast({
            "type": "knowledge_fed",
            "message": f"Knowledge {'fed successfully' if success else 'failed to feed'}",
            "knowledge": knowledge[:100] + "..." if len(knowledge) > 100 else knowledge,
            "success": success
        })
        
        return {
            "success": success,
            "message": result_data.get("status", "Knowledge processed") if success else "Failed to feed knowledge"
        }
    except Exception as e:
        logger.error(f"Error feeding knowledge: {e}")
        return {"success": False, "error": str(e)}


@app.post("/api/query")
async def query_intelligence_api(question: str = Form(...), hops: int = Form(2), max_results: int = Form(10)):
    """Query the biological intelligence via core service API."""
    status = await web_manager.get_system_status()
    if not status['service_running']:
        return {"success": False, "error": "Core service must be running"}
    
    if not HAS_HTTPX:
        return {"success": False, "error": "HTTP client not available"}
    
    try:
        # Query biological intelligence directly via core service API
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                f"{web_manager.core_service_url}/api/query",
                json={
                    "query": question, 
                    "max_results": max_results, 
                    "hops": hops,
                    "alpha": 0.5
                }
            )
            success = response.status_code == 200
            result_data = response.json() if success else {}
        
        await manager.broadcast({
            "type": "query_completed",
            "message": f"Query {'completed successfully' if success else 'failed'}",
            "question": question[:100] + "..." if len(question) > 100 else question,
            "success": success
        })
        
        return {
            "success": success,
            "question": question,
            "results": result_data.get("results", []) if success else [],
            "consciousness_score": result_data.get("consciousness_score", 0) if success else 0,
            "processing_time": result_data.get("processing_time", 0) if success else 0,
            "message": "Query processed successfully" if success else "Failed to process query"
        }
    except Exception as e:
        logger.error(f"Error querying intelligence: {e}")
        return {"success": False, "error": str(e)}


@app.post("/api/curriculum/feed")
async def feed_curriculum_api():
    """Feed curriculum API."""
    status = await web_manager.get_system_status()
    if not status['service_running']:
        return {"success": False, "error": "Service must be running"}
    
    try:
        if current_mode == "english":
            # Start English curriculum feeder
            subprocess.Popen(["./english_feeder.sh"], cwd=Path.cwd())
            message = "English curriculum feeding started"
        else:
            message = "General curriculum feeding not implemented via web yet"
            
        await manager.broadcast({
            "type": "curriculum_feeding",
            "message": message,
            "mode": current_mode
        })
        
        return {"success": True, "message": message}
    except Exception as e:
        logger.error(f"Error feeding curriculum: {e}")
        return {"success": False, "error": str(e)}


@app.post("/api/mode/switch")
async def switch_mode_api(new_mode: str = Form(...)):
    """Switch between general and English modes."""
    global current_mode
    old_mode = current_mode
    current_mode = new_mode
    
    await manager.broadcast({
        "type": "mode_switch",
        "message": f"Switched from {old_mode} to {new_mode} mode",
        "old_mode": old_mode,
        "new_mode": new_mode
    })
    
    return {"success": True, "old_mode": old_mode, "new_mode": new_mode}


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates."""
    await manager.connect(websocket)
    try:
        while True:
            # Send periodic status updates
            status = await web_manager.get_system_status()
            await websocket.send_json({
                "type": "status_update",
                "data": status
            })
            await asyncio.sleep(5)  # Update every 5 seconds
    except:
        manager.disconnect(websocket)


# Background tasks for monitoring
async def system_monitor():
    """Background task for system monitoring."""
    while True:
        try:
            status = await web_manager.get_system_status()
            
            # Check for thermal issues on Pi
            if PI_MODE and status.get('hardware_info', {}).get('thermal_status', {}).get('emergency_shutdown'):
                await manager.broadcast({
                    "type": "emergency",
                    "message": "EMERGENCY: CPU temperature critical! Shutting down service!",
                    "temp": status['hardware_info']['thermal_status'].get('cpu_temp', 0)
                })
                # Emergency stop
                stop_service()
            
            await asyncio.sleep(30)  # Check every 30 seconds
        except Exception as e:
            logger.error(f"System monitor error: {e}")
            await asyncio.sleep(60)


@app.on_event("startup")
async def startup_event():
    """Start background tasks on startup."""
    asyncio.create_task(system_monitor())
    if PI_MODE:
        PiConfig.setup_directories()


# Create necessary directories for templates and static files
def setup_web_directories():
    """Setup web template and static directories."""
    Path("web_templates").mkdir(exist_ok=True)
    Path("web_static").mkdir(exist_ok=True)
    Path("web_static/css").mkdir(exist_ok=True)
    Path("web_static/js").mkdir(exist_ok=True)


if __name__ == "__main__":
    setup_web_directories()
    
    # Determine host and port - allow Docker container access
    import os
    host = os.getenv("WEB_HOST", PiConfig.WEB_HOST if PI_MODE else "0.0.0.0")
    port = int(os.getenv("WEB_PORT", PiConfig.WEB_PORT if PI_MODE else 8080))
    
    print(f"🌐 Starting Biological Intelligence Web GUI on {host}:{port}")
    print(f"🥧 Pi Mode: {'Enabled' if PI_MODE else 'Disabled'}")
    print(f"🔗 Access via: http://{host}:{port}")
    
    uvicorn.run(app, host=host, port=port, log_level="info")