import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBalance } from "@/hooks/useBalance";
import { CurrencyDisplay } from "@/components/shared/CurrencyDisplay";
import { EmptyState } from "@/components/shared/EmptyState";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { AgentBudgets } from "@/components/dashboard/AgentBudgets";
import { GlobalBudget } from "@/components/dashboard/GlobalBudget";
import { Bot, ArrowUpDown } from "lucide-react";
import type { AgentBudgetSummary, GlobalBudgetSummary } from "@/types";

export function Dashboard() {
  const { balance, isLoading } = useBalance();
  const [agentBudgets, setAgentBudgets] = useState<AgentBudgetSummary[]>([]);
  const [globalBudget, setGlobalBudget] = useState<GlobalBudgetSummary | null>(
    null,
  );
  const [budgetLoading, setBudgetLoading] = useState(true);

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

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Dashboard</h1>

      <Card>
        <CardHeader>
          <CardTitle>Balance</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <span className="text-muted-foreground">Loading...</span>
          ) : balance ? (
            <span className="text-3xl font-bold">
              <CurrencyDisplay amount={balance} />
            </span>
          ) : (
            <span className="text-muted-foreground">--</span>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Global Budget Utilization</CardTitle>
        </CardHeader>
        <CardContent>
          {budgetLoading ? (
            <span className="text-muted-foreground">Loading...</span>
          ) : (
            <GlobalBudget summary={globalBudget} />
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Agent Budgets</CardTitle>
        </CardHeader>
        <CardContent>
          {budgetLoading ? (
            <span className="text-muted-foreground">Loading...</span>
          ) : (
            <AgentBudgets summaries={agentBudgets} />
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Your Agents</CardTitle>
        </CardHeader>
        <CardContent>
          <EmptyState
            title="No agents registered yet"
            description="Register an agent to get started"
            icon={Bot}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Transactions</CardTitle>
        </CardHeader>
        <CardContent>
          <EmptyState
            title="No transactions yet"
            description="Transactions will appear here"
            icon={ArrowUpDown}
          />
        </CardContent>
      </Card>
    </div>
  );
}
