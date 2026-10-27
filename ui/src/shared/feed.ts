// A value Rust keeps current with its events (the jobs, the preparation's
// snapshot), kept apart from React: read once when the first component
// shows it, followed by the events, forgotten when the last one goes. An
// install sends an event per percent; they change the value once a frame,
// and a component renders only when the part it chose (`select`) changes.
import { useSyncExternalStore } from "react";
import { frameBatch } from "./frame";
import type { Subscribe } from "./hooks";

export interface Feed<T> {
  subscribe: (listener: () => void) => () => void;
  get: () => T;
}

/**
 * A feed starting from `initial`: `listen` sends the events, each one
 * folded into the value with `apply`; `read` is the first read, resolving
 * to the change it makes to the value (events that came meanwhile are
 * newer than it, and already in the value it is given).
 */
export function createFeed<T, E>({
  initial,
  listen,
  apply,
  read,
}: {
  initial: T;
  listen: Subscribe<E>;
  apply: (value: T, event: E) => T;
  read: () => Promise<(value: T) => T>;
}): Feed<T> {
  let value = initial;
  let events: E[] = [];
  const listeners = new Set<() => void>();
  // The Tauri listener and the first read belong to a `round`: once the
  // last component goes, whatever comes from them is dropped.
  let round = 0;
  let stop: (() => void) | null = null;
  const set = (next: T) => {
    if (next === value) return;
    value = next;
    for (const listener of listeners) listener();
  };
  const fold = () => {
    const come = events;
    events = [];
    return come.reduce(apply, value);
  };
  const frame = frameBatch(() => {
    set(fold());
  });
  const start = () => {
    const at = round;
    listen((event) => {
      if (at !== round) return;
      events.push(event);
      frame.ask();
    }).then(
      (unlisten) => {
        if (at === round) stop = unlisten;
        else unlisten();
      },
      () => undefined,
    );
    read().then(
      (change) => {
        if (at === round) set(change(fold()));
      },
      () => undefined,
    );
  };
  const end = () => {
    round++;
    stop?.();
    stop = null;
    frame.cancel();
    events = [];
    value = initial;
  };
  return {
    subscribe: (listener) => {
      listeners.add(listener);
      if (listeners.size === 1) start();
      return () => {
        listeners.delete(listener);
        if (listeners.size === 0) end();
      };
    },
    get: () => value,
  };
}

/** The feed's value, or the part of it `select` chooses (a part of the
 * value or a plain value, not a new object: the component renders when it
 * is another one). */
export function useFeed<T>(feed: Feed<T>): T;
export function useFeed<T, S>(feed: Feed<T>, select: (value: T) => S): S;
export function useFeed<T, S>(feed: Feed<T>, select?: (value: T) => S): T | S {
  return useSyncExternalStore(feed.subscribe, () => (select ? select(feed.get()) : feed.get()));
}
