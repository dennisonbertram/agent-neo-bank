import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { GlobalPolicy } from "../../types";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";

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
    <Card>
      <CardHeader>
        <CardTitle>Global Policy</CardTitle>
        <CardDescription>
          Wallet-level spending controls and kill switch
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Spending caps */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">Spending Caps</h3>

          <div>
            <label className="text-sm font-medium" htmlFor="daily-cap">
              Daily Cap (USDC)
            </label>
            <Input
              id="daily-cap"
              type="number"
              value={policy.daily_cap}
              onChange={(e) => handleCapChange("daily_cap", e.target.value)}
              className="mt-1 w-40"
            />
          </div>

          <div>
            <label className="text-sm font-medium" htmlFor="weekly-cap">
              Weekly Cap (USDC)
            </label>
            <Input
              id="weekly-cap"
              type="number"
              value={policy.weekly_cap}
              onChange={(e) => handleCapChange("weekly_cap", e.target.value)}
              className="mt-1 w-40"
            />
          </div>

          <div>
            <label className="text-sm font-medium" htmlFor="monthly-cap">
              Monthly Cap (USDC)
            </label>
            <Input
              id="monthly-cap"
              type="number"
              value={policy.monthly_cap}
              onChange={(e) => handleCapChange("monthly_cap", e.target.value)}
              className="mt-1 w-40"
            />
          </div>

          <div>
            <label className="text-sm font-medium" htmlFor="min-reserve">
              Minimum Reserve Balance (USDC)
            </label>
            <Input
              id="min-reserve"
              type="number"
              value={policy.min_reserve_balance}
              onChange={(e) =>
                handleCapChange("min_reserve_balance", e.target.value)
              }
              className="mt-1 w-40"
            />
          </div>
        </div>

        <Button onClick={handleSaveCaps} disabled={isSaving}>
          {isSaving ? "Saving..." : "Save Caps"}
        </Button>

        {/* Kill Switch */}
        <div className="border-t pt-6">
          <h3 className="text-sm font-medium text-red-600">Kill Switch</h3>
          <p className="text-xs text-muted-foreground mb-4">
            Emergency stop: blocks ALL agent transactions immediately
          </p>
          <Button
            variant={policy.kill_switch_active ? "outline" : "destructive"}
            onClick={handleToggleKillSwitch}
          >
            {policy.kill_switch_active
              ? "Deactivate Kill Switch"
              : "Activate Kill Switch"}
          </Button>

          {/* Confirmation dialog for activation */}
          {showKillConfirm && (
            <div className="mt-4 p-4 border border-red-300 rounded-md bg-red-50">
              <p className="text-sm text-red-800 mb-3">
                Are you sure? This will immediately block ALL agent
                transactions.
              </p>
              <div className="flex gap-2">
                <Button
                  variant="destructive"
                  onClick={handleConfirmKillSwitch}
                >
                  Confirm Activation
                </Button>
                <Button
                  variant="outline"
                  onClick={() => setShowKillConfirm(false)}
                >
                  Cancel
                </Button>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
