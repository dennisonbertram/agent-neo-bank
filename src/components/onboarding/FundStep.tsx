import { useState } from "react";
import { Copy, Check, Wallet } from "lucide-react";

interface FundStepProps {
  address: string;
  onNext: () => void;
}

export function FundStep({ address, onNext }: FundStepProps) {
  const [copied, setCopied] = useState(false);
  const addressReady = address !== "0x..." && address !== "";

  async function handleCopy() {
    if (!addressReady) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div>
      <div className="text-center">
        <div className="mx-auto flex size-12 items-center justify-center rounded-full bg-[#EEF2FF]">
          <Wallet className="size-6 text-[#4F46E5]" />
        </div>
        <h1 className="mt-4 text-2xl font-semibold text-[#1A1A1A]">
          Fund your wallet
        </h1>
        <p className="mt-2 text-sm text-[#6B7280]">
          Send USDC to your wallet address to get started
        </p>
      </div>

      <div className="mt-8 rounded-xl border border-[#E8E5E0] bg-white p-4">
        <p className="mb-2 text-xs font-medium text-[#6B7280]">
          Your wallet address
        </p>
        <div className="flex items-center gap-2">
          <code className="flex-1 break-all text-sm font-mono text-[#1A1A1A]">
            {address}
          </code>
          <button
            onClick={handleCopy}
            disabled={!addressReady}
            aria-label="Copy address"
            className={`flex size-9 items-center justify-center rounded-lg border border-[#E8E5E0] text-[#6B7280] transition-colors ${addressReady ? "hover:bg-[#F5F5F4] hover:text-[#1A1A1A]" : "opacity-50 cursor-not-allowed"}`}
          >
            {copied ? (
              <Check className="size-4" />
            ) : (
              <Copy className="size-4" />
            )}
          </button>
        </div>
      </div>

      <button
        onClick={onNext}
        disabled={!addressReady}
        className={`mt-6 w-full rounded-lg px-6 py-3 text-base font-medium text-white transition-colors active:scale-[0.98] ${addressReady ? "bg-[#4F46E5] hover:bg-[#4338CA]" : "bg-[#4F46E5]/50 cursor-not-allowed"}`}
      >
        {addressReady ? "Continue to Dashboard" : "Loading wallet address..."}
      </button>
    </div>
  );
}
