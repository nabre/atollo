// Hooks shared by the pages, for the two ways Rust keeps them current:
// events it sends (`useSubscription`) and state read again every few seconds
// (`useEvery`); and an action with its wait and its error (`useAction`).
import { useCallback, useEffect, useRef, useState } from "react";

/** One of the `on…` functions of ./api: starts listening, resolves to the
 * function that stops. */
export type Subscribe<T> = (handler: (value: T) => void) => Promise<() => void>;

/**
 * Sends every event of `subscribe` to `handler` while the component is
 * mounted. The listener is registered once (per `subscribe`): `handler` can
 * be a new function at every render, the latest one receives the events.
 * When the component goes away before listening started, the listener is
 * removed as soon as it exists.
 */
export function useSubscription<T>(subscribe: Subscribe<T>, handler: (value: T) => void): void {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    let stop: (() => void) | null = null;
    let gone = false;
    subscribe((value) => {
      latest.current(value);
    }).then(
      (unlisten) => {
        if (gone) unlisten();
        else stop = unlisten;
      },
      () => undefined,
    );
    return () => {
      gone = true;
      stop?.();
    };
  }, [subscribe]);
}

/** Calls `read` now and every `ms` milliseconds while the component is
 * mounted, and again when `read` changes (keep it stable with useCallback). */
export function useEvery(read: () => void, ms: number): void {
  useEffect(() => {
    read();
    const timer = window.setInterval(read, ms);
    return () => {
      window.clearInterval(timer);
    };
  }, [read, ms]);
}

/** Runs an action, keeping track of what is busy and of the last error. */
export function useAction(onDone?: () => void) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = useCallback(
    (key: string, action: () => Promise<void>) => {
      setBusy(key);
      setError(null);
      action().then(
        () => {
          setBusy(null);
          onDone?.();
        },
        (e: unknown) => {
          setBusy(null);
          setError(String(e));
        },
      );
    },
    [onDone],
  );
  return { busy, error, run };
}
