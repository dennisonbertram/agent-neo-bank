import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { formatCurrency, truncateAddress } from "@/lib/format";
import { ArrowUpDown, Download, Search } from "lucide-react";
import type { Transaction, Agent, TxStatus, TxType } from "@/types";

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

function statusDotColor(status: TxStatus): string {
  switch (status) {
    case "confirmed":
      return "bg-[#10B981]";
    case "executing":
      return "bg-[#3B82F6]";
    case "pending":
    case "approved":
      return "bg-[#F59E0B]";
    case "failed":
    case "denied":
      return "bg-[#EF4444]";
    default:
      return "bg-[#9CA3AF]";
  }
}

function statusBadgeClass(status: TxStatus): string {
  switch (status) {
    case "confirmed":
      return "bg-green-100 text-green-800";
    case "executing":
      return "bg-blue-100 text-blue-800";
    case "pending":
    case "approved":
      return "bg-yellow-100 text-yellow-800";
    case "failed":
    case "denied":
      return "bg-red-100 text-red-800";
    default:
      return "bg-gray-100 text-gray-800";
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

function typeBadgeClass(txType: TxType): string {
  switch (txType) {
    case "send":
      return "bg-[#EEF2FF] text-[#4F46E5]";
    case "receive":
      return "bg-[#ECFDF5] text-[#10B981]";
    case "earn":
      return "bg-[#FFFBEB] text-[#F59E0B]";
    default:
      return "bg-gray-100 text-gray-800";
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

function StatusDot({ status }: { status: TxStatus }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-[#6B7280]">
      <span className={`size-1.5 rounded-full ${statusDotColor(status)}`} />
      {statusLabel(status)}
    </span>
  );
}

export function Transactions() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const [statusFilter, setStatusFilter] = useState("");
  const [agentFilter, setAgentFilter] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
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
    setSearchQuery("");
    setOffset(0);
  };

  // Client-side search filtering
  const filteredTransactions = searchQuery
    ? transactions.filter((tx) => {
        const query = searchQuery.toLowerCase();
        const agentName = tx.agent_id ? agentMap.get(tx.agent_id) ?? "" : "";
        return (
          (tx.description ?? "").toLowerCase().includes(query) ||
          agentName.toLowerCase().includes(query) ||
          (tx.recipient ?? "").toLowerCase().includes(query) ||
          tx.amount.includes(query)
        );
      })
    : transactions;

  return (
    <div className="space-y-6 p-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-[#1A1A1A]">Transactions</h1>
        <button className="inline-flex items-center gap-2 rounded-lg border border-[#E8E5E0] bg-white px-4 py-2 text-sm font-medium text-[#1A1A1A] hover:bg-[#F9FAFB]">
          <Download className="size-4" />
          Export CSV
        </button>
      </div>

      {/* Filter Bar */}
      <div className="rounded-xl border border-[#F0EDE8] bg-white p-4">
        <div className="flex flex-wrap items-center gap-3">
          {/* Status filter - preserving select for test compatibility */}
          <select
            data-testid="status-filter"
            value={statusFilter}
            onChange={(e) => handleStatusChange(e.target.value)}
            className="h-9 rounded-lg border border-[#E8E5E0] bg-white px-3 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
          >
            {STATUS_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          {/* Agent filter */}
          {agents.length > 0 && (
            <select
              data-testid="agent-filter"
              value={agentFilter}
              onChange={(e) => handleAgentChange(e.target.value)}
              className="h-9 rounded-lg border border-[#E8E5E0] bg-white px-3 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
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
            <button
              onClick={handleClearFilters}
              className="rounded-lg border border-[#E8E5E0] bg-white px-3 py-1.5 text-sm font-medium text-[#6B7280] hover:bg-[#F9FAFB] hover:text-[#1A1A1A]"
            >
              Clear
            </button>
          )}

          {/* Search */}
          <div className="relative ml-auto">
            <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[#9CA3AF]" />
            <input
              type="text"
              placeholder="Search transactions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="h-9 rounded-lg border border-[#E8E5E0] bg-white pl-9 pr-4 text-sm placeholder:text-[#9CA3AF] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
            />
          </div>
        </div>
      </div>

      {/* Data Table */}
      {isLoading ? (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-16 text-center text-sm text-[#6B7280]">
          Loading...
        </div>
      ) : filteredTransactions.length === 0 && transactions.length === 0 ? (
        <div className="rounded-xl border border-[#F0EDE8] bg-white">
          <div className="flex flex-col items-center py-16">
            <ArrowUpDown className="size-12 text-[#9CA3AF]" />
            <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">
              No transactions yet
            </h3>
            <p className="mt-1 text-sm text-[#6B7280]">
              Transactions will appear here once your agents start spending.
            </p>
          </div>
        </div>
      ) : filteredTransactions.length === 0 ? (
        <div className="rounded-xl border border-[#F0EDE8] bg-white">
          <div className="flex flex-col items-center py-16">
            <Search className="size-12 text-[#9CA3AF]" />
            <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">
              No matching transactions
            </h3>
            <p className="mt-1 text-sm text-[#6B7280]">
              Try adjusting your search or filters.
            </p>
          </div>
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-[#F0EDE8] bg-white">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[#F0EDE8] bg-[#F9FAFB]">
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Date
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Agent
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Type
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Amount
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Recipient
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Status
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-[#6B7280]">
                  Description
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#F0EDE8]">
              {filteredTransactions.map((tx) => (
                <tr
                  key={tx.id}
                  className="transition-colors hover:bg-[#F9FAFB]"
                  style={{ height: 52 }}
                >
                  <td className="px-4 py-3 text-sm text-[#6B7280]">
                    {formatDate(tx.created_at)}
                  </td>
                  <td className="px-4 py-3 text-sm font-medium text-[#1A1A1A]">
                    {tx.agent_id
                      ? agentMap.get(tx.agent_id) ?? tx.agent_id
                      : "—"}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${typeBadgeClass(tx.tx_type)}`}
                    >
                      {tx.tx_type}
                    </span>
                  </td>
                  <td
                    className="px-4 py-3 text-sm font-mono"
                    style={{ fontFeatureSettings: '"tnum"' }}
                  >
                    <span
                      className={
                        tx.tx_type === "send"
                          ? "text-[#EF4444]"
                          : "text-[#10B981]"
                      }
                    >
                      {tx.tx_type === "send" ? "−" : "+"}
                      {formatCurrency(tx.amount, tx.asset)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm font-mono text-[#6B7280]">
                    {tx.recipient ? truncateAddress(tx.recipient) : "—"}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      data-testid={`status-badge-${tx.id}`}
                      data-variant="default"
                      className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium ${statusBadgeClass(tx.status)}`}
                    >
                      <span
                        className={`size-1.5 rounded-full ${statusDotColor(tx.status)}`}
                      />
                      {statusLabel(tx.status)}
                    </span>
                  </td>
                  <td className="max-w-[200px] truncate px-4 py-3 text-sm text-[#6B7280]">
                    {tx.description || "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {/* Pagination inside table card */}
          {total > 0 && (
            <div className="flex items-center justify-between border-t border-[#F0EDE8] px-4 py-3">
              <span className="text-sm text-[#6B7280]">
                Showing {showStart}-{showEnd} of {total}
              </span>
              <div className="flex items-center gap-2">
                <span className="text-sm text-[#6B7280]">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={handlePrev}
                  disabled={offset === 0}
                  className="rounded-lg border border-[#E8E5E0] px-3 py-1.5 text-sm font-medium text-[#1A1A1A] transition-colors hover:bg-[#F9FAFB] disabled:opacity-50"
                >
                  Previous
                </button>
                <button
                  onClick={handleNext}
                  disabled={offset + PAGE_SIZE >= total}
                  className="rounded-lg bg-[#4F46E5] px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-[#4338CA] disabled:opacity-50"
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Pagination outside table for empty search results */}
      {!isLoading &&
        filteredTransactions.length === 0 &&
        transactions.length > 0 &&
        total > 0 && (
          <div className="flex items-center justify-between">
            <span className="text-sm text-[#6B7280]">
              Showing {showStart}-{showEnd} of {total}
            </span>
            <div className="flex items-center gap-2">
              <button
                onClick={handlePrev}
                disabled={offset === 0}
                className="rounded-lg border border-[#E8E5E0] px-3 py-1.5 text-sm font-medium text-[#1A1A1A] disabled:opacity-50"
              >
                Previous
              </button>
              <button
                onClick={handleNext}
                disabled={offset + PAGE_SIZE >= total}
                className="rounded-lg bg-[#4F46E5] px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50"
              >
                Next
              </button>
            </div>
          </div>
        )}
    </div>
  );
}
