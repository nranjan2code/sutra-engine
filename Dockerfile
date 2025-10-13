FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements first for better caching
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Install additional distributed dependencies
RUN pip install fastapi uvicorn httpx aiofiles pydantic

# Copy the entire project
COPY . .

# Create directories for persistence
RUN mkdir -p /app/biological_workspace /app/english_biological_workspace

# Expose port for API
EXPOSE 8000

# Default command - can be overridden
CMD ["python", "biological_service.py", "--api", "--host", "0.0.0.0", "--port", "8000"]