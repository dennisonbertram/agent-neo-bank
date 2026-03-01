import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useProvisioningStore } from './provisioningStore'
import type { DetectionResult, ProvisioningState } from '../types'

// Mock safeTauriCall and tauriApi from the tauri module
const mockDetectTools = vi.fn()
const mockGetState = vi.fn()

vi.mock('../lib/tauri', () => ({
  tauriApi: {
    provisioning: {
      detectTools: (...args: unknown[]) => mockDetectTools(...args),
      getState: (...args: unknown[]) => mockGetState(...args),
    },
  },
  safeTauriCall: vi.fn(async (fn: () => Promise<unknown>) => fn()),
}))

const initialState = {
  detectionResults: [],
  state: null,
  isDetecting: false,
  error: null,
}

const mockDetectionResults: DetectionResult[] = [
  {
    tool: 'claude_code',
    detected: true,
    methods: ['config_directory'],
    version: '1.0.0',
    config_paths: [
      {
        path: '~/.claude/config.json',
        resolved_path: '/home/user/.claude/config.json',
        exists: true,
        writable: true,
        format: 'json',
        purpose: 'mcp_server',
        is_symlink: false,
      },
    ],
  },
  {
    tool: 'cursor',
    detected: true,
    methods: ['binary_in_path', 'config_directory'],
    version: '0.42.0',
    config_paths: [],
  },
]

const mockProvisioningState: ProvisioningState = {
  schema_version: 1,
  machine_id: 'test-machine-id',
  tally_version: '0.1.0',
  tools: {
    claude_code: {
      status: 'provisioned',
      provisioned_at: '2026-01-01T00:00:00Z',
      last_verified: '2026-01-01T00:00:00Z',
      provisioned_version: '0.1.0',
      tool_version: '1.0.0',
      removal_count: 0,
      respect_removal: false,
      files_managed: ['/home/user/.claude/config.json'],
    },
  },
  excluded_tools: [],
  last_scan: '2026-01-01T00:00:00Z',
}

