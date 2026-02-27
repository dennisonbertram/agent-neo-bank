import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import type { Agent } from "../types";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../components/ui/card";

const statusColors: Record<string, string> = {
  active: "bg-green-100 text-green-800",
  pending: "bg-yellow-100 text-yellow-800",
  suspended: "bg-orange-100 text-orange-800",
  revoked: "bg-red-100 text-red-800",
};

export function Agents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    invoke<Agent[]>("list_agents")
      .then(setAgents)
      .catch(() => {})
      .finally(() => setIsLoading(false));
  }, []);

  if (isLoading) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Loading agents...</p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Agents</h1>
      {agents.length === 0 ? (
        <p className="text-muted-foreground">No agents registered</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {agents.map((agent) => (
            <Card
              key={agent.id}
              data-testid={`agent-card-${agent.id}`}
              className="cursor-pointer hover:border-accent transition-colors"
              onClick={() => navigate(`/agents/${agent.id}`)}
            >
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{agent.name}</CardTitle>
                  <span
                    data-testid={`status-badge-${agent.id}`}
                    className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${statusColors[agent.status] || "bg-gray-100 text-gray-800"}`}
                  >
                    {agent.status}
                  </span>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground mb-2">
                  {agent.purpose}
                </p>
                <p className="text-xs text-muted-foreground">
                  Type: {agent.agent_type}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
