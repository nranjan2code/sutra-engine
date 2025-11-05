import { createContext, useState, useContext, useEffect, ReactNode } from 'react'
import { useNavigate } from 'react-router-dom'
import { authApi } from '../services/api'

interface ApiError {
  response?: {
    data?: {
      detail?: string
    }
  }
}

interface User {
  id: string
  email: string
  role: string
  organization: string
  active: boolean
  created_at: string
}

interface AuthContextType {
  user: User | null
  login: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
  loading: boolean
  error: string | null
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

interface AuthProviderProps {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()
  
  // Check if user is logged in on mount (via httpOnly cookies)
  useEffect(() => {
    const checkAuth = async () => {
      try {
        // Session validated via httpOnly cookies automatically
        const userData = await authApi.getCurrentUser()
        setUser(userData)
      } catch (err) {
        // No valid session - user will be redirected to login if needed
        console.debug('No active session')
      } finally {
        setLoading(false)
      }
    }
    
    checkAuth()
  }, [])
  
  const login = async (email: string, password: string) => {
    try {
      setError(null)
      setLoading(true)
      const response = await authApi.login(email, password)
      
      // ✅ PRODUCTION SECURITY: Tokens stored in httpOnly cookies (server-side)
      // No client-side token storage - immune to XSS attacks
      
      // Set user from response
      setUser(response.user)
      
      // Navigate to home
      navigate('/')
    } catch (err) {
      const apiError = err as ApiError
      const errorMessage = apiError.response?.data?.detail || 'Login failed'
      setError(errorMessage)
      throw new Error(errorMessage)
    } finally {
      setLoading(false)
    }
  }
  
  const logout = async () => {
    try {
      setLoading(true)
      // Logout clears httpOnly cookies server-side
      await authApi.logout()
    } catch (err) {
      console.error('Logout error:', err)
    } finally {
      // Clear local state (cookies already cleared by server)
      setUser(null)
      setLoading(false)
      navigate('/login')
    }
  }
  
  return (
    <AuthContext.Provider value={{ user, login, logout, loading, error }}>
      {children}
    </AuthContext.Provider>
  )
}

// Hook to use auth context
export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider')
  }
  return context
}