describe('provisioningStore', () => {
  beforeEach(() => {
    useProvisioningStore.setState(initialState)
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('has empty detectionResults', () => {
      expect(useProvisioningStore.getState().detectionResults).toEqual([])
    })

    it('has null state', () => {
      expect(useProvisioningStore.getState().state).toBeNull()
    })

    it('has isDetecting=false', () => {
      expect(useProvisioningStore.getState().isDetecting).toBe(false)
    })

    it('has null error', () => {
      expect(useProvisioningStore.getState().error).toBeNull()
    })
  })

  describe('initialize()', () => {
    it('sets isDetecting=true during fetch', async () => {
      let capturedIsDetecting = false
      mockDetectTools.mockImplementation(() => {
        capturedIsDetecting = useProvisioningStore.getState().isDetecting
        return Promise.resolve(mockDetectionResults)
      })
      mockGetState.mockResolvedValue(mockProvisioningState)

      await useProvisioningStore.getState().initialize()

      expect(capturedIsDetecting).toBe(true)
    })

    it('populates detectionResults and state on success', async () => {
      mockDetectTools.mockResolvedValue(mockDetectionResults)
      mockGetState.mockResolvedValue(mockProvisioningState)

      await useProvisioningStore.getState().initialize()

      const { detectionResults, state, isDetecting, error } =
        useProvisioningStore.getState()
      expect(detectionResults).toEqual(mockDetectionResults)
      expect(state).toEqual(mockProvisioningState)
      expect(isDetecting).toBe(false)
      expect(error).toBeNull()
    })

    it('sets error on failure and clears isDetecting', async () => {
      const { safeTauriCall } = await import('../lib/tauri')
      const mockSafeTauriCall = vi.mocked(safeTauriCall)
      mockSafeTauriCall.mockRejectedValueOnce(new Error('Backend unavailable'))

      await useProvisioningStore.getState().initialize()

      const { error, isDetecting } = useProvisioningStore.getState()
      expect(error).toBe('Error: Backend unavailable')
      expect(isDetecting).toBe(false)
    })

    it('clears previous error on new initialize', async () => {
      useProvisioningStore.setState({ error: 'previous error' })
      mockDetectTools.mockResolvedValue([])
      mockGetState.mockResolvedValue(null)

      await useProvisioningStore.getState().initialize()

      expect(useProvisioningStore.getState().error).toBeNull()
    })
  })

  describe('detectTools()', () => {
    it('sets isDetecting=true during fetch', async () => {
      let capturedIsDetecting = false
      mockDetectTools.mockImplementation(() => {
        capturedIsDetecting = useProvisioningStore.getState().isDetecting
        return Promise.resolve(mockDetectionResults)
      })

      await useProvisioningStore.getState().detectTools()

      expect(capturedIsDetecting).toBe(true)
    })

    it('updates detectionResults on success', async () => {
      mockDetectTools.mockResolvedValue(mockDetectionResults)

      await useProvisioningStore.getState().detectTools()

      expect(useProvisioningStore.getState().detectionResults).toEqual(
        mockDetectionResults,
      )
      expect(useProvisioningStore.getState().isDetecting).toBe(false)
    })

    it('clears isDetecting after completion', async () => {
      mockDetectTools.mockResolvedValue([])

      await useProvisioningStore.getState().detectTools()

      expect(useProvisioningStore.getState().isDetecting).toBe(false)
    })

    it('sets error on failure and clears isDetecting', async () => {
      const { safeTauriCall } = await import('../lib/tauri')
      const mockSafeTauriCall = vi.mocked(safeTauriCall)
      mockSafeTauriCall.mockRejectedValueOnce(new Error('Detection failed'))

      await useProvisioningStore.getState().detectTools()

      const { error, isDetecting } = useProvisioningStore.getState()
      expect(error).toBe('Error: Detection failed')
      expect(isDetecting).toBe(false)
    })

    it('clears previous error on new detectTools call', async () => {
      useProvisioningStore.setState({ error: 'old error' })
      mockDetectTools.mockResolvedValue([])

      await useProvisioningStore.getState().detectTools()

      expect(useProvisioningStore.getState().error).toBeNull()
    })
  })

  describe('refreshState()', () => {
    it('updates state on success', async () => {
      mockGetState.mockResolvedValue(mockProvisioningState)

      await useProvisioningStore.getState().refreshState()

      expect(useProvisioningStore.getState().state).toEqual(
        mockProvisioningState,
      )
    })

    it('does not touch detectionResults on success', async () => {
      const existingResults = [...mockDetectionResults]
      useProvisioningStore.setState({ detectionResults: existingResults })
      mockGetState.mockResolvedValue(mockProvisioningState)

      await useProvisioningStore.getState().refreshState()

      expect(useProvisioningStore.getState().detectionResults).toEqual(
        existingResults,
      )
    })

    it('sets error on failure', async () => {
      const { safeTauriCall } = await import('../lib/tauri')
      const mockSafeTauriCall = vi.mocked(safeTauriCall)
      mockSafeTauriCall.mockRejectedValueOnce(new Error('State fetch failed'))

      await useProvisioningStore.getState().refreshState()

      expect(useProvisioningStore.getState().error).toBe(
        'Error: State fetch failed',
      )
    })

    it('does not touch detectionResults on failure', async () => {
      const existingResults = [...mockDetectionResults]
      useProvisioningStore.setState({ detectionResults: existingResults })

      const { safeTauriCall } = await import('../lib/tauri')
      const mockSafeTauriCall = vi.mocked(safeTauriCall)
      mockSafeTauriCall.mockRejectedValueOnce(new Error('fail'))

      await useProvisioningStore.getState().refreshState()

      expect(useProvisioningStore.getState().detectionResults).toEqual(
        existingResults,
      )
    })

    it('does not set isDetecting', async () => {
      mockGetState.mockResolvedValue(mockProvisioningState)

      await useProvisioningStore.getState().refreshState()

      expect(useProvisioningStore.getState().isDetecting).toBe(false)
    })
  })
})
