"""
Sutra Storage Explorer - FastAPI Backend

Provides REST API for exploring Sutra storage.dat files.
"""

import os
from pathlib import Path
from typing import List, Optional

from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import sutra_explorer

app = FastAPI(
    title="Sutra Storage Explorer API",
    description="REST API for exploring Sutra knowledge graph storage files",
    version="1.0.0",
)

# CORS middleware for React frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global explorer instance (loaded on demand)
explorer = None
storage_path = None


# Response models
class StorageStats(BaseModel):
    version: int
    total_concepts: int
    total_edges: int
    total_vectors: int
    file_size_mb: float
    timestamp: int


class Concept(BaseModel):
    id: str
    content: str
    strength: float
    confidence: float
    access_count: int
    created: int
    vector_dimension: Optional[int]
    neighbors: List[str]


class Association(BaseModel):
    source_id: str
    target_id: str
    confidence: float


class GraphPath(BaseModel):
    concepts: List[str]
    length: int
    confidence: float


class NeighborhoodGraph(BaseModel):
    nodes: List[Concept]
    edges: List[Association]


class LoadStorageRequest(BaseModel):
    path: str


class SearchRequest(BaseModel):
    query: str
    limit: Optional[int] = 100


class PathRequest(BaseModel):
    start_id: str
    end_id: str
    max_depth: int = 6


class NeighborhoodRequest(BaseModel):
    id: str
    depth: int = 2


class SimilarityRequest(BaseModel):
    id1: str
    id2: str


# API Endpoints

@app.get("/health")
def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "storage_loaded": explorer is not None,
        "storage_path": str(storage_path) if storage_path else None,
    }


@app.post("/load")
def load_storage(request: LoadStorageRequest):
    """Load a storage.dat file"""
    global explorer, storage_path
    
    path = Path(request.path)
    if not path.exists():
        raise HTTPException(status_code=404, detail=f"Storage file not found: {request.path}")
    
    try:
        explorer = sutra_explorer.StorageExplorer.load(str(path))
        storage_path = path
        return {
            "success": True,
            "message": f"Loaded storage from {path}",
            "stats": explorer.stats(),
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to load storage: {str(e)}")


@app.get("/stats", response_model=StorageStats)
def get_stats():
    """Get storage statistics"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded. Use POST /load first.")
    
    return explorer.stats()


@app.get("/concepts", response_model=List[str])
def list_concepts(limit: Optional[int] = Query(100, ge=1, le=10000)):
    """List all concept IDs"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    return explorer.list_concepts(limit)


@app.get("/concepts/{concept_id}", response_model=Concept)
def get_concept(concept_id: str):
    """Get concept details by ID"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    try:
        return explorer.get_concept(concept_id)
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.post("/search", response_model=List[Concept])
def search_content(request: SearchRequest):
    """Search concepts by content substring"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    return explorer.search_content(request.query, request.limit)


@app.get("/associations/{concept_id}", response_model=List[Association])
def get_associations(concept_id: str):
    """Get associations for a concept"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    try:
        return explorer.get_associations(concept_id)
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.post("/path", response_model=Optional[GraphPath])
def find_path(request: PathRequest):
    """Find path between two concepts"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    try:
        result = explorer.find_path(request.start_id, request.end_id, request.max_depth)
        return result
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/neighborhood", response_model=NeighborhoodGraph)
def get_neighborhood(request: NeighborhoodRequest):
    """Get neighborhood graph (N-hop neighbors)"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    try:
        return explorer.get_neighborhood(request.id, request.depth)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/similarity", response_model=dict)
def vector_similarity(request: SimilarityRequest):
    """Calculate vector similarity between two concepts"""
    if explorer is None:
        raise HTTPException(status_code=400, detail="No storage loaded")
    
    try:
        similarity = explorer.vector_similarity(request.id1, request.id2)
        return {
            "id1": request.id1,
            "id2": request.id2,
            "similarity": similarity,
        }
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


# Auto-load storage from environment variable
@app.on_event("startup")
async def startup_event():
    """Auto-load storage on startup if STORAGE_PATH is set"""
    global explorer, storage_path
    
    env_path = os.getenv("STORAGE_PATH")
    if env_path:
        path = Path(env_path)
        if path.exists():
            try:
                explorer = sutra_explorer.StorageExplorer.load(str(path))
                storage_path = path
                print(f"✅ Auto-loaded storage from {path}")
                print(f"📊 Stats: {explorer.stats()}")
            except Exception as e:
                print(f"⚠️  Failed to auto-load storage: {e}")
        else:
            print(f"⚠️  STORAGE_PATH set but file not found: {env_path}")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8100, log_level="info")
