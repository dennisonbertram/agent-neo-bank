import { Routes, Route } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { Onboarding } from "./pages/Onboarding";
import { Agents } from "./pages/Agents";
import { AgentDetail } from "./pages/AgentDetail";
import { Transactions } from "./pages/Transactions";
import { Settings } from "./pages/Settings";
import { Approvals } from "./pages/Approvals";
import { Shell } from "./components/layout/Shell";

export function App() {
  return (
    <Routes>
      <Route path="/onboarding" element={<Onboarding />} />
      <Route element={<Shell />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/agents" element={<Agents />} />
        <Route path="/agents/:id" element={<AgentDetail />} />
        <Route path="/transactions" element={<Transactions />} />
        <Route path="/approvals" element={<Approvals />} />
        <Route path="/settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}
