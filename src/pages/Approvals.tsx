import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ApprovalRequest, Agent } from "../types";
import { Bot, Check, X, CheckCircle } from "lucide-react";

interface TransactionPayload {
  tx_id?: string;
  to?: string;
  amount?: string;
  asset?: string;
}

interface LimitIncreasePayload {
  proposed_daily?: string;
  proposed_monthly?: string;
}

interface RegistrationPayload {
  agent_name?: string;
}

type ParsedPayload = TransactionPayload & LimitIncreasePayload & RegistrationPayload;

function parsePayload(payload: string): ParsedPayload {
  try {
    return JSON.parse(payload) as ParsedPayload;
  } catch {
    return {};
  }
}

function formatTimeAgo(timestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;
  if (diff < 60) return "Just now";
  const minutes = Math.floor(diff / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function truncateAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function Approvals() {
  const [approvals, setApprovals] = useState<ApprovalRequest[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [filter, setFilter] = useState<string>("pending");
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [processingId, setProcessingId] = useState<string | null>(null);
  const [resolveError, setResolveError] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<{ id: string; decision: string } | null>(null);
  const approvalRequestRef = useRef(0);

  const loadApprovals = useCallback(async () => {
    const requestId = ++approvalRequestRef.current;
    setIsLoading(true);
    setError(null);
    try {
      const args =
        filter === "pending" ? { status: "pending" } : {};
      const result = await invoke<ApprovalRequest[]>("list_approvals", args);
      if (approvalRequestRef.current !== requestId) return;
      setApprovals(result);
    } catch {
      if (approvalRequestRef.current !== requestId) return;
      setError("Couldn't load approvals");
    } finally {
      if (approvalRequestRef.current === requestId) {
        setIsLoading(false);
      }
    }
  }, [filter]);

  const loadAgents = useCallback(async () => {
    try {
      const result = await invoke<Agent[]>("list_agents");
      setAgents(result);
    } catch {
      // silently handle
    }
  }, []);

  useEffect(() => {
    loadApprovals();
    loadAgents();
  }, [loadApprovals, loadAgents]);

  const getAgentName = (agentId: string): string => {
    const agent = agents.find((a) => a.id === agentId);
    return agent?.name ?? agentId;
  };

  const handleResolve = async (approvalId: string, decision: string) => {
    setProcessingId(approvalId);
    setResolveError(null);
    try {
      await invoke("resolve_approval", {
        approval_id: approvalId,
        decision,
      });
      setConfirmAction(null);
      await loadApprovals();
    } catch (err) {
      setResolveError(
        `Failed to ${decision === "approved" ? "approve" : "deny"}: ${err instanceof Error ? err.message : String(err)}`
      );
    } finally {
      setProcessingId(null);
    }
  };

  const pendingCount = approvals.filter((a) => a.status === "pending").length;

  const renderPayloadDetails = (approval: ApprovalRequest) => {
    const data = parsePayload(approval.payload);

    if (approval.request_type === "transaction") {
      return (
        <div className="mt-4">
          {data.amount && data.asset && (
            <p className="text-xl font-semibold text-[#1A1A1A]">
              {data.amount} {data.asset}
            </p>
          )}
          {data.to && (
            <p className="mt-1 text-sm font-mono text-[#6B7280]">
              {truncateAddress(data.to)}
            </p>
          )}
        </div>
      );
    }

    if (approval.request_type === "limit_increase") {
      return (
        <div className="mt-4 space-y-1">
          {data.proposed_daily && (
            <p className="text-sm text-[#6B7280]">
              Proposed daily limit: <span className="font-semibold text-[#1A1A1A]">{data.proposed_daily}</span>
            </p>
          )}
          {data.proposed_monthly && (
            <p className="text-sm text-[#6B7280]">
              Proposed monthly limit: <span className="font-semibold text-[#1A1A1A]">{data.proposed_monthly}</span>
            </p>
          )}
        </div>
      );
    }

    if (approval.request_type === "registration") {
      return (
        <div className="mt-4">
          {data.agent_name && (
            <p className="text-sm text-[#6B7280]">
              Agent: <span className="font-semibold text-[#1A1A1A]">{data.agent_name}</span>
            </p>
          )}
        </div>
      );
    }

    return null;
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-[#1A1A1A]">Approvals</h1>
          <p className="mt-1 text-sm text-[#F59E0B] font-medium">{pendingCount} pending</p>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => setFilter("pending")}
            className={`rounded-lg px-4 py-2 text-sm font-medium transition-colors ${
              filter === "pending"
                ? "bg-[#4F46E5] text-white"
                : "border border-[#F0EDE8] bg-white text-[#6B7280] hover:bg-[#F9FAFB]"
            }`}
          >
            Pending
          </button>
          <button
            type="button"
            onClick={() => setFilter("all")}
            className={`rounded-lg px-4 py-2 text-sm font-medium transition-colors ${
              filter === "all"
                ? "bg-[#4F46E5] text-white"
                : "border border-[#F0EDE8] bg-white text-[#6B7280] hover:bg-[#F9FAFB]"
            }`}
          >
            All
          </button>
        </div>
      </div>

      {!isLoading && error ? (
        <div className="flex flex-col items-center py-16 text-center">
          <div className="flex size-16 items-center justify-center rounded-full bg-[#FEF2F2]">
            <X className="size-8 text-[#EF4444]" />
          </div>
          <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">{error}</h3>
          <button
            type="button"
            onClick={loadApprovals}
            className="mt-4 rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA]"
          >
            Retry
          </button>
        </div>
      ) : !isLoading && approvals.length === 0 ? (
        <div className="flex flex-col items-center py-16 text-center">
          <div className="flex size-16 items-center justify-center rounded-full bg-[#ECFDF5]">
            <CheckCircle className="size-8 text-[#10B981]" />
          </div>
          <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">All caught up!</h3>
          <p className="mt-1 text-sm text-[#6B7280]">No pending approvals right now.</p>
        </div>
      ) : (
        <div className="space-y-4">
          {approvals.map((approval) => (
            <div
              key={approval.id}
              className="rounded-xl border border-[#F0EDE8] bg-white p-6"
              style={{ borderLeft: "3px solid #F59E0B" }}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex size-10 items-center justify-center rounded-full bg-[#EEF2FF]">
                    <Bot className="size-5 text-[#4F46E5]" />
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-[#1A1A1A]">
                      {getAgentName(approval.agent_id)}
                    </p>
                    <p className="text-xs text-[#9CA3AF]">
                      {formatTimeAgo(approval.created_at)}
                    </p>
                  </div>
                </div>
                <span className="rounded-full bg-[#FEF3C7] px-2.5 py-0.5 text-xs font-medium text-[#92400E]">
                  {approval.request_type}
                </span>
              </div>

              {renderPayloadDetails(approval)}

              {approval.status === "pending" && (
                <div className="mt-4">
                  {resolveError && processingId === null && confirmAction?.id === approval.id && (
                    <div className="mb-3 rounded-lg bg-[#FEF2F2] px-4 py-2 text-sm text-[#EF4444]">
                      {resolveError}
                    </div>
                  )}
                  {confirmAction?.id === approval.id ? (
                    <div className="flex items-center gap-3">
                      <span className="text-sm text-[#6B7280]">
                        {confirmAction.decision === "approved" ? "Approve" : "Deny"} this request?
                      </span>
                      <button
                        type="button"
                        onClick={() => handleResolve(approval.id, confirmAction.decision)}
                        disabled={processingId === approval.id}
                        className={`inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium text-white disabled:opacity-50 ${
                          confirmAction.decision === "approved" ? "bg-[#10B981] hover:bg-[#059669]" : "bg-[#EF4444] hover:bg-[#DC2626]"
                        }`}
                      >
                        {processingId === approval.id ? "Processing..." : "Confirm"}
                      </button>
                      <button
                        type="button"
                        onClick={() => { setConfirmAction(null); setResolveError(null); }}
                        disabled={processingId === approval.id}
                        className="rounded-lg border border-[#E8E5E0] px-3 py-1.5 text-sm font-medium text-[#6B7280] hover:bg-[#F9FAFB]"
                      >
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <div className="flex items-center gap-3">
                      <button
                        type="button"
                        onClick={() => setConfirmAction({ id: approval.id, decision: "approved" })}
                        disabled={processingId !== null}
                        className="inline-flex items-center gap-2 rounded-lg bg-[#10B981] px-4 py-2 text-sm font-medium text-white hover:bg-[#059669] disabled:opacity-50"
                      >
                        <Check className="size-4" />
                        Approve
                      </button>
                      <button
                        type="button"
                        onClick={() => setConfirmAction({ id: approval.id, decision: "denied" })}
                        disabled={processingId !== null}
                        className="inline-flex items-center gap-2 rounded-lg border border-[#EF4444] px-4 py-2 text-sm font-medium text-[#EF4444] hover:bg-[#FEF2F2] disabled:opacity-50"
                      >
                        <X className="size-4" />
                        Deny
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
