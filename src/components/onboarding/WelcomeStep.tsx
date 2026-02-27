import { Wallet } from "lucide-react";

interface WelcomeStepProps {
  onNext: () => void;
}

export function WelcomeStep({ onNext }: WelcomeStepProps) {
  return (
    <div className="text-center">
      {/* Logo */}
      <div className="mx-auto flex size-12 items-center justify-center rounded-2xl bg-[#4F46E5]">
        <Wallet className="size-6 text-white" />
      </div>

      <h1 className="mt-6 text-2xl font-semibold text-[#1A1A1A]">
        Give your AI agents spending power
      </h1>

      <p className="mt-3 text-base text-[#6B7280]">
        Set up a wallet, define budgets, and let your AI agents pay for services
        autonomously — with guardrails you control.
      </p>

      <p className="mt-2 text-xs text-[#9CA3AF]">Set up in 2 minutes</p>

      <button
        onClick={onNext}
        className="mt-8 w-full rounded-lg bg-[#4F46E5] px-6 py-3 text-base font-medium text-white transition-colors hover:bg-[#4338CA] active:scale-[0.98]"
      >
        Get Started
      </button>

      <p className="mt-6 text-xs text-[#9CA3AF]">
        Powered by Coinbase Agent Wallet
      </p>
    </div>
  );
}
