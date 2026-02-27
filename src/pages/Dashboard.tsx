import { useBalance } from "@/hooks/useBalance";
import { CurrencyDisplay } from "@/components/shared/CurrencyDisplay";
import { EmptyState } from "@/components/shared/EmptyState";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Bot, ArrowUpDown } from "lucide-react";

export function Dashboard() {
  const { balance, isLoading } = useBalance();

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Dashboard</h1>

      <Card>
        <CardHeader>
          <CardTitle>Balance</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <span className="text-muted-foreground">Loading...</span>
          ) : balance ? (
            <span className="text-3xl font-bold">
              <CurrencyDisplay amount={balance} />
            </span>
          ) : (
            <span className="text-muted-foreground">--</span>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Your Agents</CardTitle>
        </CardHeader>
        <CardContent>
          <EmptyState
            title="No agents registered yet"
            description="Register an agent to get started"
            icon={Bot}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Transactions</CardTitle>
        </CardHeader>
        <CardContent>
          <EmptyState
            title="No transactions yet"
            description="Transactions will appear here"
            icon={ArrowUpDown}
          />
        </CardContent>
      </Card>
    </div>
  );
}
