import { create } from "zustand";

interface SettingsState {
  mockMode: boolean;
  network: string;
  setMockMode: (enabled: boolean) => void;
  setNetwork: (network: string) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  mockMode: false,
  network: "base-sepolia",
  setMockMode: (enabled: boolean) => set({ mockMode: enabled }),
  setNetwork: (network: string) => set({ network }),
}));
