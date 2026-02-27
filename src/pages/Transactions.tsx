import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from "@/components/ui/table";
import { EmptyState } from "@/components/shared/EmptyState";
import { formatCurrency, truncateAddress } from "@/lib/format";
import { ArrowUpDown } from "lucide-react";
import type { Transaction, Agent, TxStatus } from "@/types";

const PAGE_SIZE = 20;

const STATUS_OPTIONS: { value: string; label: string }[] = [
  { value: "", label: "All statuses" },
  { value: "pending", label: "Pending" },
  { value: "executing", label: "Executing" },
  { value: "confirmed", label: "Confirmed" },
  { value: "failed", label: "Failed" },
  { value: "denied", label: "Denied" },
  { value: "awaiting_approval", label: "Awaiting Approval" },
];

interface ListTransactionsResponse {
  transactions: Transaction[];
  total: number;
}

function statusBadgeClass(status: TxStatus): string {
  switch (status) {
    case "confirmed":
      return "bg-green-100 text-green-800 hover:bg-green-100";
    case "executing":
      return "bg-blue-100 text-blue-800 hover:bg-blue-100";
    case "pending":
    case "approved":
      return "bg-yellow-100 text-yellow-800 hover:bg-yellow-100";
    case "failed":
    case "denied":
      return "bg-red-100 text-red-800 hover:bg-red-100";
    default:
      return "bg-gray-100 text-gray-800 hover:bg-gray-100";
  }
}

function statusLabel(status: TxStatus): string {
  switch (status) {
    case "confirmed":
      return "Confirmed";
    case "executing":
      return "Executing";
    case "pending":
      return "Pending";
    case "approved":
      return "Approved";
    case "failed":
      return "Failed";
    case "denied":
      return "Denied";
    default:
      return status;
  }
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export function Transactions() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [statusFilter, setStatusFilter] = useState("");
  const [agentFilter, setAgentFilter] = useState("");
  const [isLoading, setIsLoading] = useState(true);

  const agentMap = new Map(agents.map((a) => [a.id, a.name]));

  const fetchTransactions = useCallback(async () => {
    setIsLoading(true);
    try {
      const params: Record<string, unknown> = {
        limit: PAGE_SIZE,
        offset,
      };
      if (statusFilter) {
        params.status = statusFilter;
      }
      if (agentFilter) {
        params.agent_id = agentFilter;
      }

      const result = await invoke<ListTransactionsResponse>(
        "list_transactions",
        params
      );
      setTransactions(result.transactions);
      setTotal(result.total);
    } catch {
      setTransactions([]);
      setTotal(0);
    } finally {
      setIsLoading(false);
    }
  }, [offset, statusFilter, agentFilter]);

  const fetchAgents = useCallback(async () => {
    try {
      const result = await invoke<Agent[]>("list_agents");
      setAgents(result);
    } catch {
      setAgents([]);
    }
  }, []);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  useEffect(() => {
    fetchTransactions();
  }, [fetchTransactions]);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));
  const currentPage = Math.floor(offset / PAGE_SIZE) + 1;
  const showStart = total > 0 ? offset + 1 : 0;
  const showEnd = Math.min(offset + PAGE_SIZE, total);

  const handlePrev = () => {
    setOffset((prev) => Math.max(0, prev - PAGE_SIZE));
  };

  const handleNext = () => {
    setOffset((prev) => prev + PAGE_SIZE);
  };

  const handleStatusChange = (value: string) => {
    setStatusFilter(value);
    setOffset(0);
  };

  const handleAgentChange = (value: string) => {
    setAgentFilter(value);
    setOffset(0);
  };

  const handleClearFilters = () => {
    setStatusFilter("");
    setAgentFilter("");
    setOffset(0);
  };

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Transaction History</h1>

      {/* Filters */}
      <div className="flex items-center gap-4 flex-wrap">
        <select
          data-testid="status-filter"
          value={statusFilter}
          onChange={(e) => handleStatusChange(e.target.value)}
          className="h-9 rounded-md border bg-background px-3 text-sm"
        >
          {STATUS_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>

        {agents.length > 0 && (
          <select
            data-testid="agent-filter"
            value={agentFilter}
            onChange={(e) => handleAgentChange(e.target.value)}
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            <option value="">All agents</option>
            {agents.map((agent) => (
              <option key={agent.id} value={agent.id}>
                {agent.name}
              </option>
            ))}
          </select>
        )}

        {(statusFilter || agentFilter) && (
          <Button variant="outline" size="sm" onClick={handleClearFilters}>
            Clear
          </Button>
        )}
      </div>

      {/* Table */}
      <Card>
        <CardContent className="p-0">
          {isLoading ? (
            <div className="p-6 text-center text-muted-foreground">
              Loading...
            </div>
          ) : transactions.length === 0 ? (
            <div className="p-6">
              <EmptyState
                title="No transactions yet"
                description="Transactions will appear here"
                icon={ArrowUpDown}
              />
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>Agent</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Recipient</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Description</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {transactions.map((tx) => (
                  <TableRow key={tx.id}>
                    <TableCell>{formatDate(tx.created_at)}</TableCell>
                    <TableCell>
                      {tx.agent_id
                        ? agentMap.get(tx.agent_id) ?? tx.agent_id
                        : "-"}
                    </TableCell>
                    <TableCell>{formatCurrency(tx.amount, tx.asset)}</TableCell>
                    <TableCell>
                      {tx.recipient
                        ? truncateAddress(tx.recipient)
                        : "-"}
                    </TableCell>
                    <TableCell>
                      <span
                        data-testid={`status-badge-${tx.id}`}
                        data-variant="default"
                        className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass(tx.status)}`}
                      >
                        {statusLabel(tx.status)}
                      </span>
                    </TableCell>
                    <TableCell>{tx.description}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Pagination */}
      {total > 0 && (
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">
            Showing {showStart}-{showEnd} of {total}
          </span>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handlePrev}
              disabled={offset === 0}
            >
              Previous
            </Button>
            <span className="text-sm">
              Page {currentPage} of {totalPages}
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={handleNext}
              disabled={offset + PAGE_SIZE >= total}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
