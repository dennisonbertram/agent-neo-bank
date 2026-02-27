interface FundStepProps {
  onNext: () => void;
}

export function FundStep({ onNext }: FundStepProps) {
  return (
    <div className="text-center">
      <h2 className="text-xl font-bold">Fund Your Wallet</h2>
      <p className="mt-2 text-muted-foreground">Fund step placeholder</p>
      <button onClick={onNext} className="mt-4">
        Done
      </button>
    </div>
  );
}
