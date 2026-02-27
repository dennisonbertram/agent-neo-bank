import { Button } from "@/components/ui/button";

interface WelcomeStepProps {
  onNext: () => void;
}

export function WelcomeStep({ onNext }: WelcomeStepProps) {
  return (
    <div className="text-center space-y-6">
      <h2 className="text-2xl font-bold">Welcome to Agent Neo Bank</h2>
      <p className="text-muted-foreground">
        Your agent-powered crypto wallet. Manage spending policies, approve
        transactions, and let your AI agents handle payments securely.
      </p>
      <Button size="lg" onClick={onNext}>
        Get Started
      </Button>
    </div>
  );
}
