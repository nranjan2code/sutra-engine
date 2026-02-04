# API Design and Integration Architecture

## 1. REST API Specification

### Core API Endpoints

```yaml
openapi: 3.0.3
info:
  title: Sutra-Embedder API
  version: 1.0.0
  description: |
    High-performance embedding generation API with hardware adaptation,
    multi-dimensional support, and production-grade optimizations.
  
  contact:
    name: Sutra-Embedder Team
    url: https://github.com/sutra-embedder
    email: api-support@sutra-embedder.com
    
  license:
    name: MIT
    url: https://opensource.org/licenses/MIT

servers:
  - url: https://api.sutra-embedder.com/v1
    description: Production server
  - url: https://staging-api.sutra-embedder.com/v1  
    description: Staging server

paths:
  /embeddings:
    post:
      summary: Generate embeddings for text input
      description: |
        Generate high-quality embeddings for one or more text inputs.
        Supports arbitrary dimensions (64D-4096D) with automatic model selection.
      
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/EmbeddingRequest'
            examples:
              single_text:
                summary: Single text embedding
                value:
                  texts: ["Machine learning revolutionizes data processing"]
                  dimensions: 384
                  model: "efficient"
              
              batch_texts:
                summary: Batch text embedding  
                value:
                  texts: [
                    "Natural language processing",
                    "Computer vision applications", 
                    "Recommendation systems"
                  ]
                  dimensions: 768
                  model: "high-quality"
                  
              custom_config:
                summary: Custom configuration
                value:
                  texts: ["Scientific research paper abstract"]
                  dimensions: 512
                  model: "custom"
                  config:
                    quantization: "int8"
                    use_fp16: true
                    batch_size: 16
      
      responses:
        '200':
          description: Embeddings generated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/EmbeddingResponse'
              examples:
                success_response:
                  summary: Successful embedding generation
                  value:
                    embeddings: [
                      [0.1234, -0.5678, 0.9012, "...384 dimensions"],
                      [0.2345, -0.6789, 0.0123, "...384 dimensions"]
                    ]
                    model_info:
                      model_id: "all-MiniLM-L6-v2"
                      dimensions: 384
                      quality_score: 56.26
                    
                    performance:
                      latency_ms: 13.69
                      throughput_emb_per_sec: 73.04
                    
                    metadata:
                      request_id: "req_abc123"
                      timestamp: "2025-11-13T10:30:00Z"
                      hardware_profile: "medium-system"
        
        '400':
          description: Bad request - invalid parameters
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
              examples:
                invalid_dimensions:
                  summary: Invalid dimension request
                  value:
                    error:
                      code: "INVALID_DIMENSIONS"
                      message: "Dimensions must be between 64 and 4096"
                      details:
                        requested_dimensions: 8000
                        max_supported: 4096
                
        '429':
          description: Rate limit exceeded
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
                
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
      
      security:
        - ApiKeyAuth: []
        - BearerAuth: []

  /embeddings/stream:
    post:
      summary: Stream embeddings for real-time processing
      description: |
        Generate embeddings with streaming response for real-time applications.
        Supports chunked processing and backpressure control.
      
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/StreamingRequest'
      
      responses:
        '200':
          description: Streaming embeddings
          content:
            application/x-ndjson:
              schema:
                $ref: '#/components/schemas/StreamingResponse'
            text/event-stream:
              schema:
                $ref: '#/components/schemas/ServerSentEvent'
      
      security:
        - ApiKeyAuth: []

  /models:
    get:
      summary: List available embedding models
      description: |
        Get information about all available embedding models,
        including dimensions, quality scores, and hardware requirements.
      
      parameters:
        - name: dimensions
          in: query
          description: Filter models by supported dimensions
          schema:
            type: integer
            minimum: 64
            maximum: 4096
        
        - name: hardware_profile
          in: query
          description: Filter models by hardware compatibility
          schema:
            type: string
            enum: [minimal, low, medium, high, extreme]
      
      responses:
        '200':
          description: Available models
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ModelsResponse'

  /models/{model_id}/download:
    post:
      summary: Download and cache a specific model
      description: |
        Trigger download and caching of a specific model for faster inference.
        Useful for pre-warming the cache before high-volume usage.
      
      parameters:
        - name: model_id
          in: path
          required: true
          schema:
            type: string
            example: "all-MiniLM-L6-v2"
      
      responses:
        '200':
          description: Model download initiated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DownloadResponse'
        
        '404':
          description: Model not found

  /health:
    get:
      summary: Service health check
      description: |
        Check service health including model availability,
        GPU status, and system performance.
      
      responses:
        '200':
          description: Service is healthy
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'
        
        '503':
          description: Service unavailable

  /metrics:
    get:
      summary: Prometheus metrics endpoint
      description: Export performance metrics in Prometheus format
      
      responses:
        '200':
          description: Metrics in Prometheus format
          content:
            text/plain:
              schema:
                type: string
                example: |
                  # HELP embedding_requests_total Total embedding requests
                  # TYPE embedding_requests_total counter
                  embedding_requests_total{status="success"} 12345
                  
                  # HELP embedding_request_duration_seconds Request duration
                  # TYPE embedding_request_duration_seconds histogram
                  embedding_request_duration_seconds_bucket{le="0.01"} 1000

components:
  schemas:
    EmbeddingRequest:
      type: object
      required: [texts]
      properties:
        texts:
          type: array
          items:
            type: string
          minItems: 1
          maxItems: 1000
          description: Text inputs to generate embeddings for
          example: ["Natural language processing", "Machine learning"]
        
        dimensions:
          type: integer
          minimum: 64
          maximum: 4096
          default: 384
          description: Target embedding dimensions
          example: 768
        
        model:
          type: string
          enum: [efficient, high-quality, ultra-efficient, custom]
          default: "efficient"
          description: Model configuration preset
        
        config:
          $ref: '#/components/schemas/EmbeddingConfig'
          description: Advanced configuration options
    
    EmbeddingConfig:
      type: object
      properties:
        quantization:
          type: string
          enum: [none, int8, int4, binary]
          default: "int8"
        
        use_fp16:
          type: boolean
          default: true
          description: Enable FP16 mixed precision if supported
        
        batch_size:
          type: integer
          minimum: 1
          maximum: 256
          default: 32
          description: Batch size for processing
        
        binary_quantization:
          type: boolean
          default: false
          description: Enable 1-bit binary quantization for extreme efficiency
        
        hardware_profile:
          type: string
          enum: [auto, minimal, desktop, server, h100]
          default: "auto"
          description: Target hardware profile for optimization
    
    EmbeddingResponse:
      type: object
      required: [embeddings, model_info, performance, metadata]
      properties:
        embeddings:
          type: array
          items:
            type: array
            items:
              type: number
              format: float
          description: Generated embeddings (one per input text)
        
        model_info:
          $ref: '#/components/schemas/ModelInfo'
        
        performance:
          $ref: '#/components/schemas/PerformanceMetrics'
        
        metadata:
          $ref: '#/components/schemas/ResponseMetadata'
    
    ModelInfo:
      type: object
      properties:
        model_id:
          type: string
          example: "all-MiniLM-L6-v2"
        
        name:
          type: string
          example: "All MiniLM L6 v2"
        
        dimensions:
          type: integer
          example: 384
        
        quality_score:
          type: number
          format: float
          example: 56.26
          description: MTEB benchmark score
        
        size_mb:
          type: number
          format: float
          example: 90.9
    
    PerformanceMetrics:
      type: object
      properties:
        latency_ms:
          type: number
          format: float
          example: 13.69
          description: Request processing latency in milliseconds
        
        throughput_emb_per_sec:
          type: number
          format: float
          example: 73.04
          description: Embeddings generated per second
        
        gpu_utilization:
          type: number
          format: float
          example: 0.75
          description: GPU utilization percentage (0.0-1.0)
        
        memory_usage_mb:
          type: number
          format: float
          example: 256.5
    
    StreamingRequest:
      type: object
      required: [config]
      properties:
        config:
          $ref: '#/components/schemas/StreamingConfig'
    
    StreamingConfig:
      type: object
      properties:
        dimensions:
          type: integer
          minimum: 64
          maximum: 4096
          default: 384
        
        buffer_size:
          type: integer
          minimum: 1
          maximum: 1000
          default: 100
        
        max_latency_ms:
          type: integer
          minimum: 10
          maximum: 5000
          default: 100
          description: Maximum acceptable latency before forcing batch processing
    
    StreamingResponse:
      type: object
      properties:
        chunk_id:
          type: string
          example: "chunk_001"
        
        embedding:
          type: array
          items:
            type: number
            format: float
        
        metadata:
          type: object
          properties:
            latency_ms:
              type: number
              format: float
            timestamp:
              type: string
              format: date-time
    
    ErrorResponse:
      type: object
      required: [error]
      properties:
        error:
          type: object
          required: [code, message]
          properties:
            code:
              type: string
              example: "INVALID_DIMENSIONS"
            
            message:
              type: string
              example: "Dimensions must be between 64 and 4096"
            
            details:
              type: object
              additionalProperties: true
            
            request_id:
              type: string
              example: "req_abc123"
    
    ModelsResponse:
      type: object
      properties:
        models:
          type: array
          items:
            $ref: '#/components/schemas/ModelInfo'
        
        total_count:
          type: integer
          example: 6
    
    HealthResponse:
      type: object
      required: [status]
      properties:
        status:
          type: string
          enum: [healthy, degraded, unhealthy]
          example: "healthy"
        
        timestamp:
          type: string
          format: date-time
        
        version:
          type: string
          example: "1.0.0"
        
        checks:
          type: object
          properties:
            models_available:
              type: boolean
              example: true
            
            gpu_accessible:
              type: boolean
              example: true
            
            model_cache_healthy:
              type: boolean
              example: true
        
        performance:
          type: object
          properties:
            avg_latency_ms:
              type: number
              format: float
              example: 13.69
            
            requests_per_second:
              type: number
              format: float
              example: 73.04

  securitySchemes:
    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
    
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
```

