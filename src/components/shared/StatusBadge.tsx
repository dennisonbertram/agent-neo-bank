import { cn } from "@/lib/utils";

interface StatusBadgeProps {
  status: "active" | "pending" | "suspended" | "revoked";
  className?: string;
}

const statusConfig = {
  active: { dot: "bg-[#10B981]", text: "text-[#10B981]", bg: "bg-[#ECFDF5]", label: "Active" },
  pending: { dot: "bg-[#F59E0B] animate-pulse", text: "text-[#F59E0B]", bg: "bg-[#FFFBEB]", label: "Pending" },
  suspended: { dot: "bg-[#EF4444]", text: "text-[#EF4444]", bg: "bg-[#FEF2F2]", label: "Suspended" },
  revoked: { dot: "bg-[#6B7280]", text: "text-[#6B7280]", bg: "bg-[#F9FAFB]", label: "Revoked" },
};

export function StatusBadge({ status, className }: StatusBadgeProps) {
  const config = statusConfig[status];
  return (
    <span className={cn("inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium", config.bg, config.text, className)}>
      <span className={cn("size-1.5 rounded-full", config.dot)} />
      {config.label}
    </span>
  );
}
