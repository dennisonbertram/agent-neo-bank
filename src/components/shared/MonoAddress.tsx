import { useState, useCallback } from "react";
import { Copy, Check } from "lucide-react";
import { cn } from "@/lib/utils";

interface MonoAddressProps {
  address: string;
  className?: string;
  full?: boolean;
}

export function MonoAddress({ address, className, full = false }: MonoAddressProps) {
  const [copied, setCopied] = useState(false);
  const displayAddress = full ? address : `${address.slice(0, 6)}...${address.slice(-4)}`;

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [address]);

  return (
    <span className={cn("inline-flex items-center gap-1.5", className)}>
      <code className="font-mono text-sm" style={{ fontFeatureSettings: '"tnum"' }}>{displayAddress}</code>
      <button
        onClick={handleCopy}
        className="inline-flex size-5 items-center justify-center rounded text-[#9CA3AF] transition-colors hover:text-[#4F46E5]"
        title={copied ? "Copied!" : "Copy address"}
      >
        {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
      </button>
    </span>
  );
}
