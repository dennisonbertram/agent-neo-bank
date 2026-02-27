import { useState } from "react";
import { ArrowLeft } from "lucide-react";

interface EmailStepProps {
  onNext: (email: string) => void;
  onBack: () => void;
}

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

export function EmailStep({ onNext, onBack }: EmailStepProps) {
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!isValidEmail(email)) {
      setError("Please enter a valid email address");
      return;
    }
    setError(null);
    onNext(email);
  }

  return (
    <div>
      {/* Back button */}
      <button
        onClick={onBack}
        className="mb-6 flex items-center gap-1 text-sm text-[#6B7280] hover:text-[#1A1A1A]"
      >
        <ArrowLeft className="size-4" />
        Back
      </button>

      <h1 className="text-center text-2xl font-semibold text-[#1A1A1A]">
        Connect your wallet
      </h1>
      <p className="mt-2 text-center text-sm text-[#6B7280]">
        Enter your email to set up your Agent Wallet. We'll send you a
        verification code.
      </p>

      <form onSubmit={handleSubmit} className="mt-8 space-y-4">
        <div>
          <label className="mb-1.5 block text-sm font-medium text-[#1A1A1A]">
            Email address
          </label>
          <input
            type="text"
            inputMode="email"
            autoComplete="email"
            value={email}
            onChange={(e) => {
              setEmail(e.target.value);
              if (error) setError(null);
            }}
            className="w-full rounded-lg border border-[#E8E5E0] bg-white px-4 py-3 text-base text-[#1A1A1A] placeholder:text-[#9CA3AF] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
            placeholder="you@example.com"
          />
          {error && <p className="mt-2 text-sm text-[#EF4444]">{error}</p>}
        </div>
        <button
          type="submit"
          className="w-full rounded-lg bg-[#4F46E5] px-6 py-3 text-base font-medium text-white transition-colors hover:bg-[#4338CA] disabled:opacity-50"
        >
          Send Verification Code
        </button>
      </form>

      <p className="mt-4 text-center text-xs text-[#9CA3AF]">
        By continuing, you agree to the Coinbase Agent Wallet terms.
      </p>
    </div>
  );
}
