import { create } from 'zustand'
import { tauriApi } from '../lib/tauri'

interface AuthState {
  isAuthenticated: boolean
  email: string | null
  flowId: string | null

  setAuthenticated: (email: string) => void
  setFlowId: (id: string) => void
  logout: () => void
  checkAuthStatus: () => Promise<void>
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  email: null,
  flowId: null,

  setAuthenticated: (email) => set({ isAuthenticated: true, email }),

  setFlowId: (id) => set({ flowId: id }),

  logout: () => set({ isAuthenticated: false, email: null, flowId: null }),

  checkAuthStatus: async () => {
    try {
      const result = await tauriApi.auth.status()
      if (result.authenticated && result.email) {
        set({ isAuthenticated: true, email: result.email })
      } else {
        // In browser (non-Tauri), keep current state
        if (typeof window !== 'undefined' && !(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__) return
        set({ isAuthenticated: false, email: null, flowId: null })
      }
    } catch {
      // In browser (non-Tauri), keep current state for visual testing
      if (typeof window !== 'undefined' && !(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__) return
      set({ isAuthenticated: false, email: null, flowId: null })
    }
  },
}))
