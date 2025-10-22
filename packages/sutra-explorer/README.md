# 🔍 Sutra Storage Explorer

**Standalone application for deep exploration and visualization of Sutra knowledge graph storage files.**

A modern, interactive tool for browsing, analyzing, and visualizing `storage.dat` files independently from the main Sutra system.

---

## ✨ Features

### **Core Capabilities**
- 📊 **Storage Statistics** - File size, concept count, edge count, vector dimensions
- 🔎 **Full-Text Search** - Search concepts by content with substring matching
- 🌐 **Graph Visualization** - Interactive force-directed graph with D3.js
- 🗺️ **Path Finding** - BFS path discovery between any two concepts
- 🔗 **Association Browser** - Explore edges with confidence scores
- 📍 **Neighborhood Explorer** - N-hop neighborhood visualization
- 📏 **Vector Similarity** - Cosine similarity between concept embeddings
- 🎨 **Modern UI** - Dark theme with Material Design 3

### **Technical Features**
- ✅ **Read-Only** - Safe exploration without modification risk
- ✅ **Independent** - No dependencies on running Sutra services
- ✅ **Fast** - Pure Rust binary parser with zero-copy reads
- ✅ **Complete** - Parses all storage.dat v2 format sections
- ✅ **Portable** - Docker containerized for any environment

---

## 🏗️ Architecture

```
sutra-explorer/
├── src/                      # Rust library (read-only storage parser)
│   └── lib.rs               # Binary format parser + graph queries
├── backend/                  # FastAPI REST API
│   └── main.py              # Exploration endpoints
├── frontend/                 # React + TypeScript UI
│   ├── src/
│   │   ├── App.tsx          # Main application
│   │   ├── components/       # Reusable UI components
│   │   └── pages/           # Dashboard, Browser, Graph, Search
│   └── package.json
├── Cargo.toml               # Rust dependencies
├── Dockerfile               # Multi-stage build
├── docker-compose.yml       # Standalone deployment
└── README.md
```

**Stack:**
- **Parser**: Rust (anyhow, serde, hex)
- **Backend**: Python 3.11 + FastAPI + uvicorn
- **Frontend**: React 18 + TypeScript + Material-UI + D3.js + react-force-graph
- **Deployment**: Docker multi-stage builds

---

## 🚀 Quick Start

### **Option 1: Docker (Recommended)**

```bash
# 1. Build the image
cd packages/sutra-explorer
docker build -t sutra-explorer:latest .

# 2. Run with your storage file
docker run -d \
  -p 8100:8100 \
  -p 3000:3000 \
  -v /path/to/your/storage.dat:/data/storage.dat \
  -e STORAGE_PATH=/data/storage.dat \
  sutra-explorer:latest

# 3. Open browser
open http://localhost:3000
```

### **Option 2: Docker Compose**

```bash
# 1. Edit docker-compose.yml to set your storage path
# 2. Start services
docker-compose up -d

# 3. Check logs
docker-compose logs -f

# 4. Access UI
open http://localhost:3000
```

### **Option 3: Local Development**

```bash
# 1. Build Rust library
cargo build --release

# 2. Install Python backend
cd backend
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# 3. Install frontend dependencies
cd ../frontend
npm install

# 4. Run backend (terminal 1)
cd ../backend
export STORAGE_PATH=/path/to/storage.dat
uvicorn main:app --host 0.0.0.0 --port 8100

# 5. Run frontend (terminal 2)
cd ../frontend
npm run dev

# 6. Open browser
open http://localhost:3000
```

---

## 📖 API Documentation

### **REST API Endpoints**

Base URL: `http://localhost:8100`

#### **Storage Management**
- `GET /health` - Health check and storage status
- `POST /load` - Load a storage.dat file
  ```json
  {"path": "/path/to/storage.dat"}
  ```
- `GET /stats` - Get storage statistics

#### **Concept Operations**
- `GET /concepts?limit=100` - List concept IDs (paginated)
- `GET /concepts/{concept_id}` - Get concept details
- `POST /search` - Search concepts by content
  ```json
  {"query": "search term", "limit": 100}
  ```

#### **Graph Operations**
- `GET /associations/{concept_id}` - Get concept associations
- `POST /path` - Find path between concepts
  ```json
  {"start_id": "abc...", "end_id": "def...", "max_depth": 6}
  ```
- `POST /neighborhood` - Get N-hop neighborhood
  ```json
  {"id": "abc...", "depth": 2}
  ```

#### **Vector Operations**
- `POST /similarity` - Calculate vector similarity
  ```json
  {"id1": "abc...", "id2": "def..."}
  ```

**Interactive API Docs:** http://localhost:8100/docs

---

## 🎨 UI Features

### **Dashboard**
- Storage statistics overview
- File information (size, version, timestamp)
- Concept/edge/vector counts
- Quick actions

### **Concept Browser**
- Paginated list of all concepts
- Content preview
- Metadata display (strength, confidence, access count)
- Neighbor count badges
- Click to explore associations

### **Graph Explorer**
- Interactive force-directed graph visualization
- Node size = confidence
- Edge thickness = association strength
- Click to select concept
- Drag to reposition
- Zoom and pan
- Color-coded by vector dimension

