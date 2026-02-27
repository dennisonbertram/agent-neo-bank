interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span className="rounded-full bg-secondary px-2 py-1 text-xs">
      {status}
    </span>
  );
}
