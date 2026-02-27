import { cn } from "@/lib/utils";

interface ProgressBarProps {
  value: number;
  max: number;
  color?: "indigo" | "green" | "amber" | "red";
  className?: string;
}

const colorMap = {
  indigo: "bg-[#4F46E5]",
  green: "bg-[#10B981]",
  amber: "bg-[#F59E0B]",
  red: "bg-[#EF4444]",
};

export function ProgressBar({ value, max, color = "indigo", className }: ProgressBarProps) {
  const percentage = max > 0 ? Math.min((value / max) * 100, 100) : 0;
  return (
    <div className={cn("h-1.5 w-full overflow-hidden rounded-full bg-[#F0EDE8]", className)}>
      <div
        className={cn("h-full rounded-full transition-all duration-300", colorMap[color])}
        style={{ width: `${percentage}%` }}
      />
    </div>
  );
}
