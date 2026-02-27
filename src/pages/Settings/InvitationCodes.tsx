import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { InvitationCode } from "../../types";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

function getCodeStatus(
  code: InvitationCode
): "active" | "used" | "expired" {
  if (code.expires_at && code.expires_at < Date.now() / 1000) return "expired";
  if (code.use_count >= code.max_uses) return "used";
  return "active";
}

const statusConfig = {
  active: { label: "Active", className: "bg-green-100 text-green-800" },
  used: { label: "Used", className: "bg-gray-100 text-gray-800" },
  expired: { label: "Expired", className: "bg-red-100 text-red-800" },
} as const;

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString();
}

export function InvitationCodes() {
  const [codes, setCodes] = useState<InvitationCode[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [label, setLabel] = useState("");

  const loadCodes = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<InvitationCode[]>("list_invitation_codes");
      setCodes(result);
    } catch {
      // silently handle - codes will remain empty
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCodes();
  }, [loadCodes]);

  const handleGenerate = async () => {
    try {
      await invoke<InvitationCode>("generate_invitation_code", {
        label,
      });
      setLabel("");
      setIsDialogOpen(false);
      await loadCodes();
    } catch {
      // silently handle error
    }
  };

  const handleRevoke = async (code: string) => {
    try {
      await invoke("revoke_invitation_code", { code });
      await loadCodes();
    } catch {
      // silently handle error
    }
  };

  return (
    <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
      <div className="flex items-center justify-between mb-1">
        <h2 className="text-base font-semibold text-[#1A1A1A]">Invitation Codes</h2>
        <button
          type="button"
          onClick={() => setIsDialogOpen(true)}
          className="rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA]"
        >
          Generate Code
        </button>
      </div>
      <p className="text-sm text-[#6B7280] mb-6">
        Manage invitation codes for agent registration
      </p>

      {!isLoading && codes.length === 0 ? (
        <p className="text-[#6B7280] text-center py-8 text-sm">
          No invitation codes
        </p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#F0EDE8]">
                <th className="py-3 pr-4 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider">Code</th>
                <th className="py-3 pr-4 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider">Label</th>
                <th className="py-3 pr-4 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider">Status</th>
                <th className="py-3 pr-4 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider">Uses</th>
                <th className="py-3 pr-4 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider">Created</th>
                <th className="py-3 text-left text-xs font-medium text-[#9CA3AF] uppercase tracking-wider" />
              </tr>
            </thead>
            <tbody className="divide-y divide-[#F0EDE8]">
              {codes.map((code) => {
                const status = getCodeStatus(code);
                const config = statusConfig[status];
                return (
                  <tr key={code.code}>
                    <td className="py-3 pr-4 font-mono text-[#1A1A1A]">{code.code}</td>
                    <td className="py-3 pr-4 text-[#374151]">{code.label}</td>
                    <td className="py-3 pr-4">
                      <span
                        data-testid={`status-badge-${code.code}`}
                        className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${config.className}`}
                      >
                        {config.label}
                      </span>
                    </td>
                    <td className="py-3 pr-4 text-[#6B7280]">
                      {code.use_count} / {code.max_uses}
                    </td>
                    <td className="py-3 pr-4 text-[#6B7280]">{formatDate(code.created_at)}</td>
                    <td className="py-3">
                      {status === "active" && (
                        <button
                          type="button"
                          onClick={() => handleRevoke(code.code)}
                          className="rounded-lg border border-[#EF4444] px-3 py-1.5 text-xs font-medium text-[#EF4444] hover:bg-[#FEF2F2]"
                        >
                          Revoke
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Generate Invitation Code</DialogTitle>
            <DialogDescription>
              Create a new invitation code for agent registration.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label htmlFor="code-label" className="text-sm font-medium text-[#374151]">
                Label
              </label>
              <input
                id="code-label"
                placeholder="Label for this code"
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                className="block w-full rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] placeholder:text-[#9CA3AF] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
              />
            </div>
          </div>
          <DialogFooter>
            <button
              type="button"
              onClick={() => setIsDialogOpen(false)}
              className="rounded-lg border border-[#E5E7EB] bg-white px-4 py-2 text-sm font-medium text-[#374151] hover:bg-[#F9FAFB]"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleGenerate}
              disabled={!label.trim()}
              className="rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA] disabled:opacity-50"
            >
              Generate
            </button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
