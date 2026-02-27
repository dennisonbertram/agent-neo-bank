import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GlobalPolicy } from "../../types";

export function GlobalPolicySettings() {
  const [policy, setPolicy] = useState<GlobalPolicy | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [showKillConfirm, setShowKillConfirm] = useState(false);

  const loadPolicy = () => {
    invoke<GlobalPolicy>("get_global_policy")
      .then(setPolicy)
      .catch(() => {})
      .finally(() => setIsLoading(false));
  };

  useEffect(() => {
    loadPolicy();
  }, []);

  const handleCapChange = (field: keyof GlobalPolicy, value: string) => {
    if (!policy) return;
    setPolicy({ ...policy, [field]: value });
  };

  const handleSaveCaps = async () => {
    if (!policy) return;
    setIsSaving(true);
    try {
      await invoke("update_global_policy", {
        policy: {
          ...policy,
          updated_at: Math.floor(Date.now() / 1000),
        },
      });
    } catch {
      // handle error
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleKillSwitch = async () => {
    if (!policy) return;
    if (!policy.kill_switch_active) {
      // Activating - show confirmation
      setShowKillConfirm(true);
      return;
    }
    // Deactivating - no confirmation needed
    try {
      await invoke("toggle_kill_switch", { active: false });
      loadPolicy();
    } catch {
      // handle error
    }
  };

  const handleConfirmKillSwitch = async () => {
    try {
      await invoke("toggle_kill_switch", {
        active: true,
        reason: "Manual activation",
      });
      setShowKillConfirm(false);
      loadPolicy();
    } catch {
      // handle error
    }
  };

  if (isLoading) return null;
  if (!policy) return null;

  return (
    <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
      <h2 className="text-base font-semibold text-[#1A1A1A] mb-1">Global Policy</h2>
      <p className="text-sm text-[#6B7280] mb-6">
        Wallet-level spending controls and kill switch
      </p>

      <div className="space-y-4">
        <h3 className="text-sm font-medium text-[#1A1A1A]">Spending Caps</h3>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="daily-cap">
            Daily Cap (USDC)
          </label>
          <input
            id="daily-cap"
            type="number"
            value={policy.daily_cap}
            onChange={(e) => handleCapChange("daily_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="weekly-cap">
            Weekly Cap (USDC)
          </label>
          <input
            id="weekly-cap"
            type="number"
            value={policy.weekly_cap}
            onChange={(e) => handleCapChange("weekly_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="monthly-cap">
            Monthly Cap (USDC)
          </label>
          <input
            id="monthly-cap"
            type="number"
            value={policy.monthly_cap}
            onChange={(e) => handleCapChange("monthly_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="min-reserve">
            Minimum Reserve Balance (USDC)
          </label>
          <input
            id="min-reserve"
            type="number"
            value={policy.min_reserve_balance}
            onChange={(e) =>
              handleCapChange("min_reserve_balance", e.target.value)
            }
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
        </div>
      </div>

      <div className="mt-6">
        <button
          type="button"
          onClick={handleSaveCaps}
          disabled={isSaving}
          className="rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA] disabled:opacity-50"
        >
          {isSaving ? "Saving..." : "Save Caps"}
        </button>
      </div>

      {/* Kill Switch */}
      <div className="mt-6 border-t border-[#F0EDE8] pt-6">
        <h3 className="text-sm font-medium text-[#EF4444]">Kill Switch</h3>
        <p className="text-xs text-[#6B7280] mb-4">
          Emergency stop: blocks ALL agent transactions immediately
        </p>
        <button
          type="button"
          onClick={handleToggleKillSwitch}
          className={
            policy.kill_switch_active
              ? "rounded-lg border border-[#E5E7EB] bg-white px-4 py-2 text-sm font-medium text-[#374151] hover:bg-[#F9FAFB]"
              : "rounded-lg bg-[#EF4444] px-4 py-2 text-sm font-medium text-white hover:bg-[#DC2626]"
          }
        >
          {policy.kill_switch_active
            ? "Deactivate Kill Switch"
            : "Activate Kill Switch"}
        </button>

        {/* Confirmation dialog for activation */}
        {showKillConfirm && (
          <div className="mt-4 rounded-lg border border-[#FCA5A5] bg-[#FEF2F2] p-4">
            <p className="text-sm text-[#991B1B] mb-3">
              Are you sure? This will immediately block ALL agent
              transactions.
            </p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleConfirmKillSwitch}
                className="rounded-lg bg-[#EF4444] px-4 py-2 text-sm font-medium text-white hover:bg-[#DC2626]"
              >
                Confirm Activation
              </button>
              <button
                type="button"
                onClick={() => setShowKillConfirm(false)}
                className="rounded-lg border border-[#E5E7EB] bg-white px-4 py-2 text-sm font-medium text-[#374151] hover:bg-[#F9FAFB]"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
