import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface OtpStepProps {
  onNext: (otp: string) => void;
  onBack: () => void;
}

export function OtpStep({ onNext, onBack }: OtpStepProps) {
  const [otp, setOtp] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!/^\d{6}$/.test(otp)) {
      setError("Please enter exactly 6 digits");
      return;
    }
    setError(null);
    onNext(otp);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Enter verification code</h2>
        <p className="text-muted-foreground mt-1">
          Check your email for the 6-digit code
        </p>
      </div>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Input
            type="text"
            placeholder="000000"
            maxLength={6}
            value={otp}
            onChange={(e) => {
              const val = e.target.value.replace(/\D/g, "");
              setOtp(val);
              if (error) setError(null);
            }}
          />
          {error && <p className="mt-2 text-sm text-red-500">{error}</p>}
        </div>
        <div className="flex gap-3">
          <Button type="button" variant="outline" onClick={onBack}>
            Back
          </Button>
          <Button type="submit">Verify</Button>
        </div>
      </form>
    </div>
  );
}
