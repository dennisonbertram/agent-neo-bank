import { create } from 'zustand'
import { tauriApi, isTauri } from '../lib/tauri'

interface AuthState {
  isAuthenticated: boolean
  isLoading: boolean
  email: string | null
  flowId: string | null

  setAuthenticated: (email: string) => void
  setFlowId: (id: string) => void
  logout: () => void
  checkAuthStatus: () => Promise<void>
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  isLoading: true,
  email: null,
  flowId: null,

  setAuthenticated: (email) => set({ isAuthenticated: true, email, flowId: null }),

  setFlowId: (id) => set({ flowId: id }),

  logout: () => set({ isAuthenticated: false, email: null, flowId: null }),

  checkAuthStatus: async () => {
    console.log('[auth] checkAuthStatus called, isTauri:', isTauri())
    if (!isTauri()) {
      set({ isLoading: false })
      return
    }
    try {
      const timeout = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('Auth check timed out')), 5000)
      )
      console.log('[auth] calling tauriApi.auth.status()...')
      const result = await Promise.race([tauriApi.auth.status(), timeout])
      console.log('[auth] result:', JSON.stringify(result))
      if (result.authenticated && result.email) {
        set({ isAuthenticated: true, email: result.email, isLoading: false })
      } else {
        set({ isAuthenticated: false, email: null, flowId: null, isLoading: false })
      }
    } catch (err) {
      console.error('[auth] checkAuthStatus error:', err)
      set({ isAuthenticated: false, email: null, flowId: null, isLoading: false })
    }
  },
}))
