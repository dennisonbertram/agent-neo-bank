import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

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
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Enter your email</h2>
        <p className="text-muted-foreground mt-1">
          We will send you a verification code
        </p>
      </div>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <Input
            type="text"
            placeholder="Email address"
            value={email}
            onChange={(e) => {
              setEmail(e.target.value);
              if (error) setError(null);
            }}
          />
          {error && <p className="mt-2 text-sm text-red-500">{error}</p>}
        </div>
        <div className="flex gap-3">
          <Button type="button" variant="outline" onClick={onBack}>
            Back
          </Button>
          <Button type="submit">Continue</Button>
        </div>
      </form>
    </div>
  );
}
