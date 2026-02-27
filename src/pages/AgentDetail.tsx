import { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import type { Agent, SpendingPolicy, Transaction } from "../types";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Input } from "../components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";

export function AgentDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [agent, setAgent] = useState<Agent | null>(null);
  const [policy, setPolicy] = useState<SpendingPolicy | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isEditing, setIsEditing] = useState(false);
  const [editPolicy, setEditPolicy] = useState<SpendingPolicy | null>(null);

  const loadData = useCallback(async () => {
    if (!id) return;
    setIsLoading(true);
    try {
      const [agentData, policyData, txData] = await Promise.all([
        invoke<Agent>("get_agent", { agentId: id }),
        invoke<SpendingPolicy>("get_agent_spending_policy", { agentId: id }),
        invoke<Transaction[]>("get_agent_transactions", {
          agentId: id,
          limit: 20,
        }),
      ]);
      setAgent(agentData);
      setPolicy(policyData);
      setEditPolicy(policyData);
      setTransactions(txData);
    } catch {
      // Error loading data
    } finally {
      setIsLoading(false);
    }
  }, [id]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSuspend = async () => {
    if (!id) return;
    await invoke("suspend_agent", { agentId: id });
    await loadData();
  };

  const handleRevoke = async () => {
    if (!id) return;
    await invoke("revoke_agent", { agentId: id });
    await loadData();
  };

  const handleSaveLimits = async () => {
    if (!editPolicy) return;
    await invoke("update_agent_spending_policy", { policy: editPolicy });
    setPolicy(editPolicy);
    setIsEditing(false);
  };

  const handleEditChange = (field: keyof SpendingPolicy, value: string) => {
    if (!editPolicy) return;
    setEditPolicy({ ...editPolicy, [field]: value });
  };

  const formatDate = (timestamp: number | null) => {
    if (!timestamp) return "Never";
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading agent details...</p>
      </div>
    );
  }

  if (!agent) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Agent not found</p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => navigate("/agents")}
          >
            Back
          </Button>
          <h1 className="text-2xl font-bold mt-2">{agent.name}</h1>
          <p className="text-muted-foreground">{agent.purpose}</p>
        </div>
        <div className="flex gap-2">
          {agent.status === "active" && (
            <Button variant="outline" onClick={handleSuspend}>
              Suspend
            </Button>
          )}
          {agent.status !== "revoked" && (
            <Button variant="destructive" onClick={handleRevoke}>
              Revoke
            </Button>
          )}
        </div>
      </div>

      {/* Agent info card */}
      <Card>
        <CardHeader>
          <CardTitle>Agent Profile</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <dt className="text-muted-foreground">Type</dt>
              <dd>{agent.agent_type}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">Status</dt>
              <dd>{agent.status}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">Registered</dt>
              <dd>{formatDate(agent.created_at)}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">Last Active</dt>
              <dd>{formatDate(agent.last_active_at)}</dd>
            </div>
          </dl>
        </CardContent>
      </Card>

      {/* Spending limits card with edit mode */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Spending Limits</CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                if (isEditing) {
                  setEditPolicy(policy);
                }
                setIsEditing(!isEditing);
              }}
            >
              {isEditing ? "Cancel" : "Edit"}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {policy && !isEditing && (
            <dl className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <dt className="text-muted-foreground">Per Transaction Max</dt>
                <dd>{policy.per_tx_max}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Daily Cap</dt>
                <dd>{policy.daily_cap}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Weekly Cap</dt>
                <dd>{policy.weekly_cap}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Monthly Cap</dt>
                <dd>{policy.monthly_cap}</dd>
              </div>
            </dl>
          )}
          {isEditing && editPolicy && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label
                    htmlFor="per_tx_max"
                    className="text-sm text-muted-foreground"
                  >
                    Per Transaction Max
                  </label>
                  <Input
                    id="per_tx_max"
                    value={editPolicy.per_tx_max}
                    onChange={(e) =>
                      handleEditChange("per_tx_max", e.target.value)
                    }
                  />
                </div>
                <div>
                  <label
                    htmlFor="daily_cap"
                    className="text-sm text-muted-foreground"
                  >
                    Daily Cap
                  </label>
                  <Input
                    id="daily_cap"
                    value={editPolicy.daily_cap}
                    onChange={(e) =>
                      handleEditChange("daily_cap", e.target.value)
                    }
                  />
                </div>
                <div>
                  <label
                    htmlFor="weekly_cap"
                    className="text-sm text-muted-foreground"
                  >
                    Weekly Cap
                  </label>
                  <Input
                    id="weekly_cap"
                    value={editPolicy.weekly_cap}
                    onChange={(e) =>
                      handleEditChange("weekly_cap", e.target.value)
                    }
                  />
                </div>
                <div>
                  <label
                    htmlFor="monthly_cap"
                    className="text-sm text-muted-foreground"
                  >
                    Monthly Cap
                  </label>
                  <Input
                    id="monthly_cap"
                    value={editPolicy.monthly_cap}
                    onChange={(e) =>
                      handleEditChange("monthly_cap", e.target.value)
                    }
                  />
                </div>
              </div>
              <Button onClick={handleSaveLimits}>Save Limits</Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Recent transactions */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Activity</CardTitle>
        </CardHeader>
        <CardContent>
          {transactions.length === 0 ? (
            <p className="text-muted-foreground">No transactions</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Description</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Date</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {transactions.map((tx) => (
                  <TableRow key={tx.id}>
                    <TableCell>{tx.description}</TableCell>
                    <TableCell>{tx.amount}</TableCell>
                    <TableCell>{tx.status}</TableCell>
                    <TableCell>{formatDate(tx.created_at)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
