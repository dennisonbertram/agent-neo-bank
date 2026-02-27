import { cn } from "@/lib/utils";

const statusColors: Record<string, string> = {
  pending: "bg-yellow-500/20 text-yellow-400",
  active: "bg-green-500/20 text-green-400",
  suspended: "bg-red-500/20 text-red-400",
  revoked: "bg-zinc-500/20 text-zinc-400",
  confirmed: "bg-green-500/20 text-green-400",
  failed: "bg-red-500/20 text-red-400",
  denied: "bg-red-500/20 text-red-400",
};

interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-1 text-xs font-medium capitalize",
        statusColors[status] ?? "bg-zinc-500/20 text-zinc-400"
      )}
    >
      {status}
    </span>
  );
}
