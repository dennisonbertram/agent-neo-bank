import { BudgetProgress } from "./BudgetProgress";
import type { AgentBudgetSummary } from "@/types";

export function AgentBudgets({
  summaries,
}: {
  summaries: AgentBudgetSummary[];
}) {
  if (summaries.length === 0) {
    return (
      <p className="text-muted-foreground text-sm">No agents registered</p>
    );
  }
  return (
    <div className="space-y-4">
      {summaries.map((s) => (
        <div key={s.agent_id} className="border rounded-lg p-3 space-y-2">
          <p className="text-sm font-medium">{s.agent_name}</p>
          <BudgetProgress
            label="Daily"
            spent={s.daily_spent}
            cap={s.daily_cap}
          />
          <BudgetProgress
            label="Weekly"
            spent={s.weekly_spent}
            cap={s.weekly_cap}
          />
          <BudgetProgress
            label="Monthly"
            spent={s.monthly_spent}
            cap={s.monthly_cap}
          />
        </div>
      ))}
    </div>
  );
}
