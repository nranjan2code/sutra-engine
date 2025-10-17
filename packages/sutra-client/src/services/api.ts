import axios from 'axios'
import type {
  ReasoningResult,
  LearnResponse,
  MetricsResponse,
  HealthResponse,
} from '../types/api'

const API_BASE = '/api'

const api = axios.create({
  baseURL: API_BASE,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

export const sutraApi = {
  /**
   * Query the reasoning engine
   */
  async reason(query: string): Promise<ReasoningResult> {
    const { data } = await api.post('/reason', { query })
    return data
  },

  /**
   * Learn new facts
   */
  async learn(text: string): Promise<LearnResponse> {
    const { data } = await api.post('/learn', { text })
    return data
  },

  /**
   * Get system statistics
   */
  async getMetrics(): Promise<MetricsResponse> {
    const { data } = await api.get('/stats')
    return data
  },

  /**
   * Check system health
   */
  async getHealth(): Promise<HealthResponse> {
    const { data } = await api.get('/health')
    return data
  },

  /**
   * Clear the knowledge base
   */
  async clearKnowledge(): Promise<{ status: string; message: string }> {
    const { data } = await api.post('/clear')
    return data
  },
}
