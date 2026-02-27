import { cn } from "@/lib/utils";
import type { ReactNode } from "react";

interface GradientCardProps {
  children: ReactNode;
  className?: string;
}

export function GradientCard({ children, className }: GradientCardProps) {
  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-2xl p-6 text-white shadow-lg transition-all duration-200 hover:-translate-y-0.5 hover:shadow-xl",
        className
      )}
      style={{
        background: "linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%)",
      }}
    >
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-br from-white/10 via-transparent to-transparent" />
      <div className="relative">{children}</div>
    </div>
  );
}
