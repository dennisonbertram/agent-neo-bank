import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { InvitationCode } from "../../types";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Input } from "@/components/ui/input";

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
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Invitation Codes</CardTitle>
          <Button onClick={() => setIsDialogOpen(true)}>Generate Code</Button>
        </div>
        <CardDescription>
          Manage invitation codes for agent registration
        </CardDescription>
      </CardHeader>
      <CardContent>
        {!isLoading && codes.length === 0 ? (
          <p className="text-muted-foreground text-center py-8">
            No invitation codes
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Code</TableHead>
                <TableHead>Label</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Uses</TableHead>
                <TableHead>Created</TableHead>
                <TableHead />
              </TableRow>
            </TableHeader>
            <TableBody>
              {codes.map((code) => {
                const status = getCodeStatus(code);
                const config = statusConfig[status];
                return (
                  <TableRow key={code.code}>
                    <TableCell className="font-mono">{code.code}</TableCell>
                    <TableCell>{code.label}</TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        data-testid={`status-badge-${code.code}`}
                        className={config.className}
                      >
                        {config.label}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {code.use_count} / {code.max_uses}
                    </TableCell>
                    <TableCell>{formatDate(code.created_at)}</TableCell>
                    <TableCell>
                      {status === "active" && (
                        <Button
                          variant="destructive"
                          size="sm"
                          onClick={() => handleRevoke(code.code)}
                        >
                          Revoke
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
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
                <label htmlFor="code-label" className="text-sm font-medium">
                  Label
                </label>
                <Input
                  id="code-label"
                  placeholder="Label for this code"
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => setIsDialogOpen(false)}
              >
                Cancel
              </Button>
              <Button onClick={handleGenerate} disabled={!label.trim()}>
                Generate
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </CardContent>
    </Card>
  );
}
