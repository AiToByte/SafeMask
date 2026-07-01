import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Safely subscribes to a Tauri event with React lifecycle cleanup.
 * Handles React StrictMode double-mount correctly via ref tracking.
 */
export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void,
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    listen<T>(eventName, (event) => {
      if (!cancelled) {
        handlerRef.current(event.payload);
      }
    }).then((fn) => {
      if (!cancelled) {
        unlisten = fn;
      } else {
        fn(); // late arrival — clean up immediately
      }
    });

    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }, [eventName]);
}
