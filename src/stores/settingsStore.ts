import { create } from "zustand";

interface SettingsState {
  network: string;
  mockMode: boolean;
  setNetwork: (network: string) => void;
  setMockMode: (mockMode: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  network: "base-sepolia",
  mockMode: false,
  setNetwork: (network) => set({ network }),
  setMockMode: (mockMode) => set({ mockMode }),
}));
