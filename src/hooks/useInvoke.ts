import { useState, useCallback } from "react";

interface InvokeState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  invoke: (...args: unknown[]) => Promise<T | null>;
}

export function useInvoke<T>(_command: string): InvokeState<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const invoke = useCallback(async (..._args: unknown[]): Promise<T | null> => {
    setLoading(true);
    setError(null);
    try {
      // Will use @tauri-apps/api/core invoke
      const result = null as T | null;
      setData(result);
      setLoading(false);
      return result;
    } catch (err) {
      setError(String(err));
      setLoading(false);
      return null;
    }
  }, []);

  return { data, loading, error, invoke };
}