## 2. gRPC Service Definition

```protobuf
syntax = "proto3";

package sutra.embedder.v1;

option go_package = "github.com/sutra-embedder/api/gen/go/sutra/embedder/v1";
option java_package = "com.sutraembedder.api.v1";
option csharp_namespace = "Sutra.Embedder.V1";

import "google/protobuf/timestamp.proto";
import "google/api/annotations.proto";

// Sutra Embedder gRPC Service
service EmbedderService {
  // Generate embeddings for text inputs
  rpc GenerateEmbeddings(EmbeddingRequest) returns (EmbeddingResponse) {
    option (google.api.http) = {
      post: "/v1/embeddings"
      body: "*"
    };
  }
  
  // Stream embeddings for real-time processing
  rpc StreamEmbeddings(StreamingRequest) returns (stream StreamingResponse);
  
  // Get available models
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse) {
    option (google.api.http) = {
      get: "/v1/models"
    };
  }
  
  // Download and cache model
  rpc DownloadModel(DownloadModelRequest) returns (DownloadModelResponse) {
    option (google.api.http) = {
      post: "/v1/models/{model_id}/download"
    };
  }
  
  // Health check
  rpc GetHealth(HealthRequest) returns (HealthResponse) {
    option (google.api.http) = {
      get: "/v1/health"
    };
  }
}

// Request to generate embeddings
message EmbeddingRequest {
  // Text inputs to embed
  repeated string texts = 1;
  
  // Target embedding dimensions
  int32 dimensions = 2;
  
  // Model configuration
  string model = 3;
  
  // Advanced configuration
  EmbeddingConfig config = 4;
}

// Advanced embedding configuration
message EmbeddingConfig {
  // Quantization type
  enum QuantizationType {
    QUANTIZATION_TYPE_UNSPECIFIED = 0;
    QUANTIZATION_TYPE_NONE = 1;
    QUANTIZATION_TYPE_INT8 = 2;
    QUANTIZATION_TYPE_INT4 = 3;
    QUANTIZATION_TYPE_BINARY = 4;
  }
  
  QuantizationType quantization = 1;
  bool use_fp16 = 2;
  int32 batch_size = 3;
  bool binary_quantization = 4;
  string hardware_profile = 5;
}

// Response containing generated embeddings
message EmbeddingResponse {
  // Generated embeddings (one per input text)
  repeated Embedding embeddings = 1;
  
  // Model information
  ModelInfo model_info = 2;
  
  // Performance metrics
  PerformanceMetrics performance = 3;
  
  // Response metadata
  ResponseMetadata metadata = 4;
}

// Single embedding vector
message Embedding {
  // Embedding values
  repeated float values = 1;
}

// Model information
message ModelInfo {
  string model_id = 1;
  string name = 2;
  int32 dimensions = 3;
  float quality_score = 4;
  float size_mb = 5;
}

// Performance metrics
message PerformanceMetrics {
  float latency_ms = 1;
  float throughput_emb_per_sec = 2;
  float gpu_utilization = 3;
  float memory_usage_mb = 4;
}

// Response metadata
message ResponseMetadata {
  string request_id = 1;
  google.protobuf.Timestamp timestamp = 2;
  string hardware_profile = 3;
}

// Streaming request
message StreamingRequest {
  StreamingConfig config = 1;
}

// Streaming configuration
message StreamingConfig {
  int32 dimensions = 1;
  int32 buffer_size = 2;
  int32 max_latency_ms = 3;
}

// Streaming response
message StreamingResponse {
  string chunk_id = 1;
  Embedding embedding = 2;
  StreamingMetadata metadata = 3;
}

// Streaming metadata
message StreamingMetadata {
  float latency_ms = 1;
  google.protobuf.Timestamp timestamp = 2;
}

// List models request
message ListModelsRequest {
  // Filter by dimensions
  int32 dimensions = 1;
  
  // Filter by hardware profile
  string hardware_profile = 2;
}

// List models response
message ListModelsResponse {
  repeated ModelInfo models = 1;
  int32 total_count = 2;
}

// Download model request
message DownloadModelRequest {
  string model_id = 1;
}

// Download model response
message DownloadModelResponse {
  string status = 1;
  float progress = 2;
  int64 estimated_time_remaining_ms = 3;
}

// Health check request
message HealthRequest {}

// Health check response
message HealthResponse {
  enum HealthStatus {
    HEALTH_STATUS_UNSPECIFIED = 0;
    HEALTH_STATUS_HEALTHY = 1;
    HEALTH_STATUS_DEGRADED = 2;
    HEALTH_STATUS_UNHEALTHY = 3;
  }
  
  HealthStatus status = 1;
  google.protobuf.Timestamp timestamp = 2;
  string version = 3;
  HealthChecks checks = 4;
  HealthPerformance performance = 5;
}

// Health check details
message HealthChecks {
  bool models_available = 1;
  bool gpu_accessible = 2;
  bool model_cache_healthy = 3;
}

// Health performance metrics
message HealthPerformance {
  float avg_latency_ms = 1;
  float requests_per_second = 2;
}
```