### **Search**
- Full-text substring search
- Real-time results
- Highlighted matches
- Result count
- Click to view details

### **Path Finder**
- Find shortest path between two concepts
- Adjustable max depth
- Path visualization
- Hop count display
- Confidence aggregation

---

## 🔧 Configuration

### **Environment Variables**

```bash
# Backend
STORAGE_PATH=/path/to/storage.dat  # Auto-load on startup
API_HOST=0.0.0.0
API_PORT=8100

# Frontend
VITE_API_URL=http://localhost:8100  # Backend API URL
```

### **Docker Compose Configuration**

```yaml
services:
  explorer:
    build: .
    ports:
      - "8100:8100"  # Backend API
      - "3000:3000"  # Frontend UI
    volumes:
      - /your/storage/path:/data/storage.dat:ro  # Read-only mount
    environment:
      - STORAGE_PATH=/data/storage.dat
```

---

## 📊 Storage Format Support

**Supported:** SUTRA binary format v2

```
[Header: 64 bytes]
  - Magic: "SUTRADAT" (8 bytes)
  - Version: 2 (4 bytes)
  - Concept count (4 bytes)
  - Edge count (4 bytes)
  - Vector count (4 bytes)
  - Timestamp (8 bytes)
  - Reserved (32 bytes)

[Concepts Section]
  - Concept header (36 bytes each)
  - Variable-length content

[Edges Section]
  - Edge data (36 bytes each)
  - Source ID, Target ID, Confidence

[Vectors Section]
  - Vector header (20 bytes each)
  - Variable-length f32 components
```

---

## 🧪 Example Queries

### **Python API Client**

```python
import requests

BASE_URL = "http://localhost:8100"

# Load storage
response = requests.post(f"{BASE_URL}/load", json={"path": "/data/storage.dat"})
print(response.json())

# Get stats
stats = requests.get(f"{BASE_URL}/stats").json()
print(f"Total concepts: {stats['total_concepts']}")

# Search
results = requests.post(f"{BASE_URL}/search", json={"query": "Eiffel", "limit": 10}).json()
for concept in results:
    print(f"{concept['id']}: {concept['content'][:50]}...")

# Find path
path = requests.post(f"{BASE_URL}/path", json={
    "start_id": "abc123...",
    "end_id": "def456...",
    "max_depth": 6
}).json()
if path:
    print(f"Path length: {path['length']} hops")
    print(f"Path: {' -> '.join(path['concepts'])}")
```

### **cURL Examples**

```bash
# Health check
curl http://localhost:8100/health

# Load storage
curl -X POST http://localhost:8100/load \
  -H "Content-Type: application/json" \
  -d '{"path": "/data/storage.dat"}'

# Get stats
curl http://localhost:8100/stats

# Search
curl -X POST http://localhost:8100/search \
  -H "Content-Type: application/json" \
  -d '{"query": "Eiffel Tower", "limit": 10}'

# Get neighborhood
curl -X POST http://localhost:8100/neighborhood \
  -H "Content-Type: application/json" \
  -d '{"id": "abc123...", "depth": 2}'
```

---

## 🛠️ Development

### **Build Rust Library**
```bash
cargo build --release
cargo test
```

### **Run Backend Tests**
```bash
cd backend
pytest tests/
```

### **Run Frontend Dev Server**
```bash
cd frontend
npm run dev
```

### **Linting**
```bash
# Backend
cd backend
flake8 main.py

# Frontend
cd frontend
npm run lint
```

---

## 🐛 Troubleshooting

### **"No storage loaded" Error**
- Ensure `STORAGE_PATH` environment variable is set
- Verify file exists and is readable
- Check file is v2 format (magic bytes: `SUTRADAT`)

### **Port Already in Use**
```bash
# Change ports in docker-compose.yml or use:
docker-compose down
lsof -ti:8100 | xargs kill -9  # Kill process using port
```

### **Frontend Can't Connect to Backend**
- Check `VITE_API_URL` environment variable
- Verify backend is running: `curl http://localhost:8100/health`
- Check CORS settings in backend/main.py

### **Large Storage Files Slow to Load**
- Loading happens once at startup
- For >100MB files, increase Docker memory limits
- Consider adding pagination/lazy loading

---

## 📚 Related Documentation

- **WARP.md** - Main Sutra project documentation
- **Storage Format** - See `packages/sutra-storage/docs/`
- **Association Creation** - See architecture documentation

---

## 🎯 Roadmap

**Planned Features:**
- [ ] Export to GraphML/GEXF formats
- [ ] Advanced filtering (by confidence, date range)
- [ ] Community detection algorithms
- [ ] Heatmaps for access patterns
- [ ] Comparison mode (diff two storage files)
- [ ] Bulk operations (merge, extract subgraphs)
- [ ] Natural language query interface
- [ ] Real-time updates (watch mode)

---

## 📄 License

Part of the Sutra AI project. See main repository for license details.

---

## 🤝 Contributing

This is a standalone exploration tool. Contributions welcome:

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open Pull Request

---

## 📞 Support

For issues specific to Sutra Storage Explorer:
- Open an issue in the main Sutra repository
- Tag with `component: explorer`
- Include storage file version and error logs

---

**Built with ❤️ for the Sutra AI community**
