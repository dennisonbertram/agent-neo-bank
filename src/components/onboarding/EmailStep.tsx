interface EmailStepProps {
  onNext: () => void;
}

export function EmailStep({ onNext }: EmailStepProps) {
  return (
    <div className="text-center">
      <h2 className="text-xl font-bold">Enter Email</h2>
      <p className="mt-2 text-muted-foreground">Email step placeholder</p>
      <button onClick={onNext} className="mt-4">
        Continue
      </button>
    </div>
  );
}
