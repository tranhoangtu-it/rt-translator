import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

/**
 * Generic hook to listen for Tauri backend events.
 * Cleans up listener on unmount or when eventName changes.
 * Uses mounted flag to handle async listen() resolving after unmount.
 */
export function useTauriEvent<T>(
  eventName: string,
  callback: (payload: T) => void,
): void {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    let mounted = true;
    let unlisten: UnlistenFn | undefined;

    listen<T>(eventName, (event) => {
      if (mounted) callbackRef.current(event.payload);
    }).then((fn) => {
      if (mounted) {
        unlisten = fn;
      } else {
        fn(); // already unmounted, immediately unlisten
      }
    });

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [eventName]);
}
