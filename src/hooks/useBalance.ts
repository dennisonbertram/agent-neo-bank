import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { BalanceResponse } from "../types";

interface UseBalanceReturn {
  balance: string | null;
  balances: BalanceResponse["balances"];
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
}

export function useBalance(): UseBalanceReturn {
  const [balance, setBalance] = useState<string | null>(null);
  const [balances, setBalances] = useState<BalanceResponse["balances"]>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<BalanceResponse>("get_balance");
      setBalance(result.balance);
      setBalances(result.balances);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  return { balance, balances, isLoading, error, refetch: fetchBalance };
}
