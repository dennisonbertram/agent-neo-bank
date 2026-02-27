import { useState, useEffect } from "react";

interface BalanceState {
  balance: string | null;
  asset: string;
  loading: boolean;
  error: string | null;
}

export function useBalance(): BalanceState {
  const [state, setState] = useState<BalanceState>({
    balance: null,
    asset: "USDC",
    loading: true,
    error: null,
  });

  useEffect(() => {
    setState((prev) => ({ ...prev, loading: false }));
  }, []);

  return state;
}
