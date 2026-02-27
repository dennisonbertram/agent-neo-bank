import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Copy, Check } from "lucide-react";

interface FundStepProps {
  address: string;
  onNext: () => void;
}

export function FundStep({ address, onNext }: FundStepProps) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Fund your wallet</h2>
        <p className="text-muted-foreground mt-1">
          Send USDC to your wallet address to get started
        </p>
      </div>
      <div className="rounded-lg border border-border bg-zinc-800/50 p-4">
        <p className="text-xs text-muted-foreground mb-2">Your wallet address</p>
        <div className="flex items-center gap-2">
          <code className="flex-1 break-all text-sm font-mono">{address}</code>
          <Button
            variant="outline"
            size="icon"
            onClick={handleCopy}
            aria-label="Copy address"
          >
            {copied ? <Check className="size-4" /> : <Copy className="size-4" />}
          </Button>
        </div>
      </div>
      <Button size="lg" className="w-full" onClick={onNext}>
        Continue to Dashboard
      </Button>
    </div>
  );
}
