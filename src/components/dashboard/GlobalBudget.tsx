import { BudgetProgress } from "./BudgetProgress";
import type { GlobalBudgetSummary } from "@/types";

export function GlobalBudget({
  summary,
}: {
  summary: GlobalBudgetSummary | null;
}) {
  if (!summary) return null;
  return (
    <div className="space-y-3">
      {summary.kill_switch_active && (
        <div className="p-2 rounded bg-red-100 text-red-800 text-sm font-medium">
          Kill Switch Active — All transactions blocked
        </div>
      )}
      <BudgetProgress
        label="Daily Global"
        spent={summary.daily_spent}
        cap={summary.daily_cap}
      />
      <BudgetProgress
        label="Weekly Global"
        spent={summary.weekly_spent}
        cap={summary.weekly_cap}
      />
      <BudgetProgress
        label="Monthly Global"
        spent={summary.monthly_spent}
        cap={summary.monthly_cap}
      />
    </div>
  );
}
