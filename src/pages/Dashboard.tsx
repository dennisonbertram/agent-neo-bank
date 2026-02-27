import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Link } from "react-router-dom";
import { useBalance } from "@/hooks/useBalance";
import { CurrencyDisplay } from "@/components/shared/CurrencyDisplay";
import { GradientCard } from "@/components/shared/GradientCard";
import { MonoAddress } from "@/components/shared/MonoAddress";
import { ProgressBar } from "@/components/shared/ProgressBar";
import { GlobalBudget } from "@/components/dashboard/GlobalBudget";
import { AgentBudgets } from "@/components/dashboard/AgentBudgets";
import {
  ArrowUpRight,
  Wallet,
  UserPlus,
  Settings,
  Plus,
  Bot,
  ArrowUpDown,
} from "lucide-react";
import type { AgentBudgetSummary, GlobalBudgetSummary, AddressResponse } from "@/types";

function formatBalance(amount: string): string {
  const num = parseFloat(amount);
  if (isNaN(num)) return amount;
  return num.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

function getProgressColor(spent: number, cap: number): "green" | "amber" | "red" {
  if (cap <= 0) return "green";
  const pct = (spent / cap) * 100;
  if (pct > 95) return "red";
  if (pct > 80) return "amber";
  return "green";
}

function AgentCard({ agent }: { agent: AgentBudgetSummary }) {
  const dailySpent = parseFloat(agent.daily_spent) || 0;
  const dailyCap = parseFloat(agent.daily_cap) || 0;

  return (
    <div className="flex flex-col rounded-xl border border-[#F0EDE8] bg-white p-5 transition-shadow hover:shadow-md">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className="flex size-10 items-center justify-center rounded-full bg-[#EEF2FF]">
            <Bot className="size-5 text-[#4F46E5]" />
          </div>
          <div>
            <p className="text-sm font-semibold text-[#1A1A1A]">{agent.agent_name}</p>
            <p className="text-xs text-[#6B7280]">Agent</p>
          </div>
        </div>
      </div>
      <div className="mt-4">
        <ProgressBar
          value={dailySpent}
          max={dailyCap}
          color={getProgressColor(dailySpent, dailyCap)}
        />
        <div className="mt-1.5 flex justify-between text-xs text-[#6B7280]">
          <span>
            <CurrencyDisplay amount={agent.daily_spent} /> spent
          </span>
          <span>
            <CurrencyDisplay amount={agent.daily_cap} /> daily cap
          </span>
        </div>
      </div>
    </div>
  );
}

const quickActions = [
  { label: "Send", icon: ArrowUpRight, to: "/transactions" },
  { label: "Fund", icon: Wallet, to: "/fund" },
  { label: "Invite Agent", icon: UserPlus, to: "/settings" },
  { label: "Settings", icon: Settings, to: "/settings" },
];

export function Dashboard() {
  const { balance, balances, isLoading } = useBalance();
  const [agentBudgets, setAgentBudgets] = useState<AgentBudgetSummary[]>([]);
  const [globalBudget, setGlobalBudget] = useState<GlobalBudgetSummary | null>(
    null,
  );
  const [budgetLoading, setBudgetLoading] = useState(true);
  const [walletAddress, setWalletAddress] = useState<string | null>(null);

  useEffect(() => {
    invoke<AddressResponse>("get_address")
      .then((res) => setWalletAddress(res.address))
      .catch(() => {
        // Address unavailable — MonoAddress will be hidden
      });
  }, []);

  const fetchBudgets = useCallback(async () => {
    setBudgetLoading(true);
    try {
      const [agents, global] = await Promise.all([
        invoke<AgentBudgetSummary[]>("get_agent_budget_summaries"),
        invoke<GlobalBudgetSummary>("get_global_budget_summary"),
      ]);
      setAgentBudgets(agents);
      setGlobalBudget(global);
    } catch {
      // Budget data is non-critical, fail silently
    } finally {
      setBudgetLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBudgets();
  }, [fetchBudgets]);

  // Build secondary balance text from balances map
  const secondaryBalances = balances
    ? Object.entries(balances)
        .filter(([key]) => key !== "USDC")
        .map(([key, val]) => `${val.formatted} ${key}`)
        .join(" | ")
    : null;

  return (
    <div className="space-y-8 p-6">
      {/* Section 1: Page Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-[#1A1A1A]">Dashboard</h1>
      </div>

      {/* Section 2: Balance Card (HERO) */}
      {isLoading ? (
        <div className="flex h-[220px] max-w-[480px] items-center justify-center rounded-2xl bg-[#F9FAFB]">
          <span className="text-sm text-[#6B7280]">Loading...</span>
        </div>
      ) : (
        <GradientCard className="max-w-[480px] min-h-[220px] rounded-2xl p-8">
          {/* Top: wallet address */}
          {walletAddress && (
            <div className="flex items-center gap-2 text-white/80">
              <MonoAddress
                address={walletAddress}
                className="text-xs text-white/80"
              />
            </div>
          )}

          {/* Center: Hero balance */}
          <div className="mt-6">
            <div
              className="text-5xl font-bold tracking-tight"
              style={{ fontFeatureSettings: '"tnum"' }}
            >
              {balance ? `$${formatBalance(balance)}` : "--"}
            </div>
            <div className="mt-1 text-sm text-white/80">USDC</div>
          </div>

          {/* Secondary balances */}
          {secondaryBalances && (
            <div className="mt-2 text-sm text-white/70">{secondaryBalances}</div>
          )}

          {/* Bottom: Fund button */}
          <div className="mt-6">
            <Link
              to="/fund"
              className="inline-flex items-center gap-2 rounded-lg bg-white px-4 py-2 text-sm font-medium text-[#4F46E5] transition-colors hover:bg-white/90"
            >
              <Wallet className="size-4" />
              Fund Wallet
            </Link>
          </div>
        </GradientCard>
      )}

      {/* Section 3: Quick Actions */}
      <div className="flex flex-wrap gap-3">
        {quickActions.map((action) => (
          <Link
            key={action.label}
            to={action.to}
            className="inline-flex items-center gap-2 rounded-full border border-[#E8E5E0] bg-white px-4 py-2 text-sm font-medium text-[#1A1A1A] transition-colors hover:bg-[#F9FAFB]"
          >
            <action.icon className="size-4 text-[#6B7280]" />
            {action.label}
          </Link>
        ))}
      </div>

      {/* Global Budget Utilization (preserved from original) */}
      {budgetLoading ? (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
          <h2 className="mb-4 text-lg font-semibold text-[#1A1A1A]">
            Global Budget Utilization
          </h2>
          <span className="text-sm text-[#6B7280]">Loading...</span>
        </div>
      ) : (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
          <h2 className="mb-4 text-lg font-semibold text-[#1A1A1A]">
            Global Budget Utilization
          </h2>
          <GlobalBudget summary={globalBudget} />
        </div>
      )}

      {/* Section 4: Your Agents */}
      <div>
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-[#1A1A1A]">Your Agents</h2>
          <Link
            to="/agents"
            className="text-sm font-medium text-[#4F46E5] hover:text-[#4338CA]"
          >
            View all
          </Link>
        </div>

        {budgetLoading ? (
          <span className="text-sm text-[#6B7280]">Loading...</span>
        ) : agentBudgets.length > 0 ? (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {agentBudgets.map((agent) => (
              <AgentCard key={agent.agent_id} agent={agent} />
            ))}
            <Link
              to="/settings"
              className="flex min-h-[160px] flex-col items-center justify-center rounded-xl border-2 border-dashed border-[#E8E5E0] bg-white p-5 text-center transition-colors hover:border-[#4F46E5] hover:bg-[#EEF2FF]"
            >
              <Plus className="size-8 text-[#9CA3AF]" />
              <span className="mt-2 text-sm font-medium text-[#6B7280]">
                Add Agent
              </span>
            </Link>
          </div>
        ) : (
          <div className="rounded-xl border border-[#F0EDE8] bg-white p-8 text-center">
            <Bot className="mx-auto size-12 text-[#D1D5DB]" />
            <p className="mt-3 text-sm font-medium text-[#1A1A1A]">
              No agents yet
            </p>
            <p className="mt-1 text-sm text-[#6B7280]">
              Generate an invitation code to let an AI agent connect.
            </p>
            <Link
              to="/settings"
              className="mt-4 inline-flex items-center gap-2 rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[#4338CA]"
            >
              <UserPlus className="size-4" />
              Invite Agent
            </Link>
          </div>
        )}
      </div>

      {/* Agent Budgets (preserved from original) */}
      {!budgetLoading && agentBudgets.length > 0 && (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
          <h2 className="mb-4 text-lg font-semibold text-[#1A1A1A]">
            Agent Budgets
          </h2>
          <AgentBudgets summaries={agentBudgets} />
        </div>
      )}

      {/* Section 5: Recent Transactions */}
      <div>
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-[#1A1A1A]">
            Recent Transactions
          </h2>
          <Link
            to="/transactions"
            className="text-sm font-medium text-[#4F46E5] hover:text-[#4338CA]"
          >
            View all
          </Link>
        </div>
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-8 text-center">
          <ArrowUpDown className="mx-auto size-12 text-[#D1D5DB]" />
          <p className="mt-3 text-sm font-medium text-[#1A1A1A]">
            No transactions yet
          </p>
          <p className="mt-1 text-sm text-[#6B7280]">
            Transactions will appear here once your agents start spending.
          </p>
        </div>
      </div>
    </div>
  );
}
