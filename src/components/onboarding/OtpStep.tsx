import { useState, useRef } from "react";
import { ArrowLeft, Mail } from "lucide-react";

interface OtpStepProps {
  onNext: (otp: string) => void | Promise<void>;
  onBack: () => void;
  loading?: boolean;
  serverError?: string | null;
  email?: string;
}

export function OtpStep({ onNext, onBack, loading, serverError, email }: OtpStepProps) {
  const [digits, setDigits] = useState(["", "", "", "", "", ""]);
  const [error, setError] = useState<string | null>(null);
  const refs = [
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
  ];

  function handleDigitChange(index: number, value: string) {
    const sanitized = value.replace(/\D/g, "");
    if (!sanitized && !value) {
      // Allow clearing
      const next = [...digits];
      next[index] = "";
      setDigits(next);
      return;
    }
    if (!sanitized) return;

    const next = [...digits];
    next[index] = sanitized.slice(-1);
    setDigits(next);
    if (error) setError(null);

    // Auto-advance to next input
    if (sanitized && index < 5) {
      refs[index + 1]?.current?.focus();
    }
  }

  function handleKeyDown(index: number, e: React.KeyboardEvent) {
    if (e.key === "Backspace" && !digits[index] && index > 0) {
      refs[index - 1]?.current?.focus();
    }
  }

  function handleVerify() {
    const otp = digits.join("");
    if (!/^\d{6}$/.test(otp)) {
      setError("Please enter exactly 6 digits");
      return;
    }
    setError(null);
    onNext(otp);
  }

  return (
    <div>
      <button
        onClick={onBack}
        className="mb-6 flex items-center gap-1 text-sm text-[#6B7280] hover:text-[#1A1A1A]"
      >
        <ArrowLeft className="size-4" />
        Back
      </button>

      <div className="text-center">
        <div className="mx-auto flex size-12 items-center justify-center rounded-full bg-[#EEF2FF]">
          <Mail className="size-6 text-[#4F46E5]" />
        </div>
        <h1 className="mt-4 text-2xl font-semibold text-[#1A1A1A]">
          Check your email
        </h1>
        <p className="mt-2 text-sm text-[#6B7280]">
          Enter the 6-digit code we sent{email ? ` to ${email}` : " you"}
        </p>
      </div>

      {/* 6 digit inputs in a row */}
      <div className="mt-8 flex justify-center gap-2">
        {digits.map((digit, i) => (
          <input
            key={i}
            ref={refs[i]}
            type="text"
            inputMode="numeric"
            maxLength={1}
            value={digit}
            onChange={(e) => handleDigitChange(i, e.target.value)}
            onKeyDown={(e) => handleKeyDown(i, e)}
            aria-label={`Digit ${i + 1}`}
            className="size-12 rounded-xl border-2 border-[#E8E5E0] bg-white text-center text-xl font-bold text-[#1A1A1A] focus:border-[#4F46E5] focus:outline-none focus:ring-2 focus:ring-[#4F46E5]/20"
          />
        ))}
      </div>

      {error && (
        <p className="mt-3 text-center text-sm text-[#EF4444]">{error}</p>
      )}
      {serverError && (
        <p className="mt-3 text-center text-sm text-[#EF4444]">{serverError}</p>
      )}

      <button
        onClick={handleVerify}
        disabled={loading}
        className="mt-6 w-full rounded-lg bg-[#4F46E5] px-6 py-3 text-base font-medium text-white transition-colors hover:bg-[#4338CA] disabled:opacity-50"
      >
        {loading ? "Verifying..." : "Verify"}
      </button>

      <p className="mt-4 text-center text-sm text-[#6B7280]">
        Didn't receive it?{" "}
        <button className="text-[#4F46E5] hover:text-[#4338CA]">
          Resend code
        </button>
      </p>
    </div>
  );
}
