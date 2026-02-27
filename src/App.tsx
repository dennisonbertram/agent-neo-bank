import { Routes, Route } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { Onboarding } from "./pages/Onboarding";
import { Agents } from "./pages/Agents";
import { Transactions } from "./pages/Transactions";
import { Settings } from "./pages/Settings";
import { Shell } from "./components/layout/Shell";

export function App() {
  return (
    <Routes>
      <Route path="/onboarding" element={<Onboarding />} />
      <Route element={<Shell />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/agents" element={<Agents />} />
        <Route path="/transactions" element={<Transactions />} />
        <Route path="/settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}
