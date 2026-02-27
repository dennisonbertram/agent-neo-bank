interface OtpStepProps {
  onNext: () => void;
}

export function OtpStep({ onNext }: OtpStepProps) {
  return (
    <div className="text-center">
      <h2 className="text-xl font-bold">Verify OTP</h2>
      <p className="mt-2 text-muted-foreground">OTP step placeholder</p>
      <button onClick={onNext} className="mt-4">
        Verify
      </button>
    </div>
  );
}
