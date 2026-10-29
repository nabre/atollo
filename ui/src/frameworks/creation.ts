// The creation of a project from the wizard, kept outside the page (D-190):
// it goes on while other pages are shown, and its progress is there again
// when the wizard comes back. The app keeps it too (frameworks_state), so a
// reload of the window finds it; its lines come as `framework` events,
// applied once a frame (frameBatch), its end as `framework-end`.
import { useEffect, useSyncExternalStore } from "react";
import {
  frameworksCreate,
  frameworksState,
  NO_CREATION,
  onFramework,
  onFrameworkEnd,
  type EnvVar,
  type FrameworkCreation,
  type FrameworkEvent,
} from "../ipc";
import { frameBatch } from "../shared/frame";

const MAX_LOG = 600;

/** The creation as the pages show it: `run` grows with every creation this
 * window has known; `seen` is the run whose end the wizard has shown (the
 * status bar tells the others). */
export type Creation = FrameworkCreation & { run: number; seen: number };

let current: Creation = { ...NO_CREATION, run: 0, seen: 0 };
const listeners = new Set<() => void>();
let listening = false;

function set(next: FrameworkCreation, fresh = next.id !== current.id) {
  current = { ...next, run: current.run + (fresh ? 1 : 0), seen: current.seen };
  listeners.forEach((l) => {
    l();
  });
}

/** A step or a line of the creation `e.id`; a line already there (read
 * with the state) is not added again. */
export function applyEvent(c: FrameworkCreation, e: FrameworkEvent): FrameworkCreation {
  if (e.id < c.id) return c;
  const base: FrameworkCreation =
    e.id > c.id ? { ...c, id: e.id, running: true, lines: 0, log: [], error: null, created: null } : c;
  const text = e.command !== null ? `> ${e.command}` : e.line;
  const next = { ...base, step: e.step, total: e.total };
  if (text === null || e.seq <= base.lines) return next;
  return { ...next, lines: e.seq, log: [...base.log, text].slice(-MAX_LOG) };
}

/** The events of a frame, applied together. */
let pending: FrameworkEvent[] = [];
function flush() {
  const events = pending;
  pending = [];
  if (events.length > 0) set(events.reduce<FrameworkCreation>(applyEvent, current));
}
const batch = frameBatch(flush);

/** The events waiting, applied now: before an end. */
function flushNow() {
  batch.cancel();
  flush();
}

function listen() {
  if (listening) return;
  listening = true;
  void onFramework((e) => {
    pending.push(e);
    batch.ask();
  });
  void onFrameworkEnd((end) => {
    flushNow();
    if (end.id >= current.id) set(end);
  });
  frameworksState().then(
    (s) => {
      if (s.id > current.id || (s.id === current.id && s.lines > current.lines)) set(s);
    },
    () => undefined,
  );
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** The creation running now, or the last one. */
export function useCreation(): Creation {
  useEffect(listen, []);
  return useSyncExternalStore(subscribe, () => current);
}

/** The wizard has shown the end of the creation `run`. */
export function markSeen(run: number) {
  if (current.seen === run) return;
  current = { ...current, seen: run };
  listeners.forEach((l) => {
    l();
  });
}

/** Whether a creation runs: renders only when that changes. */
export function useCreationRunning(): boolean {
  useEffect(listen, []);
  return useSyncExternalStore(subscribe, () => current.running);
}

/** Starts a creation; its progress is in useCreation. */
export function startCreation(base: string, addons: string[], name: string, env: EnvVar[]) {
  listen();
  // Shown at once; the first event brings the creation's id.
  set({ ...NO_CREATION, id: current.id, running: true, base, addons, name, env }, true);
  const started = current.run;
  // The end event may come first; a newer creation is never touched.
  const mine = () => current.running && current.name === name && current.run >= started;
  frameworksCreate(base, addons, name, env).then(
    (project) => {
      flushNow();
      if (mine()) set({ ...current, running: false, created: project });
    },
    (e: unknown) => {
      // Refused before it started (a name taken, another creation, a
      // refusal of the levels with its fix), or ended without its event.
      flushNow();
      if (mine()) set({ ...current, running: false, error: e });
    },
  );
}
