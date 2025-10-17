# Development Guide

## Quick Start (Full Stack)

### Option 1: Using npm script (Recommended)

```bash
# Install dependencies first (one-time)
cd packages/sutra-client && npm install && cd ../..

# Install concurrently (one-time)
npm install

# Start both API and Client together
npm run dev
```

This will launch:
- **API**: http://localhost:8000
- **Client**: http://localhost:3000

Press **Ctrl+C** to stop both servers.

### Option 2: Using bash script

```bash
# Make executable (one-time)
chmod +x dev.sh

# Run
./dev.sh
```

## Individual Services

### API Only
```bash
cd packages/sutra-api
python -m sutra_api.main
```

### Client Only
```bash
cd packages/sutra-client
npm run dev
```

## Available Commands

From the **root directory**:

```bash
npm run dev              # Start both API + Client
npm run dev:api          # Start API only
npm run dev:client       # Start Client only
npm run install:client   # Install client dependencies
npm run build:client     # Build client for production
```

## Development Workflow

1. **First time setup**:
   ```bash
   npm install                  # Root dependencies
   npm run install:client       # Client dependencies
   ```

2. **Daily development**:
   ```bash
   npm run dev                  # Start everything
   ```

3. **The client auto-opens** at http://localhost:3000
4. **Edit code** - both API and Client have hot reload
5. **Press Ctrl+C** - stops everything cleanly

## Logs

When using `npm run dev`, logs are color-coded:
- 🔵 **Cyan** = API logs
- 🟣 **Magenta** = Client logs

When using `./dev.sh`, logs are written to:
- `/tmp/sutra-api.log`
- `/tmp/sutra-client.log`

## Troubleshooting

### Port Already in Use

**API (8000)**:
```bash
lsof -ti:8000 | xargs kill -9
```

**Client (3000)**:
```bash
lsof -ti:3000 | xargs kill -9
```

### API Connection Errors

If the client shows "ECONNREFUSED":
1. Make sure API is running: `curl http://localhost:8000/health`
2. Check API logs for errors
3. Verify Python environment is activated

### Client Not Loading

1. Check that dependencies are installed: `npm run install:client`
2. Clear cache: `rm -rf packages/sutra-client/node_modules/.vite`
3. Restart: `npm run dev`

## Production Build

```bash
# Build client
npm run build:client

# Output will be in packages/sutra-client/dist/
```

## IDE Setup

### VS Code
Recommended extensions:
- ESLint
- Prettier
- Python
- Volar (for Vue/React)

### Multiple Terminals
If you prefer separate terminals:
1. **Terminal 1**: `npm run dev:api`
2. **Terminal 2**: `npm run dev:client`

## Environment Variables

### API (.env)
```bash
export SUTRA_STORAGE_PATH="./knowledge"
export SUTRA_API_PORT="8000"
```

### Client (.env.local)
```bash
VITE_API_URL=http://localhost:8000
```

## Next Steps

- See `packages/sutra-client/README.md` for client-specific docs
- See `packages/sutra-api/README.md` for API-specific docs
- Check `WARP.md` for architecture overview
