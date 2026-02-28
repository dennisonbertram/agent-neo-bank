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
    if (!isTauri()) {
      // In browser (non-Tauri), skip auth check and mark as loaded
      set({ isLoading: false })
      return
    }
    try {
      const result = await tauriApi.auth.status()
      if (result.authenticated && result.email) {
        set({ isAuthenticated: true, email: result.email, isLoading: false })
      } else {
        set({ isAuthenticated: false, email: null, flowId: null, isLoading: false })
      }
    } catch {
      set({ isAuthenticated: false, email: null, flowId: null, isLoading: false })
    }
  },
}))
