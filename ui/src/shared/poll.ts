// What Rust is asked again every few seconds, kept so that only a change
// renders: answers compared with the last one (`same`), timers stopped while
// the window is hidden (`whileShown`), and one shared poll per command for
// the parts of the window that show it (`createPoll`, `usePoll`): the app's
// Activity counts and the page of the same things read it once, not twice.
import { useCallback, useEffect, useSyncExternalStore } from "react";

/**
 * Whether two answers of Rust carry the same data. They are compared as
 * JSON, which is what they were before they reached the page: plain
 * objects, arrays, strings, numbers, booleans and null, no functions,
 * dates or cycles, the keys in the order serde writes them. A few KB every
 * few seconds is far cheaper than the render of a window that a new object
 * would cause; keys in another order only cost that render, as before.
 */
export function same(a: unknown, b: unknown): boolean {
  if (Object.is(a, b)) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  try {
    return JSON.stringify(a) === JSON.stringify(b);
  } catch {
    return false;
  }
}

/** Calls `read` every `ms` milliseconds while the window is shown: never
 * while it is minimized or hidden (`document.hidden`), and once at once
 * when it shows again, so what it shows is current. Returns the function
 * that stops. */
export function whileShown(read: () => void, ms: number): () => void {
  let timer: number | undefined;
  const start = () => {
    timer ??= window.setInterval(read, ms);
  };
  const stop = () => {
    window.clearInterval(timer);
    timer = undefined;
  };
  const onChange = () => {
    if (document.hidden) {
      stop();
    } else if (timer === undefined) {
      read();
      start();
    }
  };
  if (!document.hidden) start();
  document.addEventListener("visibilitychange", onChange);
  return () => {
    stop();
    document.removeEventListener("visibilitychange", onChange);
  };
}

/** A poll's last answer: `value` undefined before the first one, `error`
 * why the last read failed (null when it worked). */
export interface PollState<T> {
  value: T | undefined;
  error: string | null;
}

/** One command of Rust read for every part of the window that shows it
 * (`usePoll`). */
export interface Poll<T> {
  subscribe: (listener: () => void) => () => void;
  get: () => PollState<T>;
  /** Reads now, with `using` in place of the usual read when given (a
   * fresh scan), even with a read on its way: that one's answer may come
   * from before an action, and is dropped. Resolves to whether this read
   * worked and was still the latest. */
  refresh: (using?: () => Promise<T>) => Promise<boolean>;
  /** An action's answer, newer than a read on its way (its answer is
   * dropped). */
  set: (value: T) => void;
  /** A part that shows it: reads now unless a read is on its way, then
   * every `ms` (none: only now). The fastest part sets the pace; when the
   * last one goes the poll stops and forgets its answer. Returns the
   * function that ends the request. */
  want: (ms: number | undefined) => () => void;
}

const NOTHING: PollState<never> = { value: undefined, error: null };

/** A poll of `load`: the timer never sends a read while another is on its
 * way (a slow answer does not pile up requests), and only the answer of
 * the latest read counts. */
export function createPoll<T>(load: () => Promise<T>): Poll<T> {
  let state: PollState<T> = NOTHING;
  const listeners = new Set<() => void>();
  const wants = new Map<object, number | undefined>();
  let pace: number | undefined;
  let stopTimer: (() => void) | null = null;
  // `latest` numbers the reads and the actions' answers: an answer counts
  // when nothing came after its read. `round` changes when the poll
  // forgets: answers of before are dropped.
  let latest = 0;
  let onTheWay = 0;
  let round = 0;

  const update = (value: T | undefined, error: string | null) => {
    const kept = same(value, state.value) ? state.value : value;
    if (kept === state.value && error === state.error) return;
    state = { value: kept, error };
    for (const listener of listeners) listener();
  };
  const read = (using: () => Promise<T> = load): Promise<boolean> => {
    const id = ++latest;
    const at = round;
    onTheWay++;
    const settle = (apply: () => void) => {
      if (at !== round) return false;
      onTheWay--;
      if (id !== latest) return false;
      apply();
      return true;
    };
    return new Promise<T>((resolve) => {
      resolve(using());
    }).then(
      (value) =>
        settle(() => {
          update(value, null);
        }),
      (e: unknown) => {
        settle(() => {
          update(state.value, String(e));
        });
        return false;
      },
    );
  };
  const tick = () => {
    if (onTheWay === 0) void read();
  };
  const repace = () => {
    let next: number | undefined;
    for (const ms of wants.values()) if (ms !== undefined && (next === undefined || ms < next)) next = ms;
    if (next === pace) return;
    stopTimer?.();
    stopTimer = null;
    pace = next;
    if (next !== undefined) stopTimer = whileShown(tick, next);
  };
  const forget = () => {
    stopTimer?.();
    stopTimer = null;
    pace = undefined;
    round++;
    onTheWay = 0;
    state = NOTHING;
  };
  return {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    get: () => state,
    refresh: (using) => read(using),
    set: (value) => {
      latest++;
      update(value, state.error);
    },
    want: (ms) => {
      const token = {};
      wants.set(token, ms);
      tick();
      repace();
      return () => {
        wants.delete(token);
        if (wants.size === 0) forget();
        else repace();
      };
    },
  };
}

export interface PollOptions {
  /** What a failed read does: `"keep"` leaves the value as it was, `"show"`
   * leaves it too and puts the error in `error` (cleared by the next read
   * that works), as `useLoad`. */
  onError: "keep" | "show";
  /** Read again every `every` milliseconds while the window is shown. */
  every?: number;
}

/**
 * `poll`'s answer while the component is mounted, as `useLoad` gives a read
 * of its own: `data` (`initial` before the first answer), `error`,
 * `reload`/`retry` (a read now, see `Poll.refresh`) and `setData` (an
 * action's answer, `Poll.set`). Every component on the same poll shares one
 * read; the component renders only when the answer changes.
 */
export function usePoll<T, I = T>(poll: Poll<T>, initial: I, { onError, every }: PollOptions) {
  const state = useSyncExternalStore(poll.subscribe, poll.get);
  useEffect(() => poll.want(every), [poll, every]);
  const reload = useCallback(() => {
    void poll.refresh();
  }, [poll]);
  const retry = useCallback(() => poll.refresh(), [poll]);
  return {
    data: state.value === undefined ? initial : state.value,
    error: onError === "show" ? state.error : null,
    reload,
    retry,
    setData: poll.set,
  };
}
