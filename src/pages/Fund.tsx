import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Wallet, CreditCard, AlertCircle } from "lucide-react";

export function Fund() {
  const [activeTab, setActiveTab] = useState<"buy" | "deposit">("buy");
  const [copied, setCopied] = useState(false);
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [addressError, setAddressError] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  useEffect(() => {
    return () => { clearTimeout(timerRef.current); };
  }, []);

  useEffect(() => {
    invoke<string>("get_wallet_address")
      .then(setWalletAddress)
      .catch(() => {
        invoke<{ address?: string }>("auth_status")
          .then((res) => {
            if (res.address) setWalletAddress(res.address);
            else setAddressError(true);
          })
          .catch(() => setAddressError(true));
      });
  }, []);

  const addressReady = walletAddress && walletAddress.length > 4;

  const handleCopy = async () => {
    if (!addressReady) return;
    try {
      await navigator.clipboard.writeText(walletAddress);
      setCopied(true);
      clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard may fail
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold text-[#1A1A1A]">Fund Wallet</h1>
        <p className="mt-1 text-sm text-[#6B7280]">Add funds to your wallet</p>
      </div>

      {/* Tab bar */}
      <div className="flex gap-1 border-b border-[#F0EDE8]">
        <button
          onClick={() => setActiveTab("buy")}
          className={`px-4 py-2.5 text-sm font-medium transition-colors ${
            activeTab === "buy"
              ? "border-b-2 border-[#4F46E5] text-[#4F46E5]"
              : "text-[#6B7280] hover:text-[#1A1A1A]"
          }`}
        >
          Buy Crypto
        </button>
        <button
          onClick={() => setActiveTab("deposit")}
          className={`px-4 py-2.5 text-sm font-medium transition-colors ${
            activeTab === "deposit"
              ? "border-b-2 border-[#4F46E5] text-[#4F46E5]"
              : "text-[#6B7280] hover:text-[#1A1A1A]"
          }`}
        >
          Deposit
        </button>
      </div>

      {activeTab === "buy" ? (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-8">
          <div className="flex flex-col items-center text-center">
            <div className="flex size-16 items-center justify-center rounded-2xl border-2 border-dashed border-[#E8E5E0] bg-[#F9FAFB]">
              <CreditCard className="size-8 text-[#9CA3AF]" />
            </div>
            <h2 className="mt-4 text-lg font-semibold text-[#1A1A1A]">Buy crypto with Coinbase</h2>
            <p className="mt-2 max-w-sm text-sm text-[#6B7280]">
              Purchase USDC, ETH, or WETH using your credit card, debit card, or bank transfer.
            </p>
            <button disabled className="mt-6 inline-flex items-center gap-2 rounded-lg bg-[#4F46E5] px-6 py-3 text-sm font-medium text-white opacity-50 cursor-not-allowed">
              Continue to Coinbase
            </button>
            <p className="mt-3 text-xs text-[#9CA3AF]">Coming soon</p>
          </div>

          <div className="mt-8 flex flex-wrap items-center justify-center gap-4 border-t border-[#F0EDE8] pt-6">
            <div className="flex items-center gap-3">
              <span className="text-xs font-medium text-[#6B7280]">Supported tokens:</span>
              {["USDC", "ETH", "WETH"].map(token => (
                <span key={token} className="rounded-full bg-[#F9FAFB] px-2.5 py-1 text-xs font-medium text-[#1A1A1A]">{token}</span>
              ))}
            </div>
            <span className="inline-flex items-center gap-1.5 rounded-full bg-[#ECFDF5] px-2.5 py-1 text-xs font-medium text-[#10B981]">
              <span className="size-1.5 rounded-full bg-[#10B981]" />
              Base Sepolia
            </span>
          </div>
        </div>
      ) : (
        <div className="rounded-xl border border-[#F0EDE8] bg-white p-8">
          <div className="flex flex-col items-center text-center">
            <h2 className="text-lg font-semibold text-[#1A1A1A]">Your Wallet Address</h2>

            {addressError ? (
              <div className="mt-4 flex items-center gap-2 rounded-lg border border-[#FCA5A5] bg-[#FEF2F2] px-4 py-3 text-sm text-[#EF4444]">
                <AlertCircle className="size-4" />
                Couldn't load wallet address. Please check your connection and try again.
              </div>
            ) : !addressReady ? (
              <div className="mt-4 flex w-full max-w-md items-center justify-center rounded-lg border border-[#E8E5E0] bg-[#F9FAFB] px-4 py-3 text-sm text-[#9CA3AF]">
                Loading wallet address...
              </div>
            ) : (
              <div className="mt-4 flex w-full max-w-md items-center gap-2 rounded-lg border border-[#E8E5E0] bg-[#F9FAFB] px-4 py-3">
                <code className="flex-1 truncate text-sm font-mono text-[#1A1A1A]">{walletAddress}</code>
                <button
                  onClick={handleCopy}
                  className="rounded-md bg-[#EEF2FF] px-3 py-1 text-xs font-medium text-[#4F46E5] hover:bg-[#4F46E5] hover:text-white"
                >
                  {copied ? "Copied!" : "Copy"}
                </button>
              </div>
            )}

            <p className="mt-4 max-w-sm text-sm text-[#6B7280]">
              Copy the address above to send funds to your wallet
            </p>

            <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
              {["USDC", "ETH", "WETH"].map(token => (
                <span key={token} className="rounded-full bg-[#F9FAFB] px-2.5 py-1 text-xs font-medium text-[#1A1A1A]">{token}</span>
              ))}
            </div>

            <span className="mt-3 inline-flex items-center gap-1.5 rounded-full bg-[#ECFDF5] px-2.5 py-1 text-xs font-medium text-[#10B981]">
              <span className="size-1.5 rounded-full bg-[#10B981]" />
              Base Sepolia
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
