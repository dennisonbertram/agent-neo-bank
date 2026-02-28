import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'
import { useAuthStore } from './stores/authStore'

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
  const { checkAuthStatus, isLoading } = useAuthStore()

  useEffect(() => {
    checkAuthStatus()
  }, [checkAuthStatus])

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
      <Route path="/transactions/:txId" element={<ProtectedRoute><TransactionDetail /></ProtectedRoute>} />
      <Route path="/stats" element={<ProtectedRoute><Stats /></ProtectedRoute>} />
      <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />

      {/* Default redirect */}
      <Route path="/" element={<DefaultRedirect />} />
      <Route path="*" element={<DefaultRedirect />} />
    </Routes>
  )
}
