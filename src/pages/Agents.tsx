import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Link } from "react-router-dom";
import { Search, Plus, Bot } from "lucide-react";
import type { Agent } from "../types";
import { StatusBadge } from "../components/shared/StatusBadge";

export function Agents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [activeTab, setActiveTab] = useState<string>("all");

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

  const filteredAgents = agents.filter((agent) => {
    const matchesTab = activeTab === "all" || agent.status === activeTab;
    const matchesSearch =
      !search || agent.name.toLowerCase().includes(search.toLowerCase());
    return matchesTab && matchesSearch;
  });

  const getCount = (tab: string) => {
    if (tab === "all") return agents.length;
    return agents.filter((a) => a.status === tab).length;
  };

  return (
    <div className="space-y-6 p-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold text-[#1A1A1A]">Agents</h1>
        <div className="flex items-center gap-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[#9CA3AF]" />
            <input
              type="text"
              placeholder="Search agents..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-10 rounded-lg border border-[#E8E5E0] bg-white pl-10 pr-4 text-sm text-[#1A1A1A] placeholder:text-[#9CA3AF] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
            />
          </div>
          <Link
            to="/settings"
            className="inline-flex items-center gap-2 rounded-lg bg-[#4F46E5] px-4 py-2.5 text-sm font-medium text-white hover:bg-[#4338CA]"
          >
            <Plus className="size-4" />
            Generate Invitation Code
          </Link>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-1 border-b border-[#F0EDE8]">
        {[
          { key: "all", label: "All" },
          { key: "active", label: "Active" },
          { key: "pending", label: "Pending" },
          { key: "suspended", label: "Suspended" },
        ].map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`px-4 py-2.5 text-sm font-medium transition-colors ${
              activeTab === tab.key
                ? "border-b-2 border-[#4F46E5] text-[#4F46E5]"
                : "text-[#6B7280] hover:text-[#1A1A1A]"
            }`}
          >
            {tab.label}
            <span className="ml-1.5 rounded-full bg-[#F0EDE8] px-1.5 py-0.5 text-xs">
              {getCount(tab.key)}
            </span>
          </button>
        ))}
      </div>

      {/* Agent Cards Grid or Empty State */}
      {filteredAgents.length === 0 && agents.length === 0 ? (
        <div className="flex flex-col items-center py-16 text-center">
          <Bot className="size-12 text-[#9CA3AF]" />
          <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">
            No agents yet
          </h3>
          <p className="mt-1 text-sm text-[#6B7280]">
            Generate an invitation code to let an AI agent connect to your
            wallet.
          </p>
        </div>
      ) : filteredAgents.length === 0 ? (
        <div className="flex flex-col items-center py-16 text-center">
          <Bot className="size-12 text-[#9CA3AF]" />
          <h3 className="mt-4 text-lg font-medium text-[#1A1A1A]">
            No matching agents
          </h3>
          <p className="mt-1 text-sm text-[#6B7280]">
            Try adjusting your search or filter criteria.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredAgents.map((agent) => (
            <Link
              key={agent.id}
              to={`/agents/${agent.id}`}
              data-testid={`agent-card-${agent.id}`}
              className="group rounded-xl border border-[#F0EDE8] bg-white p-5 transition-all hover:-translate-y-0.5 hover:shadow-md"
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex size-10 items-center justify-center rounded-full bg-[#EEF2FF]">
                    <Bot className="size-5 text-[#4F46E5]" />
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold text-[#1A1A1A]">
                      {agent.name}
                    </h3>
                    <p className="max-w-[180px] truncate text-xs text-[#9CA3AF]">
                      {agent.purpose || "General purpose agent"}
                    </p>
                  </div>
                </div>
                <span data-testid={`status-badge-${agent.id}`}>
                  <StatusBadge status={agent.status} />
                </span>
              </div>
            </Link>
          ))}

          {/* Add Agent card */}
          <Link
            to="/settings"
            className="flex min-h-[140px] flex-col items-center justify-center rounded-xl border-2 border-dashed border-[#E8E5E0] bg-white p-5 text-center transition-colors hover:border-[#4F46E5] hover:bg-[#EEF2FF]"
          >
            <Plus className="size-8 text-[#9CA3AF]" />
            <span className="mt-2 text-sm font-medium text-[#6B7280]">
              Add New Agent
            </span>
          </Link>
        </div>
      )}
    </div>
  );
}
