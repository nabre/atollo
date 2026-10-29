// Hooks shared by the pages, for the two ways Rust keeps them current:
// events it sends (`useSubscription`, rendered once a frame with
// `useBatched` when they come many at a time) and state read again every few seconds
// (`useEvery`); a value read once or every few seconds, with what a failed
// read does (`useLoad`); an action with its wait and its error (`useAction`);
// a copy to the clipboard (`useCopy`); a value kept while its page is not
// shown (`useKept`).
import { useCallback, useEffect, useRef, useState } from "react";
import { errorText, fixOf, type Fix } from "../ipc/levels";
import { frameBatch } from "./frame";
import { same, whileShown } from "./poll";

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

/** A function that gathers what it is given and hands it to `flush` all
 * together at the next frame (`frameBatch`): events that come by the dozen
 * a second render once a frame. `flush` can be a new function at every
 * render, the latest one is called; nothing is called once the component
 * is gone. */
export function useBatched<T>(flush: (items: T[]) => void): (item: T) => void {
  const latest = useRef(flush);
  useEffect(() => {
    latest.current = flush;
  });
  const items = useRef<T[]>([]);
  const frame = useRef<ReturnType<typeof frameBatch> | null>(null);
  useEffect(() => {
    const batch = frameBatch(() => {
      const ready = items.current;
      items.current = [];
      latest.current(ready);
    });
    frame.current = batch;
    return () => {
      batch.cancel();
      frame.current = null;
    };
  }, []);
  return useCallback((item: T) => {
    items.current.push(item);
    frame.current?.ask();
  }, []);
}

/** Calls `read` now and every `ms` milliseconds while the component is
 * mounted, and again when `read` changes (keep it stable with useCallback).
 * Not while the window is hidden: then at once when it shows again
 * (`whileShown`). */
export function useEvery(read: () => void, ms: number): void {
  useEffect(() => {
    read();
    return whileShown(read, ms);
  }, [read, ms]);
}

/** What a failed read does: `"keep"` leaves the value as it was, `"show"`
 * leaves it too and puts the error in `error` (cleared by the next read that
 * works), `{ fallback }` replaces the value. */
export type OnLoadError<T> = "keep" | "show" | { fallback: T };

export interface LoadOptions<T> {
  onError: OnLoadError<T>;
  /** Read again every `every` milliseconds, as `useEvery` (not while the
   * window is hidden). */
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
 * is gone. An answer with the same data as the value (`same`) keeps the
 * value as it is: a poll that finds nothing new renders nothing.
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
  // What the component shows, so that an answer changing nothing sets no
  // state: a state set renders the component even when React then finds
  // the value unchanged.
  const shown = useRef({ data, error });
  useEffect(() => {
    shown.current = { data, error };
  });
  // The read, resolving once its answer is in: true when it worked (and is
  // still the latest read), false otherwise.
  const read = useCallback((): Promise<boolean> => {
    if (!enabled) return Promise.resolve(false);
    const id = ++seq.current;
    const show = (value: T) => {
      if (same(shown.current.data, value)) return;
      shown.current = { ...shown.current, data: value };
      setData(value);
    };
    return load().then(
      (value) => {
        if (!mounted.current || id !== seq.current) return false;
        show(value);
        if (shown.current.error !== null) {
          shown.current = { ...shown.current, error: null };
          setError(null);
        }
        return true;
      },
      (e: unknown) => {
        if (!mounted.current || id !== seq.current) return false;
        const p = policy.current;
        if (p === "show") {
          shown.current = { ...shown.current, error: String(e) };
          setError(String(e));
        } else if (p !== "keep") show(p.fallback);
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
    return whileShown(reload, every);
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

/** The values of `useKept`, by key, for as long as the window is open. */
const kept = new Map<string, unknown>();

/**
 * As useState, but the value stays when the component goes away: the next
 * one with the same `key` starts from it (a page shown again, a form half
 * filled in). `null` keeps nothing, as useState.
 */
export function useKept<T>(key: string | null, initial: T): [T, (next: T | ((current: T) => T)) => void] {
  const [value, setValue] = useState<T>(() => (key !== null && kept.has(key) ? (kept.get(key) as T) : initial));
  const set = useCallback(
    (next: T | ((current: T) => T)) => {
      setValue((current) => {
        const v = typeof next === "function" ? (next as (current: T) => T)(current) : next;
        if (key !== null) kept.set(key, v);
        return v;
      });
    },
    [key],
  );
  return [value, set];
}
