import { create } from 'zustand'
import { safeTauriCall, tauriApi } from '../lib/tauri'
import type { AssetBalance, BalanceResponse } from '../types'

interface WalletState {
  address: string | null
  balances: Record<string, AssetBalance> | null
  totalBalance: string | null
  isLoading: boolean
  isInitialized: boolean

  /** Call once at app startup (after auth). Fetches address + balance, starts polling. */
  initialize: () => void
  /** Stop balance polling (call on logout / unmount). */
  teardown: () => void
}

let pollInterval: ReturnType<typeof setInterval> | null = null

export const useWalletStore = create<WalletState>((set, get) => ({
  address: null,
  balances: null,
  totalBalance: null,
  isLoading: false,
  isInitialized: false,

  initialize: () => {
    if (get().isInitialized) return
    set({ isLoading: true, isInitialized: true })

    // Fetch address (stable, only need once)
    safeTauriCall(
      () => tauriApi.wallet.getAddress(),
      { address: '' },
    ).then((result) => {
      if (result.address) set({ address: result.address })
    })

    // Fetch balance immediately, then poll
    const fetchBalance = async () => {
      const result = await safeTauriCall<BalanceResponse | null>(
        () => tauriApi.wallet.getBalance(),
        null,
      )
      if (result) {
        set({
          balances: result.balances,
          totalBalance: result.balance,
          isLoading: false,
        })
      } else {
        set({ isLoading: false })
      }
    }

    fetchBalance()
    pollInterval = setInterval(fetchBalance, 15_000)
  },

  teardown: () => {
    if (pollInterval) {
      clearInterval(pollInterval)
      pollInterval = null
    }
    set({
      address: null,
      balances: null,
      totalBalance: null,
      isLoading: false,
      isInitialized: false,
    })
  },
}))