## 3. WebSocket API for Real-Time Streaming

### WebSocket Protocol Specification

```typescript
// WebSocket Message Types
interface WebSocketMessage {
  id: string;
  type: 'request' | 'response' | 'error' | 'ping' | 'pong';
  timestamp: string;
  payload: any;
}

// Streaming Embedding Request
interface StreamingEmbeddingRequest extends WebSocketMessage {
  type: 'request';
  payload: {
    action: 'stream_embed';
    config: {
      dimensions: number;
      buffer_size: number;
      max_latency_ms: number;
      model?: string;
    };
    metadata?: {
      session_id: string;
      user_id?: string;
    };
  };
}

// Text Chunk Input
interface TextChunk extends WebSocketMessage {
  type: 'request';
  payload: {
    action: 'add_chunk';
    chunk_id: string;
    text: string;
    is_final?: boolean;
    position?: number;
  };
}

// Streaming Embedding Response
interface StreamingEmbeddingResponse extends WebSocketMessage {
  type: 'response';
  payload: {
    chunk_id: string;
    embedding: number[];
    metadata: {
      latency_ms: number;
      sequence_position: number;
      model_used: string;
      quality_score?: number;
    };
  };
}

// Error Response
interface ErrorResponse extends WebSocketMessage {
  type: 'error';
  payload: {
    error_code: string;
    error_message: string;
    chunk_id?: string;
    retry_after_ms?: number;
  };
}

// Connection State Management
interface ConnectionState {
  session_id: string;
  is_streaming: boolean;
  buffer_utilization: number;
  chunks_processed: number;
  average_latency_ms: number;
  last_activity: string;
}

// WebSocket Event Handlers
interface WebSocketEventHandlers {
  onOpen: (event: Event) => void;
  onMessage: (message: WebSocketMessage) => void;
  onError: (error: ErrorResponse) => void;
  onClose: (event: CloseEvent) => void;
  onStateChange: (state: ConnectionState) => void;
}
```

