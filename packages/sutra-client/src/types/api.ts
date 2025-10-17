export interface ReasoningPath {
  concepts: string[]
  confidence: number
  explanation: string
}

export interface ReasoningResult {
  query: string
  answer: string
  confidence: number
  paths: ReasoningPath[]
  concepts_accessed: number
}

export interface LearnResponse {
  concept_id: string
  message: string
  concepts_created: number
  associations_created: number
}

export interface MetricsResponse {
  total_concepts: number
  total_associations: number
  total_embeddings: number
  embedding_provider: string
  embedding_dimension: number
  average_strength: number
  memory_usage_mb: number | null
}

export interface HealthResponse {
  status: string
  version: string
  uptime_seconds: number
  concepts_loaded: number
}
