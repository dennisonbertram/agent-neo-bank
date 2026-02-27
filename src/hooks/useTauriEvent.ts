import { useEffect } from "react";

export function useTauriEvent<T>(_event: string, _handler: (payload: T) => void) {
  useEffect(() => {
    // Will be implemented to use @tauri-apps/api/event listen
    return () => {};
  }, [_event, _handler]);
}