### WebSocket Implementation Algorithm

```typescript
class SutraEmbedderWebSocket {
  private ws: WebSocket;
  private sessionId: string;
  private messageQueue: Map<string, WebSocketMessage>;
  private handlers: WebSocketEventHandlers;
  
  constructor(url: string, handlers: WebSocketEventHandlers) {
    this.sessionId = this.generateSessionId();
    this.messageQueue = new Map();
    this.handlers = handlers;
    this.connect(url);
  }
  
  private connect(url: string): void {
    this.ws = new WebSocket(url);
    
    this.ws.onopen = (event) => {
      this.sendHandshake();
      this.handlers.onOpen(event);
    };
    
    this.ws.onmessage = (event) => {
      const message: WebSocketMessage = JSON.parse(event.data);
      this.handleMessage(message);
    };
    
    this.ws.onerror = (event) => {
      const errorResponse: ErrorResponse = {
        id: this.generateMessageId(),
        type: 'error',
        timestamp: new Date().toISOString(),
        payload: {
          error_code: 'CONNECTION_ERROR',
          error_message: 'WebSocket connection error'
        }
      };
      this.handlers.onError(errorResponse);
    };
    
    this.ws.onclose = (event) => {
      this.handlers.onClose(event);
      
      // Automatic reconnection with exponential backoff
      if (!event.wasClean) {
        this.scheduleReconnection();
      }
    };
  }
  
  public startStreaming(config: StreamingConfig): void {
    const request: StreamingEmbeddingRequest = {
      id: this.generateMessageId(),
      type: 'request',
      timestamp: new Date().toISOString(),
      payload: {
        action: 'stream_embed',
        config: config,
        metadata: {
          session_id: this.sessionId
        }
      }
    };
    
    this.sendMessage(request);
  }
  
  public addTextChunk(text: string, isFinal: boolean = false): void {
    const chunkId = this.generateChunkId();
    
    const chunk: TextChunk = {
      id: this.generateMessageId(),
      type: 'request',
      timestamp: new Date().toISOString(),
      payload: {
        action: 'add_chunk',
        chunk_id: chunkId,
        text: text,
        is_final: isFinal
      }
    };
    
    this.sendMessage(chunk);
  }
  
  private handleMessage(message: WebSocketMessage): void {
    switch (message.type) {
      case 'response':
        if (message.payload.chunk_id) {
          this.handlers.onMessage(message);
        }
        break;
        
      case 'error':
        this.handlers.onError(message as ErrorResponse);
        break;
        
      case 'ping':
        this.sendPong(message.id);
        break;
        
      default:
        console.warn('Unknown message type:', message.type);
    }
  }
  
  private sendMessage(message: WebSocketMessage): void {
    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
      this.messageQueue.set(message.id, message);
    } else {
      throw new Error('WebSocket connection not ready');
    }
  }
  
  private sendHandshake(): void {
    const handshake = {
      id: this.generateMessageId(),
      type: 'request',
      timestamp: new Date().toISOString(),
      payload: {
        action: 'handshake',
        session_id: this.sessionId,
        protocol_version: '1.0',
        client_capabilities: {
          supports_compression: true,
          max_chunk_size: 10000,
          preferred_dimensions: [384, 768]
        }
      }
    };
    
    this.sendMessage(handshake);
  }
  
  private sendPong(pingId: string): void {
    const pong = {
      id: this.generateMessageId(),
      type: 'pong',
      timestamp: new Date().toISOString(),
      payload: {
        ping_id: pingId
      }
    };
    
    this.sendMessage(pong);
  }
  
  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateMessageId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateChunkId(): string {
    return `chunk_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private scheduleReconnection(): void {
    // Exponential backoff reconnection
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    
    setTimeout(() => {
      this.connect(this.originalUrl);
      this.reconnectAttempts++;
    }, delay);
  }
}
```

## 4. SDK and Client Libraries

### Python SDK Implementation

```python
"""
Sutra-Embedder Python SDK
High-performance embedding client with automatic optimization
"""

