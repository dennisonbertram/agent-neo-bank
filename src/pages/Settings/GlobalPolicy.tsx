import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GlobalPolicy } from "../../types";

export function GlobalPolicySettings() {
  const [policy, setPolicy] = useState<GlobalPolicy | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [showKillConfirm, setShowKillConfirm] = useState(false);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [killSwitchError, setKillSwitchError] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const loadPolicy = () => {
    setLoadError(null);
    invoke<GlobalPolicy>("get_global_policy")
      .then(setPolicy)
      .catch((err) => {
        setLoadError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => setIsLoading(false));
  };

  useEffect(() => {
    loadPolicy();
  }, []);

  const handleCapChange = (field: keyof GlobalPolicy, value: string) => {
    if (!policy) return;
    setPolicy({ ...policy, [field]: value });
    if (validationErrors[field]) {
      setValidationErrors((prev) => {
        const next = { ...prev };
        delete next[field];
        return next;
      });
    }
  };

  const handleSaveCaps = async () => {
    if (!policy) return;

    // Validate numeric fields
    const errors: Record<string, string> = {};
    for (const field of ["daily_cap", "weekly_cap", "monthly_cap", "min_reserve_balance"] as const) {
      const val = policy[field];
      const num = parseFloat(val as string);
      if (val === "" || val === undefined || isNaN(num)) {
        errors[field] = "Must be a valid number";
      } else if (num < 0) {
        errors[field] = "Must be >= 0";
      }
    }
    if (Object.keys(errors).length > 0) {
      setValidationErrors(errors);
      return;
    }
    setValidationErrors({});

    setSaveError(null);
    setIsSaving(true);
    try {
      await invoke("update_global_policy", {
        policy: {
          ...policy,
          daily_cap: parseFloat(policy.daily_cap as string).toString(),
          weekly_cap: parseFloat(policy.weekly_cap as string).toString(),
          monthly_cap: parseFloat(policy.monthly_cap as string).toString(),
          min_reserve_balance: parseFloat(policy.min_reserve_balance as string).toString(),
          updated_at: Math.floor(Date.now() / 1000),
        },
      });
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
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
    setKillSwitchError(null);
    try {
      await invoke("toggle_kill_switch", { active: false });
      loadPolicy();
    } catch (err) {
      setKillSwitchError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleConfirmKillSwitch = async () => {
    setKillSwitchError(null);
    try {
      await invoke("toggle_kill_switch", {
        active: true,
        reason: "Manual activation",
      });
      setShowKillConfirm(false);
      loadPolicy();
    } catch (err) {
      setKillSwitchError(err instanceof Error ? err.message : String(err));
    }
  };

  if (isLoading) return null;
  if (loadError) {
    return (
      <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
        <div className="rounded-lg bg-[#FEF2F2] px-4 py-3 text-sm text-[#EF4444]">
          Failed to load global policy: {loadError}
        </div>
        <button
          type="button"
          onClick={loadPolicy}
          className="mt-3 rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA]"
        >
          Retry
        </button>
      </div>
    );
  }
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
            min="0"
            step="any"
            value={policy.daily_cap}
            onChange={(e) => handleCapChange("daily_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
          {validationErrors.daily_cap && <p className="mt-1 text-xs text-[#EF4444]">{validationErrors.daily_cap}</p>}
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="weekly-cap">
            Weekly Cap (USDC)
          </label>
          <input
            id="weekly-cap"
            type="number"
            min="0"
            step="any"
            value={policy.weekly_cap}
            onChange={(e) => handleCapChange("weekly_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
          {validationErrors.weekly_cap && <p className="mt-1 text-xs text-[#EF4444]">{validationErrors.weekly_cap}</p>}
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="monthly-cap">
            Monthly Cap (USDC)
          </label>
          <input
            id="monthly-cap"
            type="number"
            min="0"
            step="any"
            value={policy.monthly_cap}
            onChange={(e) => handleCapChange("monthly_cap", e.target.value)}
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
          {validationErrors.monthly_cap && <p className="mt-1 text-xs text-[#EF4444]">{validationErrors.monthly_cap}</p>}
        </div>

        <div>
          <label className="text-sm font-medium text-[#374151]" htmlFor="min-reserve">
            Minimum Reserve Balance (USDC)
          </label>
          <input
            id="min-reserve"
            type="number"
            min="0"
            step="any"
            value={policy.min_reserve_balance}
            onChange={(e) =>
              handleCapChange("min_reserve_balance", e.target.value)
            }
            className="mt-1 block w-40 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
          {validationErrors.min_reserve_balance && <p className="mt-1 text-xs text-[#EF4444]">{validationErrors.min_reserve_balance}</p>}
        </div>
      </div>

      <div className="mt-6">
        {saveError && (
          <div className="mb-3 rounded-lg bg-[#FEF2F2] px-4 py-2 text-sm text-[#EF4444]">
            Failed to save: {saveError}
          </div>
        )}
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
        {killSwitchError && (
          <div className="mb-3 rounded-lg bg-[#FEF2F2] px-4 py-2 text-sm text-[#EF4444]">
            Kill switch error: {killSwitchError}
          </div>
        )}
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
