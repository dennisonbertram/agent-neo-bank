interface BudgetProgressProps {
  label: string;
  spent: string;
  cap: string;
}

export function BudgetProgress({ label, spent, cap }: BudgetProgressProps) {
  const spentNum = parseFloat(spent) || 0;
  const capNum = parseFloat(cap) || 0;
  const percentage = capNum > 0 ? Math.min((spentNum / capNum) * 100, 100) : 0;
  const isWarning = percentage > 80;
  const isDanger = percentage > 95;

  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs">
        <span className="text-muted-foreground">{label}</span>
        <span>
          {spent} / {cap} USDC
        </span>
      </div>
      <div className="h-2 rounded-full bg-muted overflow-hidden">
        <div
          className={`h-full rounded-full transition-all ${
            isDanger
              ? "bg-red-500"
              : isWarning
                ? "bg-yellow-500"
                : "bg-green-500"
          }`}
          style={{ width: `${percentage}%` }}
          data-testid="progress-bar"
        />
      </div>
    </div>
  );
}
