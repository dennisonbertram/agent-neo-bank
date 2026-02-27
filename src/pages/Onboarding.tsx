import { useState } from "react";
import { WelcomeStep } from "@/components/onboarding/WelcomeStep";
import { EmailStep } from "@/components/onboarding/EmailStep";
import { OtpStep } from "@/components/onboarding/OtpStep";
import { FundStep } from "@/components/onboarding/FundStep";

export function Onboarding() {
  const [step, setStep] = useState(0);
  const [walletAddress, setWalletAddress] = useState("0x...");

  function handleEmailSubmit(_email: string) {
    // In real implementation, this would call invoke("auth_login", { email })
    setStep(2);
  }

  function handleOtpSubmit(_otp: string) {
    // In real implementation, this would call invoke("auth_verify", { email, otp })
    // and retrieve the wallet address
    setWalletAddress("0x1234567890abcdef1234567890abcdef12345678");
    setStep(3);
  }

  function handleComplete() {
    // Navigate to dashboard
    window.location.href = "/";
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-6">
      <div className="w-full max-w-md">
        {step === 0 && <WelcomeStep onNext={() => setStep(1)} />}
        {step === 1 && (
          <EmailStep onNext={handleEmailSubmit} onBack={() => setStep(0)} />
        )}
        {step === 2 && (
          <OtpStep onNext={handleOtpSubmit} onBack={() => setStep(1)} />
        )}
        {step === 3 && (
          <FundStep address={walletAddress} onNext={handleComplete} />
        )}
      </div>
    </div>
  );
}