import asyncio
import aiohttp
import numpy as np
from typing import List, Optional, Dict, Any, AsyncIterator
from dataclasses import dataclass
from enum import Enum

class ModelPreset(Enum):
    """Pre-configured model presets"""
    EFFICIENT = "efficient"
    HIGH_QUALITY = "high-quality"
    ULTRA_EFFICIENT = "ultra-efficient"
    CUSTOM = "custom"

class QuantizationType(Enum):
    """Quantization types for optimization"""
    NONE = "none"
    INT8 = "int8"
    INT4 = "int4"
    BINARY = "binary"

@dataclass
class EmbeddingConfig:
    """Configuration for embedding generation"""
    dimensions: int = 384
    model: ModelPreset = ModelPreset.EFFICIENT
    quantization: QuantizationType = QuantizationType.INT8
    use_fp16: bool = True
    batch_size: int = 32
    hardware_profile: str = "auto"
    binary_quantization: bool = False

@dataclass
class EmbeddingResult:
    """Result from embedding generation"""
    embeddings: List[List[float]]
    model_info: Dict[str, Any]
    performance: Dict[str, float]
    metadata: Dict[str, Any]

@dataclass
class StreamingConfig:
    """Configuration for streaming embeddings"""
    dimensions: int = 384
    buffer_size: int = 100
    max_latency_ms: int = 100
    model: ModelPreset = ModelPreset.EFFICIENT

