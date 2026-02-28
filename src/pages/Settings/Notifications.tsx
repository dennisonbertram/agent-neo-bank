import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { NotificationPreferences } from "../../types";

export function Notifications() {
  const [prefs, setPrefs] = useState<NotificationPreferences | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [thresholdError, setThresholdError] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const loadPrefs = () => {
    setLoadError(null);
    setIsLoading(true);
    invoke<NotificationPreferences>("get_notification_preferences")
      .then(setPrefs)
      .catch((err) => {
        setLoadError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => setIsLoading(false));
  };

  useEffect(() => {
    loadPrefs();
  }, []);

  const handleToggle = (key: keyof NotificationPreferences) => {
    if (!prefs) return;
    setPrefs({ ...prefs, [key]: !prefs[key] });
  };

  const handleSave = async () => {
    if (!prefs) return;
    setSaveError(null);

    // Validate threshold if large tx notifications are enabled
    if (prefs.on_large_tx) {
      const num = parseFloat(prefs.large_tx_threshold as string);
      if (isNaN(num) || num <= 0) {
        setThresholdError("Must be a positive number");
        return;
      }
      setThresholdError(null);
    }

    setIsSaving(true);
    try {
      const normalized = prefs.on_large_tx
        ? { ...prefs, large_tx_threshold: parseFloat(prefs.large_tx_threshold as string).toString() }
        : prefs;
      await invoke("update_notification_preferences", { prefs: normalized });
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) return null;
  if (loadError) {
    return (
      <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
        <div className="rounded-lg bg-[#FEF2F2] px-4 py-3 text-sm text-[#EF4444]">
          Failed to load notification preferences: {loadError}
        </div>
        <button
          type="button"
          onClick={loadPrefs}
          className="mt-3 rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA]"
        >
          Retry
        </button>
      </div>
    );
  }
  if (!prefs) return null;

  const toggleItems = [
    {
      key: "enabled" as const,
      label: "Enable Notifications",
      description: "Master toggle for all notifications",
    },
    {
      key: "on_all_tx" as const,
      label: "All Transactions",
      description: "Notify on every transaction",
    },
    {
      key: "on_large_tx" as const,
      label: "Large Transactions Only",
      description: "Notify only above threshold",
    },
    {
      key: "on_errors" as const,
      label: "Errors",
      description: "Notify on transaction errors and failures",
    },
    {
      key: "on_limit_requests" as const,
      label: "Limit Requests",
      description: "Notify when agents request limit increases",
    },
    {
      key: "on_agent_registration" as const,
      label: "Agent Registration",
      description: "Notify when new agents register",
    },
  ];

  return (
    <div className="rounded-xl border border-[#F0EDE8] bg-white p-6">
      <h2 className="text-base font-semibold text-[#1A1A1A] mb-1">Notification Preferences</h2>
      <p className="text-sm text-[#6B7280] mb-6">
        Choose which events trigger OS notifications
      </p>

      <div className="space-y-1">
        {toggleItems.map(({ key, label, description }) => (
          <div key={key} className="flex items-center justify-between py-3">
            <div>
              <p className="text-sm font-medium text-[#1A1A1A]">{label}</p>
              <p className="text-xs text-[#9CA3AF]">{description}</p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={Boolean(prefs[key])}
              onClick={() => handleToggle(key)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                prefs[key] ? "bg-[#4F46E5]" : "bg-[#D1D5DB]"
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  prefs[key] ? "translate-x-6" : "translate-x-1"
                }`}
              />
            </button>
          </div>
        ))}
      </div>

      {prefs.on_large_tx && (
        <div className="mt-4">
          <label className="text-sm font-medium text-[#374151]" htmlFor="threshold">
            Large Transaction Threshold (USDC)
          </label>
          <input
            id="threshold"
            type="number"
            min="0"
            step="any"
            value={prefs.large_tx_threshold}
            onChange={(e) => {
              setPrefs({ ...prefs, large_tx_threshold: e.target.value });
              if (thresholdError) setThresholdError(null);
            }}
            className="mt-1 block w-32 rounded-lg border border-[#E5E7EB] bg-white px-3 py-2 text-sm text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-1 focus:ring-[#4F46E5]"
          />
          {thresholdError && <p className="mt-1 text-xs text-[#EF4444]">{thresholdError}</p>}
        </div>
      )}

      <div className="mt-6">
        {saveError && (
          <div className="mb-3 rounded-lg bg-[#FEF2F2] px-4 py-2 text-sm text-[#EF4444]">
            Failed to save: {saveError}
          </div>
        )}
        <button
          type="button"
          onClick={handleSave}
          disabled={isSaving}
          className="rounded-lg bg-[#4F46E5] px-4 py-2 text-sm font-medium text-white hover:bg-[#4338CA] disabled:opacity-50"
        >
          {isSaving ? "Saving..." : "Save Preferences"}
        </button>
      </div>
    </div>
  );
}
