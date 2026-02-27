export function Header() {
  return (
    <header className="flex h-14 items-center justify-between border-b border-border px-6">
      <div className="text-sm font-medium">Dashboard</div>
      <div className="text-sm text-muted-foreground">Balance: --</div>
    </header>
  );
}
