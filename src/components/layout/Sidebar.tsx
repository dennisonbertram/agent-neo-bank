import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Bot,
  ArrowUpDown,
  CheckCircle,
  Wallet,
  Settings,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
}

const navItems: NavItem[] = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard, end: true },
  { to: "/agents", label: "Agents", icon: Bot },
  { to: "/transactions", label: "Transactions", icon: ArrowUpDown },
  { to: "/approvals", label: "Approvals", icon: CheckCircle },
  { to: "/fund", label: "Fund", icon: Wallet },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  return (
    <aside className="flex h-screen w-60 flex-col border-r border-[#E8E5E0] bg-white">
      {/* Logo */}
      <div className="flex items-center gap-2.5 px-5 pt-6 pb-5">
        <div className="flex size-8 items-center justify-center rounded-lg bg-[#4F46E5]">
          <Wallet className="size-4 text-white" />
        </div>
        <span className="text-lg font-semibold text-[#1A1A1A]">Agent Neo Bank</span>
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 px-3">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors ${
                isActive
                  ? "bg-[#EEF2FF] font-semibold text-[#4F46E5]"
                  : "text-[#6B7280] hover:bg-[#F9FAFB] hover:text-[#1A1A1A]"
              }`
            }
          >
            <item.icon className="size-5" />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* User Profile */}
      <div className="border-t border-[#E8E5E0] px-4 py-4">
        <div className="flex items-center gap-3">
          <div className="flex size-8 items-center justify-center rounded-full bg-[#EEF2FF] text-sm font-medium text-[#4F46E5]">
            W
          </div>
          <div className="min-w-0">
            <p className="truncate text-sm font-medium text-[#1A1A1A]">Wallet</p>
            <p className="truncate text-xs text-[#9CA3AF]">Connected</p>
          </div>
        </div>
      </div>
    </aside>
  );
}