class SutraEmbedder:
    """
    Sutra-Embedder client for high-performance embedding generation
    
    Example usage:
        # Synchronous usage
        embedder = SutraEmbedder(api_key="your-api-key")
        
        result = embedder.embed([
            "Natural language processing",
            "Machine learning algorithms"
        ], dimensions=768)
        
        # Asynchronous usage
        async with SutraEmbedder(api_key="your-api-key") as embedder:
            result = await embedder.embed_async([
                "Text to embed"
            ], config=EmbeddingConfig(
                dimensions=512,
                model=ModelPreset.HIGH_QUALITY,
                use_fp16=True
            ))
    """
    
    def __init__(
        self,
        api_key: str,
        base_url: str = "https://api.sutra-embedder.com/v1",
        timeout: int = 30,
        retry_attempts: int = 3
    ):
        self.api_key = api_key
        self.base_url = base_url.rstrip('/')
        self.timeout = timeout
        self.retry_attempts = retry_attempts
        self._session: Optional[aiohttp.ClientSession] = None
    
    async def __aenter__(self):
        self._session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=self.timeout),
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
                "User-Agent": "sutra-embedder-python-sdk/1.0.0"
            }
        )
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._session:
            await self._session.close()
    
    def embed(
        self,
        texts: List[str],
        dimensions: int = 384,
        config: Optional[EmbeddingConfig] = None
    ) -> EmbeddingResult:
        """
        Generate embeddings synchronously
        
        Args:
            texts: List of text strings to embed
            dimensions: Target embedding dimensions (64-4096)
            config: Optional advanced configuration
            
        Returns:
            EmbeddingResult with embeddings and metadata
        """
        return asyncio.run(self.embed_async(texts, dimensions, config))
    
    async def embed_async(
        self,
        texts: List[str],
        dimensions: int = 384,
        config: Optional[EmbeddingConfig] = None
    ) -> EmbeddingResult:
        """
        Generate embeddings asynchronously
        
        Args:
            texts: List of text strings to embed
            dimensions: Target embedding dimensions (64-4096)
            config: Optional advanced configuration
            
        Returns:
            EmbeddingResult with embeddings and metadata
            
        Raises:
            ValueError: If inputs are invalid
            APIError: If API request fails
        """
        if not texts:
            raise ValueError("texts cannot be empty")
        
        if not (64 <= dimensions <= 4096):
            raise ValueError("dimensions must be between 64 and 4096")
        
        if len(texts) > 1000:
            raise ValueError("maximum 1000 texts per request")
        
        # Use provided config or create default
        if config is None:
            config = EmbeddingConfig(dimensions=dimensions)
        
        request_payload = {
            "texts": texts,
            "dimensions": dimensions,
            "model": config.model.value,
            "config": {
                "quantization": config.quantization.value,
                "use_fp16": config.use_fp16,
                "batch_size": config.batch_size,
                "hardware_profile": config.hardware_profile,
                "binary_quantization": config.binary_quantization
            }
        }
        
        # Make API request with retry logic
        for attempt in range(self.retry_attempts):
            try:
                async with self._session.post(
                    f"{self.base_url}/embeddings",
                    json=request_payload
                ) as response:
                    
                    if response.status == 200:
                        data = await response.json()
                        return EmbeddingResult(
                            embeddings=data["embeddings"],
                            model_info=data["model_info"],
                            performance=data["performance"],
                            metadata=data["metadata"]
                        )
                    
                    elif response.status == 429:  # Rate limited
                        retry_after = int(response.headers.get("Retry-After", 1))
                        await asyncio.sleep(retry_after)
                        continue
                    
                    else:
                        error_data = await response.json()
                        raise APIError(
                            f"API request failed: {error_data['error']['message']}",
                            status_code=response.status,
                            error_code=error_data['error']['code']
                        )
                        
            except asyncio.TimeoutError:
                if attempt < self.retry_attempts - 1:
                    await asyncio.sleep(2 ** attempt)  # Exponential backoff
                    continue
                raise APIError("Request timeout after retries")
            
            except aiohttp.ClientError as e:
                if attempt < self.retry_attempts - 1:
                    await asyncio.sleep(2 ** attempt)
                    continue
                raise APIError(f"Network error: {str(e)}")
        
        raise APIError("Max retry attempts exceeded")
    
    async def embed_stream(
        self,
        text_stream: AsyncIterator[str],
        config: StreamingConfig
    ) -> AsyncIterator[List[float]]:
        """
        Generate embeddings from a text stream
        
        Args:
            text_stream: Async iterator of text chunks
            config: Streaming configuration
            
        Yields:
            Individual embeddings as they are generated
        """
        # Implementation would use WebSocket or Server-Sent Events
        # for real-time streaming capability
        pass
    
    async def get_models(
        self,
        dimensions: Optional[int] = None,
        hardware_profile: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """
        Get available embedding models
        
        Args:
            dimensions: Filter by supported dimensions
            hardware_profile: Filter by hardware compatibility
            
        Returns:
            List of available models with metadata
        """
        params = {}
        if dimensions:
            params["dimensions"] = dimensions
        if hardware_profile:
            params["hardware_profile"] = hardware_profile
        
        async with self._session.get(
            f"{self.base_url}/models",
            params=params
        ) as response:
            data = await response.json()
            return data["models"]
    
    async def download_model(self, model_id: str) -> Dict[str, Any]:
        """
        Download and cache a model
        
        Args:
            model_id: ID of model to download
            
        Returns:
            Download status information
        """
        async with self._session.post(
            f"{self.base_url}/models/{model_id}/download"
        ) as response:
            return await response.json()
    
    async def health_check(self) -> Dict[str, Any]:
        """
        Check service health
        
        Returns:
            Service health information
        """
        async with self._session.get(f"{self.base_url}/health") as response:
            return await response.json()

class APIError(Exception):
    """Exception raised for API errors"""
    
    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        error_code: Optional[str] = None
    ):
        super().__init__(message)
        self.status_code = status_code
        self.error_code = error_code

