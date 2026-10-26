// Hooks shared by the pages, for the two ways Rust keeps them current:
// events it sends (`useSubscription`) and state read again every few seconds
// (`useEvery`); a value read once or every few seconds, with what a failed
// read does (`useLoad`); an action with its wait and its error (`useAction`);
// a copy to the clipboard (`useCopy`).
import { useCallback, useEffect, useRef, useState } from "react";
import { errorText, fixOf, type Fix } from "../ipc/levels";

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

/** What a failed read does: `"keep"` leaves the value as it was, `"show"`
 * leaves it too and puts the error in `error` (cleared by the next read that
 * works), `{ fallback }` replaces the value. */
export type OnLoadError<T> = "keep" | "show" | { fallback: T };

export interface LoadOptions<T> {
  onError: OnLoadError<T>;
  /** Read again every `every` milliseconds, as `useEvery`. */
  every?: number;
  /** False: nothing is read (`reload` does nothing either). */
  enabled?: boolean;
  /** Read again when it changes, besides when `load` changes. */
  key?: unknown;
}

/**
 * A value read with `load` when the component mounts and again when `load`
 * changes (keep it stable with useCallback), or on `reload` (`retry` is the
 * same, and resolves to whether the read worked: "Riprova"). `setData`
 * changes it in between, as events do. Nothing is set once the component
 * is gone.
 */
export function useLoad<T>(load: () => Promise<T>, initial: T, options: LoadOptions<T>) {
  const { onError, every, enabled = true, key } = options;
  const [data, setData] = useState<T>(initial);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);
  const seq = useRef(0);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  // Read when the answer comes: a new policy is no reason to read again.
  const policy = useRef(onError);
  useEffect(() => {
    policy.current = onError;
  });
  // The read, resolving once its answer is in: true when it worked (and is
  // still the latest read), false otherwise.
  const read = useCallback((): Promise<boolean> => {
    if (!enabled) return Promise.resolve(false);
    const id = ++seq.current;
    return load().then(
      (value) => {
        if (!mounted.current || id !== seq.current) return false;
        setData(value);
        setError(null);
        return true;
      },
      (e: unknown) => {
        if (!mounted.current || id !== seq.current) return false;
        const p = policy.current;
        if (p === "show") setError(String(e));
        else if (p !== "keep") setData(p.fallback);
        return false;
      },
    );
  }, [load, enabled]);
  const reload = useCallback(() => {
    void read();
  }, [read]);
  useEffect(() => {
    reload();
    if (every === undefined) return;
    const timer = window.setInterval(reload, every);
    return () => {
      window.clearInterval(timer);
    };
  }, [reload, every, key]);
  return { data, error, reload, retry: read, setData };
}

/** Starts `action` now; a throw before its promise becomes a rejection. */
function attempt(action: () => Promise<void>): Promise<void> {
  try {
    return action();
  } catch (e: unknown) {
    return Promise.reject(e instanceof Error ? e : new Error(String(e)));
  }
}

/** Runs an action, keeping track of what is busy and of the last error;
 * `fix`: what to do about it when the levels refused the command
 * (D-179, LEVELS.md §4.1). */
export function useAction(onDone?: () => void) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [fix, setFix] = useState<Fix | null>(null);
  const mounted = useRef(true);
  const seq = useRef(0);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  const run = useCallback(
    (key: string, action: () => Promise<void>) => {
      const id = ++seq.current;
      setBusy(key);
      setError(null);
      setFix(null);
      attempt(action).then(
        () => {
          if (!mounted.current) return;
          if (id === seq.current) setBusy(null);
          onDone?.();
        },
        (e: unknown) => {
          if (!mounted.current) return;
          if (id === seq.current) setBusy(null);
          setError(errorText(e));
          setFix(fixOf(e));
        },
      );
    },
    [onDone],
  );
  return { busy, error, fix, run };
}

/** Copies text to the clipboard; `copied` turns true once it is there, and
 * back to false after `resetMs` when given (or on `reset`). */
export function useCopy(resetMs?: number) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(timer.current);
    },
    [],
  );
  const copy = useCallback(
    (text: string) => {
      // A refused copy (no permission, no focus) says nothing was copied.
      attempt(() => navigator.clipboard.writeText(text)).then(
        () => {
          setCopied(true);
          window.clearTimeout(timer.current);
          if (resetMs !== undefined) {
            timer.current = window.setTimeout(() => {
              setCopied(false);
            }, resetMs);
          }
        },
        () => {
          window.clearTimeout(timer.current);
          setCopied(false);
        },
      );
    },
    [resetMs],
  );
  const reset = useCallback(() => {
    setCopied(false);
  }, []);
  return { copied, copy, reset };
}
