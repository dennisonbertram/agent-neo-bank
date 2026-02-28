import { create } from 'zustand'
import { tauriApi } from '../lib/tauri'
import type { AssetBalance } from '../types'

interface WalletState {
  address: string | null
  balances: Record<string, AssetBalance> | null
  totalBalance: string | null
  isLoading: boolean

  fetchBalance: () => Promise<void>
  fetchAddress: () => Promise<void>
}

export const useWalletStore = create<WalletState>((set) => ({
  address: null,
  balances: null,
  totalBalance: null,
  isLoading: false,

  fetchBalance: async () => {
    set({ isLoading: true })
    try {
      const result = await tauriApi.wallet.getBalance()
      set({
        balances: result.balances,
        totalBalance: result.balance,
        isLoading: false,
      })
    } catch {
      set({ isLoading: false })
    }
  },

  fetchAddress: async () => {
    try {
      const result = await tauriApi.wallet.getAddress()
      set({ address: result.address })
    } catch {
      // Address fetch failed — leave as null
    }
  },
}))
