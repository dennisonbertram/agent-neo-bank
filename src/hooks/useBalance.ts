import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface UseBalanceReturn {
  balance: string | null;
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
}

export function useBalance(): UseBalanceReturn {
  const [balance, setBalance] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<{ balance: string; asset: string }>("get_balance");
      setBalance(result.balance);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  return { balance, isLoading, error, refetch: fetchBalance };
}
