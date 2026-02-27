import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ApprovalRequest, Agent } from "../types";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Badge } from "../components/ui/badge";

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

function formatTimeRemaining(expiresAt: number): string {
  const now = Math.floor(Date.now() / 1000);
  const remaining = expiresAt - now;
  if (remaining <= 0) return "Expired";
  const hours = Math.floor(remaining / 3600);
  const minutes = Math.floor((remaining % 3600) / 60);
  if (hours > 24) {
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h remaining`;
  }
  if (hours > 0) return `${hours}h ${minutes}m remaining`;
  return `${minutes}m remaining`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

const statusConfig = {
  pending: { label: "Pending", className: "bg-yellow-100 text-yellow-800" },
  approved: { label: "Approved", className: "bg-green-100 text-green-800" },
  denied: { label: "Denied", className: "bg-red-100 text-red-800" },
  expired: { label: "Expired", className: "bg-gray-100 text-gray-800" },
} as const;

export function Approvals() {
  const [approvals, setApprovals] = useState<ApprovalRequest[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [filter, setFilter] = useState<string>("pending");
  const [isLoading, setIsLoading] = useState(true);

  const loadApprovals = useCallback(async () => {
    setIsLoading(true);
    try {
      const args =
        filter === "pending" ? { status: "pending" } : {};
      const result = await invoke<ApprovalRequest[]>("list_approvals", args);
      setApprovals(result);
    } catch {
      // silently handle
    } finally {
      setIsLoading(false);
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
    try {
      await invoke("resolve_approval", {
        approval_id: approvalId,
        decision,
      });
      await loadApprovals();
    } catch {
      // silently handle
    }
  };

  const renderPayloadDetails = (approval: ApprovalRequest) => {
    const data = parsePayload(approval.payload);

    if (approval.request_type === "transaction") {
      return (
        <div className="text-sm text-muted-foreground space-y-1">
          {data.to && <p>To: <span className="font-mono">{data.to}</span></p>}
          {data.amount && data.asset && (
            <p>
              Amount: {data.amount} {data.asset}
            </p>
          )}
        </div>
      );
    }

    if (approval.request_type === "limit_increase") {
      return (
        <div className="text-sm text-muted-foreground space-y-1">
          {data.proposed_daily && <p>Proposed daily limit: {data.proposed_daily}</p>}
          {data.proposed_monthly && (
            <p>Proposed monthly limit: {data.proposed_monthly}</p>
          )}
        </div>
      );
    }

    if (approval.request_type === "registration") {
      return (
        <div className="text-sm text-muted-foreground">
          {data.agent_name && <p>Agent: {data.agent_name}</p>}
        </div>
      );
    }

    return null;
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Approvals</h1>
        <div className="flex gap-2">
          <Button
            variant={filter === "pending" ? "default" : "outline"}
            size="sm"
            onClick={() => setFilter("pending")}
          >
            Pending
          </Button>
          <Button
            variant={filter === "all" ? "default" : "outline"}
            size="sm"
            onClick={() => setFilter("all")}
          >
            All
          </Button>
        </div>
      </div>

      {!isLoading && approvals.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground">No pending approvals</p>
        </div>
      ) : (
        <div className="grid gap-4">
          {approvals.map((approval) => {
            const config = statusConfig[approval.status];
            return (
              <Card key={approval.id}>
                <CardHeader className="pb-3">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-base">
                      {getAgentName(approval.agent_id)}
                    </CardTitle>
                    <div className="flex items-center gap-2">
                      <Badge variant="outline">{approval.request_type}</Badge>
                      <Badge variant="outline" className={config.className}>
                        {config.label}
                      </Badge>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-3">
                  {renderPayloadDetails(approval)}
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>Created: {formatDate(approval.created_at)}</span>
                    <span>{formatTimeRemaining(approval.expires_at)}</span>
                  </div>
                  {approval.status === "pending" && (
                    <div className="flex gap-2 pt-2">
                      <Button
                        size="sm"
                        onClick={() => handleResolve(approval.id, "approved")}
                      >
                        Approve
                      </Button>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleResolve(approval.id, "denied")}
                      >
                        Deny
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
