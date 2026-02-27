import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { NotificationPreferences } from "../../types";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";

export function Notifications() {
  const [prefs, setPrefs] = useState<NotificationPreferences | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    invoke<NotificationPreferences>("get_notification_preferences")
      .then(setPrefs)
      .catch(() => {})
      .finally(() => setIsLoading(false));
  }, []);

  const handleToggle = (key: keyof NotificationPreferences) => {
    if (!prefs) return;
    setPrefs({ ...prefs, [key]: !prefs[key] });
  };

  const handleSave = async () => {
    if (!prefs) return;
    setIsSaving(true);
    try {
      await invoke("update_notification_preferences", { prefs });
    } catch {
      // handle error
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) return null;
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
    <Card>
      <CardHeader>
        <CardTitle>Notification Preferences</CardTitle>
        <CardDescription>
          Choose which events trigger OS notifications
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {toggleItems.map(({ key, label, description }) => (
          <div key={key} className="flex items-center justify-between py-2">
            <div>
              <p className="text-sm font-medium">{label}</p>
              <p className="text-xs text-muted-foreground">{description}</p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={Boolean(prefs[key])}
              onClick={() => handleToggle(key)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                prefs[key] ? "bg-primary" : "bg-muted"
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

        {prefs.on_large_tx && (
          <div className="pt-2">
            <label className="text-sm font-medium" htmlFor="threshold">
              Large Transaction Threshold (USDC)
            </label>
            <Input
              id="threshold"
              type="number"
              value={prefs.large_tx_threshold}
              onChange={(e) =>
                setPrefs({ ...prefs, large_tx_threshold: e.target.value })
              }
              className="mt-1 w-32"
            />
          </div>
        )}

        <div className="pt-4">
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving ? "Saving..." : "Save Preferences"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
