import { create } from 'zustand'
import type { ReasoningResult, MetricsResponse } from '../types/api'

export type MessageType = 'user-query' | 'ai-response' | 'user-learn' | 'system'

export interface Message {
  id: string
  type: MessageType
  content: string
  timestamp: number
  reasoning?: ReasoningResult
  metadata?: {
    factsLearned?: number
    error?: string
  }
}

interface AppState {
  // Conversation messages
  messages: Message[]
  addMessage: (message: Omit<Message, 'id' | 'timestamp'>) => void

  // System metrics
  metrics: MetricsResponse | null
  setMetrics: (metrics: MetricsResponse) => void

  // UI state
  isLoading: boolean
  setIsLoading: (loading: boolean) => void

  // Expanded reasoning graphs
  expandedGraphs: Set<string>
  toggleGraph: (messageId: string) => void
}

export const useAppStore = create<AppState>((set) => ({
  messages: [],
  addMessage: (message) =>
    set((state) => ({
      messages: [
        ...state.messages,
        {
          ...message,
          id: `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          timestamp: Date.now(),
        },
      ],
    })),

  metrics: null,
  setMetrics: (metrics) => set({ metrics }),

  isLoading: false,
  setIsLoading: (isLoading) => set({ isLoading }),

  expandedGraphs: new Set(),
  toggleGraph: (messageId) =>
    set((state) => {
      const newSet = new Set(state.expandedGraphs)
      if (newSet.has(messageId)) {
        newSet.delete(messageId)
      } else {
        newSet.add(messageId)
      }
      return { expandedGraphs: newSet }
    }),
}))
