import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WelcomeStep } from "@/components/onboarding/WelcomeStep";
import { EmailStep } from "@/components/onboarding/EmailStep";
import { OtpStep } from "@/components/onboarding/OtpStep";
import { FundStep } from "@/components/onboarding/FundStep";

interface AuthLoginResponse {
  status: "otp_sent" | "verified";
  flow_id?: string;
}

interface AuthVerifyResponse {
  status: "verified" | "otp_sent";
}

export function Onboarding() {
  const [step, setStep] = useState(0);
  const [email, setEmail] = useState("");
  const [walletAddress, setWalletAddress] = useState("0x...");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleEmailSubmit(submittedEmail: string) {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AuthLoginResponse>("auth_login", {
        email: submittedEmail,
      });
      setEmail(submittedEmail);
      if (result.status === "otp_sent") {
        setStep(2);
      } else if (result.status === "verified") {
        // Already verified (e.g. returning user), skip OTP but fetch wallet address
        try {
          const status = await invoke<{ address?: string }>("auth_status");
          if (status.address) {
            setWalletAddress(status.address);
          }
        } catch {
          // auth_status may not return address yet; use placeholder
        }
        setStep(3);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleOtpSubmit(otp: string) {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AuthVerifyResponse>("auth_verify", { otp });
      if (result.status === "verified") {
        // Fetch wallet address after successful auth
        try {
          const status = await invoke<{ address?: string }>("auth_status");
          if (status.address) {
            setWalletAddress(status.address);
          }
        } catch {
          // auth_status may not return address yet; use placeholder
        }
        setStep(3);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  function handleComplete() {
    // Navigate to dashboard
    window.location.href = "/";
  }

  function handleBackToEmail() {
    setError(null);
    setStep(1);
  }

  function handleBackToWelcome() {
    setError(null);
    setStep(0);
  }

  return (
    <div
      className="flex min-h-screen items-center justify-center"
      style={{
        background:
          "radial-gradient(ellipse at 50% 30%, #EEF2FF 0%, #FAFAF9 70%)",
      }}
    >
      <div className="w-full max-w-[440px] px-4">
        {step === 0 && <WelcomeStep onNext={() => setStep(1)} />}
        {step === 1 && (
          <EmailStep
            onNext={handleEmailSubmit}
            onBack={handleBackToWelcome}
            loading={loading}
            serverError={error}
          />
        )}
        {step === 2 && (
          <OtpStep
            onNext={handleOtpSubmit}
            onBack={handleBackToEmail}
            loading={loading}
            serverError={error}
            email={email}
          />
        )}
        {step === 3 && (
          <FundStep address={walletAddress} onNext={handleComplete} />
        )}
      </div>
    </div>
  );
}
