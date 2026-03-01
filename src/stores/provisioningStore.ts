import { create } from 'zustand'
import { tauriApi } from '../lib/tauri'
import { safeTauriCall } from '../lib/tauri'
import type {
  DetectionResult,
  McpInjectionConfig,
  ProvisioningState,
  ToolId,
} from '../types'

function buildMcpConfig(): McpInjectionConfig {
  return {
    server_command: 'node',
    server_args: ['node_modules/.bin/awal', 'mcp', 'serve'],
    env: {},
    tally_version: '1.0.0',
    provisioned_at: new Date().toISOString(),
  }
}

interface ProvisioningStore {
  detectionResults: DetectionResult[]
  state: ProvisioningState | null
  isDetecting: boolean
  error: string | null
  actionInProgress: Record<string, boolean>
  actionError: Record<string, string | null>

  initialize: () => Promise<void>
  detectTools: () => Promise<void>
  refreshState: () => Promise<void>
  provisionTool: (tool: ToolId) => Promise<void>
  unprovisionTool: (tool: ToolId) => Promise<void>
}

export const useProvisioningStore = create<ProvisioningStore>((set, get) => ({
  detectionResults: [],
  state: null,
  isDetecting: false,
  error: null,
  actionInProgress: {},
  actionError: {},

  initialize: async () => {
    try {
      set({ isDetecting: true, error: null })
      const [results, state] = await Promise.all([
        safeTauriCall(() => tauriApi.provisioning.detectTools(), []),
        safeTauriCall(() => tauriApi.provisioning.getState(), null),
      ])
      set({ detectionResults: results, state, isDetecting: false })
    } catch (e) {
      set({ error: String(e), isDetecting: false })
    }
  },

  detectTools: async () => {
    try {
      set({ isDetecting: true, error: null })
      const results = await safeTauriCall(
        () => tauriApi.provisioning.detectTools(),
        [],
      )
      set({ detectionResults: results, isDetecting: false })
    } catch (e) {
      set({ error: String(e), isDetecting: false })
    }
  },

  refreshState: async () => {
    try {
      const state = await safeTauriCall(
        () => tauriApi.provisioning.getState(),
        null,
      )
      set({ state })
    } catch (e) {
      set({ error: String(e) })
    }
  },

  provisionTool: async (tool: ToolId) => {
    set((s) => ({
      actionInProgress: { ...s.actionInProgress, [tool]: true },
      actionError: { ...s.actionError, [tool]: null },
    }))
    try {
      const config = buildMcpConfig()
      await tauriApi.provisioning.provisionTool(tool, config)
      await get().refreshState()
      await get().detectTools()
    } catch (e) {
      set((s) => ({
        actionError: { ...s.actionError, [tool]: String(e) },
      }))
    } finally {
      set((s) => ({
        actionInProgress: { ...s.actionInProgress, [tool]: false },
      }))
    }
  },

  unprovisionTool: async (tool: ToolId) => {
    set((s) => ({
      actionInProgress: { ...s.actionInProgress, [tool]: true },
      actionError: { ...s.actionError, [tool]: null },
    }))
    try {
      await tauriApi.provisioning.unprovisionTool(tool)
      await get().refreshState()
      await get().detectTools()
    } catch (e) {
      set((s) => ({
        actionError: { ...s.actionError, [tool]: String(e) },
      }))
    } finally {
      set((s) => ({
        actionInProgress: { ...s.actionInProgress, [tool]: false },
      }))
    }
  },
}))
