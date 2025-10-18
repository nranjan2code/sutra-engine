# Multi-stage secure build for Sutra Control Center
# Client -> Python Gateway -> gRPC -> Internal Services

# Stage 1: Build React frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /build

# Copy React app files
COPY packages/sutra-control/package*.json ./
COPY packages/sutra-control/tsconfig*.json ./
COPY packages/sutra-control/vite.config.ts ./
COPY packages/sutra-control/index.html ./
COPY packages/sutra-control/src/ ./src/

# Install dependencies and build
RUN npm ci --silent
RUN npm run build

# Stage 2: Python backend with built React app
FROM python:3.11-slim

# Install system dependencies  
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    gcc \
    g++ \
    && useradd --create-home --uid 1000 sutra \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create requirements file for better caching
COPY packages/sutra-control/backend/requirements.txt /app/requirements.txt

# Install Python dependencies (cached layer)
RUN pip install --no-cache-dir -r requirements.txt

# Copy built React app from frontend builder
COPY --from=frontend-builder --chown=sutra:sutra /build/dist /app/build

# Copy Python backend
COPY --chown=sutra:sutra packages/sutra-control/backend/ /app/backend/

# No gRPC client needed; backend talks to REST APIs

# Switch to non-root user
USER sutra

# Set Python path
ENV PYTHONPATH=/app:$PYTHONPATH \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

# Security: Remove docs in production
ENV ENVIRONMENT=production

# Expose port (internal to Docker network only)
EXPOSE 9000

# Health check - abstract endpoint only
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:9000/health || exit 1

# Run secure gateway with Grid integration
CMD ["python", "backend/main.py"]
