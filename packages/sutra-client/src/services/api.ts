import axios from 'axios'
import type {
  ReasoningResult,
  LearnResponse,
  MetricsResponse,
  HealthResponse,
  EditionResponse,
} from '../types/api'

export const API_BASE_URL = '/api'

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  withCredentials: true, // ✅ PRODUCTION: Enable httpOnly cookies
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor to add auth token (from secure storage)
api.interceptors.request.use(
  (config) => {
    // Token is now handled via httpOnly cookies automatically
    // No manual Authorization header needed
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor to handle 401 errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    // If 401 and we haven't tried to refresh yet
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true

      try {
        // Attempt refresh using httpOnly cookie (no manual token needed)
        await axios.post(`${API_BASE_URL}/auth/refresh`, {}, {
          withCredentials: true, // Include httpOnly cookies
        })
        
        // Tokens are now set as httpOnly cookies by the server
        // Retry original request
        return api({ ...originalRequest, withCredentials: true })
      } catch (refreshError) {
        // Refresh failed, redirect to login (cookies cleared by server)
        window.location.href = '/login'
        return Promise.reject(refreshError)
      }
    }

    return Promise.reject(error)
  }
)

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
    const { data } = await api.post('/learn', { content: text })
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
   * Get edition information
   */
  async getEdition(): Promise<EditionResponse> {
    const { data } = await api.get('/edition')
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

// Authentication API
export const authApi = {
  /**
   * Register a new user
   */
  async register(email: string, password: string, organization: string, fullName?: string, role?: string) {
    const { data } = await api.post('/auth/register', {
      email,
      password,
      organization,
      full_name: fullName,
      role: role || 'user',
    })
    return data
  },

  /**
   * Login user
   */
  async login(email: string, password: string) {
    const { data } = await api.post('/auth/login', {
      email,
      password,
    }, {
      withCredentials: true, // Enable httpOnly cookies
    })
    return data
  },

  /**
   * Logout user
   */
  async logout() {
    const { data } = await api.post('/auth/logout', {}, {
      withCredentials: true, // Clear httpOnly cookies
    })
    return data
  },

  /**
   * Get current user info
   */
  async getCurrentUser() {
    const { data } = await api.get('/auth/me')
    return data
  },

  /**
   * Refresh access token
   */
  async refreshToken() {
    const { data } = await api.post('/auth/refresh', {}, {
      withCredentials: true, // Use httpOnly cookies
    })
    return data
  },

  /**
   * Check auth service health
   */
  async checkHealth() {
    const { data } = await api.get('/auth/health')
    return data
  },
}

// Conversation API
export const conversationApi = {
  /**
   * Create a new conversation
   */
  async createConversation(
    spaceId: string,
    domainStorage?: string,
    title?: string
  ) {
    const { data } = await api.post('/conversations/create', {
      space_id: spaceId,
      domain_storage: domainStorage,
      title,
    })
    return data
  },

  /**
   * List conversations
   */
  async listConversations(
    page: number = 1,
    pageSize: number = 50,
    spaceId?: string
  ) {
    const { data } = await api.get('/conversations/list', {
      params: {
        page,
        page_size: pageSize,
        space_id: spaceId,
      },
    })
    return data
  },

  /**
   * Get conversation details
   */
  async getConversation(conversationId: string) {
    const { data } = await api.get(`/conversations/${conversationId}`)
    return data
  },

  /**
   * Load conversation messages
   */
  async loadMessages(
    conversationId: string,
    limit: number = 50,
    offset: number = 0
  ) {
    const { data } = await api.get(`/conversations/${conversationId}/messages`, {
      params: {
        limit,
        offset,
      },
    })
    return data
  },

  /**
   * Send a message in a conversation
   */
  async sendMessage(conversationId: string, message: string) {
    const { data } = await api.post(`/conversations/${conversationId}/message`, {
      message,
    })
    return data
  },

  /**
   * Update conversation metadata
   */
  async updateConversation(
    conversationId: string,
    updates: {
      title?: string
      starred?: boolean
      tags?: string[]
    }
  ) {
    const { data } = await api.patch(`/conversations/${conversationId}`, updates)
    return data
  },

  /**
   * Delete a conversation (soft delete)
   */
  async deleteConversation(conversationId: string) {
    const { data} = await api.delete(`/conversations/${conversationId}`)
    return data
  },
}

// Space API
export const spaceApi = {
  /**
   * Create a new space
   */
  async createSpace(
    name: string,
    domainStorage: string,
    description?: string,
    icon?: string,
    color?: string
  ) {
    const { data } = await api.post('/spaces/create', {
      name,
      domain_storage: domainStorage,
      description,
      icon,
      color,
    })
    return data
  },

  /**
   * List spaces accessible to current user
   */
  async listSpaces(includeInactive: boolean = false) {
    const { data } = await api.get('/spaces/list', {
      params: {
        include_inactive: includeInactive,
      },
    })
    return data
  },

  /**
   * Get space details
   */
  async getSpace(spaceId: string) {
    const { data } = await api.get(`/spaces/${spaceId}`)
    return data
  },

  /**
   * Update space details
   */
  async updateSpace(
    spaceId: string,
    updates: {
      name?: string
      description?: string
      icon?: string
      color?: string
    }
  ) {
    const { data } = await api.put(`/spaces/${spaceId}`, updates)
    return data
  },

  /**
   * Delete a space (soft delete)
   */
  async deleteSpace(spaceId: string) {
    const { data } = await api.delete(`/spaces/${spaceId}`)
    return data
  },

  /**
   * Add member to space
   */
  async addMember(
    spaceId: string,
    userId: string,
    role: 'admin' | 'write' | 'read'
  ) {
    const { data } = await api.post(`/spaces/${spaceId}/members`, {
      user_id: userId,
      role,
    })
    return data
  },

  /**
   * List space members
   */
  async listMembers(spaceId: string) {
    const { data } = await api.get(`/spaces/${spaceId}/members`)
    return data
  },

  /**
   * Remove member from space
   */
  async removeMember(spaceId: string, userId: string) {
    const { data } = await api.delete(`/spaces/${spaceId}/members/${userId}`)
    return data
  },

  /**
   * Update member role in space
   */
  async updateMemberRole(
    spaceId: string,
    userId: string,
    newRole: 'admin' | 'write' | 'read'
  ) {
    const { data } = await api.put(`/spaces/${spaceId}/members/${userId}/role`, {
      user_id: userId,
      new_role: newRole,
    })
    return data
  },
}

/**
 * Graph API
 */
export const graphApi = {
  /**
   * Get knowledge graph for a message's reasoning
   */
  async getMessageGraph(request: import('../types/api').MessageGraphRequest) {
    const { data } = await api.post('/graph/message', request)
    return data as import('../types/api').MessageGraphResponse
  },

  /**
   * Get subgraph around a concept
   */
  async getConceptGraph(request: import('../types/api').ConceptGraphRequest) {
    const { data } = await api.post('/graph/concept', request)
    return data as import('../types/api').ConceptGraphResponse
  },

  /**
   * Get reasoning paths for a query
   */
  async getQueryGraph(request: import('../types/api').QueryGraphRequest) {
    const { data } = await api.post('/graph/query', request)
    return data as import('../types/api').QueryGraphResponse
  },

  /**
   * Get graph statistics
   */
  async getStatistics(domainStorage: string) {
    const { data } = await api.get(`/graph/statistics/${domainStorage}`)
    return data as import('../types/api').GraphStatisticsResponse
  },

  /**
   * Health check
   */
  async checkHealth() {
    const { data } = await api.get('/graph/health')
    return data
  },
}