# Utility functions for common operations
def cosine_similarity(a: List[float], b: List[float]) -> float:
    """Compute cosine similarity between two embeddings"""
    a_np = np.array(a)
    b_np = np.array(b)
    
    dot_product = np.dot(a_np, b_np)
    norm_a = np.linalg.norm(a_np)
    norm_b = np.linalg.norm(b_np)
    
    return dot_product / (norm_a * norm_b)

def batch_cosine_similarity(
    embeddings_a: List[List[float]],
    embeddings_b: List[List[float]]
) -> List[List[float]]:
    """Compute pairwise cosine similarities between two sets of embeddings"""
    a_matrix = np.array(embeddings_a)
    b_matrix = np.array(embeddings_b)
    
    # Normalize embeddings
    a_norm = a_matrix / np.linalg.norm(a_matrix, axis=1, keepdims=True)
    b_norm = b_matrix / np.linalg.norm(b_matrix, axis=1, keepdims=True)
    
    # Compute similarity matrix
    similarity_matrix = np.dot(a_norm, b_norm.T)
    
    return similarity_matrix.tolist()

def find_most_similar(
    query_embedding: List[float],
    candidate_embeddings: List[List[float]],
    top_k: int = 5
) -> List[tuple]:
    """
    Find most similar embeddings to a query
    
    Args:
        query_embedding: Query embedding vector
        candidate_embeddings: List of candidate embeddings
        top_k: Number of top results to return
        
    Returns:
        List of (index, similarity_score) tuples
    """
    similarities = [
        cosine_similarity(query_embedding, candidate)
        for candidate in candidate_embeddings
    ]
    
    # Get top-k indices
    sorted_indices = np.argsort(similarities)[::-1][:top_k]
    
    return [
        (idx, similarities[idx])
        for idx in sorted_indices
    ]
```

---

*Document Version: 1.0*  
*Last Updated: November 13, 2025*  
*Authors: Sutra-Embedder API Design Team*