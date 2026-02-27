import { create } from "zustand";

interface AuthState {
  isAuthenticated: boolean;
  email: string | null;
  setAuthenticated: (email: string) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  email: null,
  setAuthenticated: (email: string) => set({ isAuthenticated: true, email }),
  clearAuth: () => set({ isAuthenticated: false, email: null }),
}));
