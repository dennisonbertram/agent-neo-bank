A bottom tab bar wastes 84px of vertical space and implies a flat hierarchy, which feels wrong for a compact, drill-down wallet interface. By absorbing secondary routes (Stats, Settings) into the `TopBar` and relying on Home's existing segment control for Agents and balance card for adding funds, we reclaim screen real estate and align with standard compact fintech patterns (like Phantom or Mercury).

```tsx
// src/App.tsx
import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'
import { useAuthStore } from './stores/authStore'
import { useWalletStore } from './stores/walletStore'

// Auth flow pages
import Onboarding from './pages/Onboarding'
import InstallSkill from './pages/InstallSkill'
import ConnectCoinbase from './pages/ConnectCoinbase'
import VerifyOtp from './pages/VerifyOtp'

// Main app pages
import Home from './pages/Home'
import AddFunds from './pages/AddFunds'
import AgentsList from './pages/AgentsList'
import AgentDetail from './pages/AgentDetail'
import TransactionDetail from './pages/TransactionDetail'
import Settings from './pages/Settings'
import Stats from './pages/Stats'
import AllTransactions from './pages/AllTransactions'

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore()
  if (!isAuthenticated) return <Navigate to="/onboarding" replace />
  return <>{children}</>
}

function DefaultRedirect() {
  const { isAuthenticated } = useAuthStore()
  return <Navigate to={isAuthenticated ? '/home' : '/onboarding'} replace />
}

function SplashScreen() {
  return (
    <div className="flex items-center justify-center min-h-screen bg-[var(--bg-primary)]">
      <div className="animate-pulse text-[var(--text-secondary)] text-lg">Loading...</div>
    </div>
  )
}

export function App() {
  const { checkAuthStatus, isLoading, isAuthenticated } = useAuthStore()
  const { initialize: initWallet, teardown: teardownWallet } = useWalletStore()

  useEffect(() => {
    checkAuthStatus()
  }, [checkAuthStatus])

  useEffect(() => {
    if (isAuthenticated) {
      initWallet()
    } else {
      teardownWallet()
    }
  }, [isAuthenticated, initWallet, teardownWallet])

  if (isLoading) return <SplashScreen />

  return (
    <Routes>
      {/* Onboarding flow — Wave 2 */}
      <Route path="/onboarding" element={<Onboarding />} />
      <Route path="/setup/install" element={<InstallSkill />} />
      <Route path="/setup/connect" element={<ConnectCoinbase />} />
      <Route path="/setup/verify" element={<VerifyOtp />} />

      {/* Main app — requires auth */}
      <Route path="/home" element={<ProtectedRoute><Home /></ProtectedRoute>} />
      <Route path="/add-funds" element={<ProtectedRoute><AddFunds /></ProtectedRoute>} />
      <Route path="/agents" element={<ProtectedRoute><AgentsList /></ProtectedRoute>} />
      <Route path="/agents/:agentId" element={<ProtectedRoute><AgentDetail /></ProtectedRoute>} />
      <Route path="/transactions" element={<ProtectedRoute><AllTransactions /></ProtectedRoute>} />
      <Route path="/transactions/:txId" element={<ProtectedRoute><TransactionDetail /></ProtectedRoute>} />
      <Route path="/stats" element={<ProtectedRoute><Stats /></ProtectedRoute>} />
      <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />

      {/* Default redirect */}
      <Route path="/" element={<DefaultRedirect />} />
      <Route path="*" element={<DefaultRedirect />} />
    </Routes>
  )
}
```

```tsx
// src/components/layout/TopBar.tsx
import { Settings, Activity } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { cn } from '../../lib/cn'

interface TopBarProps {
  walletName?: string
  initials?: string
  className?: string
}

export function TopBar({ walletName = 'Tally Wallet', initials = 'DB', className }: TopBarProps) {
  const navigate = useNavigate()

  return (
    <div
      className={cn(
        'flex items-center justify-between px-6 pt-6 pb-2',
        className
      )}
    >
      <div className="flex items-center gap-3">
        <div className="w-[36px] h-[36px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[14px] font-semibold border border-[var(--brand-main)]/30">
          {initials}
        </div>
        <span className="text-[16px] font-semibold text-[var(--text-primary)]">{walletName}</span>
      </div>
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => navigate('/stats')}
          className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
          aria-label="Stats"
        >
          <Activity size={18} className="text-[var(--text-secondary)]" />
        </button>
        <button
          type="button"
          onClick={() => navigate('/settings')}
          className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
          aria-label="Settings"
        >
          <Settings size={18} className="text-[var(--text-secondary)]" />
        </button>
      </div>
    </div>
  )
}
```
