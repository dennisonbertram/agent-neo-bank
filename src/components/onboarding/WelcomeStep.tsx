interface WelcomeStepProps {
  onNext: () => void;
}

export function WelcomeStep({ onNext }: WelcomeStepProps) {
  return (
    <div className="text-center">
      <h2 className="text-xl font-bold">Welcome</h2>
      <p className="mt-2 text-muted-foreground">Welcome step placeholder</p>
      <button onClick={onNext} className="mt-4">
        Get Started
      </button>
    </div>
  );
}
