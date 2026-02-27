import { useState, useEffect, useCallback } from "react";
import { useParams, Link } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ChevronRight, Bot, Pause, RefreshCw, X } from "lucide-react";
import type { Agent, SpendingPolicy, Transaction, AgentBudgetSummary } from "../types";
import { StatusBadge } from "../components/shared/StatusBadge";
import { ProgressBar } from "../components/shared/ProgressBar";
import { MonoAddress } from "../components/shared/MonoAddress";
import { Input } from "../components/ui/input";

export function AgentDetail() {
  const { id } = useParams<{ id: string }>();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [policy, setPolicy] = useState<SpendingPolicy | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [budget, setBudget] = useState<AgentBudgetSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isEditing, setIsEditing] = useState(false);
  const [editPolicy, setEditPolicy] = useState<SpendingPolicy | null>(null);

  const loadData = useCallback(async () => {
    if (!id) return;
    setIsLoading(true);
    try {
      const [agentData, policyData, txData, budgetSummaries] = await Promise.all([
        invoke<Agent>("get_agent", { agentId: id }),
        invoke<SpendingPolicy>("get_agent_spending_policy", { agentId: id }),
        invoke<Transaction[]>("get_agent_transactions", {
          agentId: id,
          limit: 20,
        }),
        invoke<AgentBudgetSummary[]>("get_agent_budget_summaries").catch(() => [] as AgentBudgetSummary[]),
      ]);
      setAgent(agentData);
      setPolicy(policyData);
      setEditPolicy(policyData);
      setTransactions(txData);
      const agentBudget = budgetSummaries.find((b) => b.agent_id === id) ?? null;
      setBudget(agentBudget);
    } catch {
      // Error loading data
    } finally {
      setIsLoading(false);
    }
  }, [id]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSuspend = async () => {
    if (!id) return;
    await invoke("suspend_agent", { agentId: id });
    await loadData();
  };

  const handleSaveLimits = async () => {
    if (!editPolicy) return;
    await invoke("update_agent_spending_policy", { policy: editPolicy });
    setPolicy(editPolicy);
    setIsEditing(false);
  };

  const handleEditChange = (field: keyof SpendingPolicy, value: string) => {
    if (!editPolicy) return;
    setEditPolicy({ ...editPolicy, [field]: value });
  };

  const toggleEdit = () => {
    if (isEditing) {
      handleSaveLimits();
    } else {
      setIsEditing(true);
    }
  };

  const formatDate = (timestamp: number | null) => {
    if (!timestamp) return "Never";
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  const formatTime = (timestamp: number | null) => {
    if (!timestamp) return "";
    const d = new Date(timestamp * 1000);
    return d.toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getStatusDotColor = (status: string) => {
    switch (status) {
      case "confirmed":
        return "bg-[#10B981]";
      case "pending":
      case "awaiting_approval":
        return "bg-[#F59E0B]";
      case "failed":
      case "denied":
        return "bg-[#EF4444]";
      default:
        return "bg-[#9CA3AF]";
    }
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <p className="text-[#6B7280]">Loading agent details...</p>
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="p-6">
        <p className="text-[#6B7280]">Agent not found</p>
      </div>
    );
  }

  const policyRows = policy
    ? [
        {
          label: "Per Transaction",
          limit: parseFloat(policy.per_tx_max) || 0,
          spent: 0,
        },
        {
          label: "Daily",
          limit: parseFloat(policy.daily_cap) || 0,
          spent: budget ? parseFloat(budget.daily_spent) || 0 : 0,
        },
        {
          label: "Weekly",
          limit: parseFloat(policy.weekly_cap) || 0,
          spent: budget ? parseFloat(budget.weekly_spent) || 0 : 0,
        },
        {
          label: "Monthly",
          limit: parseFloat(policy.monthly_cap) || 0,
          spent: budget ? parseFloat(budget.monthly_spent) || 0 : 0,
        },
      ]
    : [];

  const getColor = (row: { spent: number; limit: number }): "indigo" | "green" | "amber" | "red" => {
    if (row.limit === 0) return "indigo";
    const ratio = row.spent / row.limit;
    if (ratio >= 0.9) return "red";
    if (ratio >= 0.7) return "amber";
    return "indigo";
  };

  const recipients = policy?.allowlist ?? [];

  return (
    <div className="p-6 space-y-4">
      {/* Breadcrumb + Header */}
      <div className="space-y-4">
        <nav className="flex items-center gap-2 text-sm text-[#6B7280]">
          <Link to="/agents" className="hover:text-[#4F46E5]">
            Agents
          </Link>
          <ChevronRight className="size-4" />
          <span className="text-[#1A1A1A] font-medium">{agent.name}</span>
        </nav>

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="flex size-12 items-center justify-center rounded-full bg-[#EEF2FF]">
              <Bot className="size-6 text-[#4F46E5]" />
            </div>
            <div>
              <div className="flex items-center gap-3">
                <h1 className="text-2xl font-semibold text-[#1A1A1A]">{agent.name}</h1>
                <StatusBadge status={agent.status} />
              </div>
              <p className="text-sm text-[#6B7280]">
                {agent.purpose} · Created {formatDate(agent.created_at)}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            {agent.status === "active" && (
              <button
                onClick={handleSuspend}
                className="inline-flex items-center gap-2 rounded-lg border border-[#EF4444] px-4 py-2 text-sm font-medium text-[#EF4444] hover:bg-[#FEF2F2]"
              >
                <Pause className="size-4" />
                Suspend Agent
              </button>
            )}
            <button
              disabled
              title="Coming soon — available in Phase 4"
              className="inline-flex items-center gap-2 rounded-lg border border-[#E8E5E0] px-4 py-2 text-sm font-medium text-[#1A1A1A] opacity-50 cursor-not-allowed"
            >
              <RefreshCw className="size-4" />
              Rotate Token
            </button>
          </div>
        </div>
      </div>

      {/* Two-Column Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-8">
        <div className="lg:col-span-2 space-y-6">
          {/* Card 1: Spending Limits */}
          <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-base font-semibold text-[#1A1A1A]">Spending Limits</h2>
              <button
                onClick={toggleEdit}
                className="text-sm font-medium text-[#4F46E5] hover:text-[#4338CA]"
              >
                {isEditing ? "Save" : "Edit"}
              </button>
            </div>

            {isEditing && editPolicy ? (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label htmlFor="per_tx_max" className="text-sm text-[#6B7280]">
                      Per Transaction Max
                    </label>
                    <Input
                      id="per_tx_max"
                      value={editPolicy.per_tx_max}
                      onChange={(e) => handleEditChange("per_tx_max", e.target.value)}
                    />
                  </div>
                  <div>
                    <label htmlFor="daily_cap" className="text-sm text-[#6B7280]">
                      Daily Cap
                    </label>
                    <Input
                      id="daily_cap"
                      value={editPolicy.daily_cap}
                      onChange={(e) => handleEditChange("daily_cap", e.target.value)}
                    />
                  </div>
                  <div>
                    <label htmlFor="weekly_cap" className="text-sm text-[#6B7280]">
                      Weekly Cap
                    </label>
                    <Input
                      id="weekly_cap"
                      value={editPolicy.weekly_cap}
                      onChange={(e) => handleEditChange("weekly_cap", e.target.value)}
                    />
                  </div>
                  <div>
                    <label htmlFor="monthly_cap" className="text-sm text-[#6B7280]">
                      Monthly Cap
                    </label>
                    <Input
                      id="monthly_cap"
                      value={editPolicy.monthly_cap}
                      onChange={(e) => handleEditChange("monthly_cap", e.target.value)}
                    />
                  </div>
                </div>
              </div>
            ) : (
              <div className="space-y-5">
                {policyRows.map((row) => (
                  <div key={row.label}>
                    <div className="flex items-center justify-between mb-1.5">
                      <span className="text-sm text-[#6B7280]">{row.label}</span>
                      <span className="text-sm font-medium font-mono text-[#1A1A1A]">
                        ${row.spent} / ${row.limit}
                      </span>
                    </div>
                    <ProgressBar value={row.spent} max={row.limit} color={getColor(row)} />
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Card 2: Allowed Recipients */}
          <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
            <h2 className="text-base font-semibold text-[#1A1A1A] mb-4">Allowed Recipients</h2>
            {recipients.length > 0 ? (
              <div className="space-y-3">
                {recipients.map((addr) => (
                  <div
                    key={addr}
                    className="flex items-center justify-between rounded-lg bg-[#F9FAFB] px-3 py-2"
                  >
                    <MonoAddress address={addr} />
                    <button className="text-[#9CA3AF] hover:text-[#EF4444]">
                      <X className="size-4" />
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-[#9CA3AF]">All recipients allowed</p>
            )}
          </div>
        </div>

        {/* Card 3: Activity Feed */}
        <div>
          <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
            <h2 className="text-base font-semibold text-[#1A1A1A] mb-4">Activity</h2>
            {transactions.length === 0 ? (
              <p className="text-sm text-[#9CA3AF]">No transactions</p>
            ) : (
              <div className="space-y-4">
                {transactions.map((tx) => (
                  <div key={tx.id} className="flex items-start gap-3">
                    <div className={`mt-1 size-2 rounded-full ${getStatusDotColor(tx.status)}`} />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium text-[#1A1A1A]">${tx.amount}</span>
                        <span className="text-xs text-[#9CA3AF]">{formatTime(tx.created_at)}</span>
                      </div>
                      <p className="text-xs text-[#6B7280] truncate">
                        {tx.description || tx.recipient}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
