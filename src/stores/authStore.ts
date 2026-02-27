import { create } from "zustand";

interface AuthState {
  authenticated: boolean;
  email: string | null;
  setAuthenticated: (authenticated: boolean, email?: string) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  authenticated: false,
  email: null,
  setAuthenticated: (authenticated, email) =>
    set({ authenticated, email: email ?? null }),
  logout: () => set({ authenticated: false, email: null }),
}));
