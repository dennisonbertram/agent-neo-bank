import { NavLink } from "react-router-dom";

export function Sidebar() {
  return (
    <aside className="flex w-64 flex-col border-r border-border bg-sidebar-background p-4">
      <div className="mb-8 text-lg font-bold text-sidebar-foreground">
        Agent Neo Bank
      </div>
      <nav className="flex flex-col gap-2">
        <NavLink
          to="/"
          className={({ isActive }) =>
            `rounded-md px-3 py-2 text-sm ${
              isActive
                ? "bg-sidebar-accent text-sidebar-accent-foreground"
                : "text-sidebar-foreground hover:bg-sidebar-accent/50"
            }`
          }
        >
          Dashboard
        </NavLink>
      </nav>
    </aside>
  );
}
