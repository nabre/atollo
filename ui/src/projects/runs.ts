// The actions started in this session and their state, for the app, the
// Activity panel and the projects' pages.
import { useCallback, useRef, useState, useSyncExternalStore } from "react";
import { onRun, runClear, runLog, runList, type RunInfo, type RunLog, type RunState } from "../ipc";
import { t } from "../i18n";
import { frameBatch } from "../shared/frame";
import { useLoad, useSubscription } from "../shared/hooks";

const MAX_LINES = 3000;

export const alive = (s: RunState | undefined) => s === "starting" || s === "running";

const capped = (lines: string[]) => (lines.length > MAX_LINES ? lines.slice(-MAX_LINES) : lines);

/** The lines of a "log" event: `seq` is the number of the last. */
export interface LogBatch {
  lines: string[];
  seq: number;
}

/** The log read, followed by the lines that came as events while it was
 * read: the read is a snapshot from some moment in between, so the first
 * of those lines may already be in it. They are told apart by their
 * numbers (RunLog), never by their text: a log repeats lines (a blank one,
 * a progress dot), and a new line like the last read is still new. */
export function mergeLog(read: RunLog, arrived: LogBatch[]): string[] {
  const later = new Map<number, string>();
  for (const batch of arrived) {
    const first = batch.seq - batch.lines.length + 1;
    batch.lines.forEach((line, i) => {
      if (first + i > read.seq) later.set(first + i, line);
    });
  }
  return [...read.lines, ...[...later].sort(([a], [b]) => a - b).map(([, line]) => line)];
}

/** An action's output as kept: its last MAX_LINES lines, `first` the
 * number of the first of them (the lines dropped before it), so a view can
 * tell the lines that stayed from the ones that came. */
export interface RunOutput {
  lines: readonly string[];
  first: number;
}

const NO_OUTPUT: RunOutput = { lines: [], first: 0 };

/** The actions' output, kept apart from the app's state: a line that
 * arrives renders only the views showing that action (`useRunOutput`), and
 * the lines of a frame render together. */
export interface RunLogs {
  get: (key: string) => RunOutput;
  subscribe: (key: string, listener: () => void) => () => void;
  /** Lines arrived as an event, shown at the next frame. */
  append: (key: string, lines: string[]) => void;
  /** The whole log, as read: replaces what was kept and what waits. */
  replace: (key: string, lines: string[]) => void;
}

/** The output of the actions, `initial` already in (the tests'). */
export function createRunLogs(initial: Record<string, string[]> = {}): RunLogs {
  const logs = new Map<string, RunOutput>(
    Object.entries(initial).map(([key, lines]) => [key, { lines: capped(lines), first: 0 }]),
  );
  const listeners = new Map<string, Set<() => void>>();
  // The lines arrived since the last frame, as a RunOutput: `first` counts
  // the ones dropped already, when more than MAX_LINES came.
  const waiting = new Map<string, RunOutput>();
  const add = (to: RunOutput, lines: readonly string[]): RunOutput => {
    const all = [...to.lines, ...lines];
    const dropped = Math.max(0, all.length - MAX_LINES);
    return { lines: dropped > 0 ? all.slice(dropped) : all, first: to.first + dropped };
  };
  const tell = (key: string) => {
    for (const listener of listeners.get(key) ?? []) listener();
  };
  const frame = frameBatch(() => {
    for (const [key, come] of waiting) {
      const kept = add(logs.get(key) ?? NO_OUTPUT, come.lines);
      logs.set(key, { lines: kept.lines, first: kept.first + come.first });
    }
    const changed = [...waiting.keys()];
    waiting.clear();
    for (const key of changed) tell(key);
  });
  return {
    get: (key) => logs.get(key) ?? NO_OUTPUT,
    subscribe: (key, listener) => {
      const set = listeners.get(key) ?? new Set();
      listeners.set(key, set.add(listener));
      return () => {
        set.delete(listener);
        if (set.size === 0) listeners.delete(key);
      };
    },
    append: (key, lines) => {
      // While the window is hidden no frame comes: what waits stays capped.
      waiting.set(key, add(waiting.get(key) ?? NO_OUTPUT, lines));
      frame.ask();
    },
    replace: (key, lines) => {
      // The lines waiting arrived before this read answered: they are in it.
      waiting.delete(key);
      logs.set(key, { lines: capped(lines), first: 0 });
      tell(key);
    },
  };
}

/** The output of the action `key`, rendering the component when it
 * changes (at most once a frame). */
export function useRunOutput(logs: RunLogs, key: string): RunOutput {
  const subscribe = useCallback((listener: () => void) => logs.subscribe(key, listener), [logs, key]);
  return useSyncExternalStore(subscribe, () => logs.get(key));
}

/** The actions started in this session, kept current by the "run" events;
 * their output in `logs`, which renders only the views showing it. */
export function useRuns() {
  const [logs] = useState(() => createRunLogs());
  const live = useRef<Record<string, RunInfo>>({});
  // For each log being read, the lines that came as events meanwhile: the
  // answer would otherwise replace them with an older snapshot.
  const reading = useRef(new Map<string, Set<LogBatch[]>>());
  const read = useCallback(
    () => runList().then((list) => ({ ...Object.fromEntries(list.map((r) => [r.key, r])), ...live.current })),
    [],
  );
  const { data: runs, setData: setRuns } = useLoad<Record<string, RunInfo>>(read, {}, { onError: "keep" });
  useSubscription(onRun, (e) => {
    if (e.kind === "state") {
      live.current[e.key] = e;
      setRuns((r) => ({ ...r, [e.key]: e }));
    } else {
      for (const buffer of reading.current.get(e.key) ?? []) buffer.push({ lines: e.lines, seq: e.seq });
      logs.append(e.key, e.lines);
    }
  });
  const loadLog = useCallback(
    (key: string) => {
      const arrived: LogBatch[] = [];
      const buffers = reading.current.get(key) ?? new Set<LogBatch[]>();
      reading.current.set(key, buffers.add(arrived));
      const done = () => {
        buffers.delete(arrived);
        if (buffers.size === 0) reading.current.delete(key);
      };
      runLog(key).then((read) => {
        done();
        logs.replace(key, mergeLog(read, arrived));
      }, done);
    },
    [logs],
  );
  const forget = useCallback(
    (key: string) => {
      void runClear(key).then(() => {
        live.current = Object.fromEntries(Object.entries(live.current).filter(([k]) => k !== key));
        setRuns((r) => Object.fromEntries(Object.entries(r).filter(([k]) => k !== key)));
      });
    },
    [setRuns],
  );
  return { runs, logs, loadLog, forget };
}

export function stateText(r: RunInfo): string {
  switch (r.state) {
    case "starting":
      return t("in avvio");
    case "running":
      return r.type === "web" ? t("in ascolto") : t("in esecuzione");
    case "finished":
      return t("terminato");
    case "stopped":
      return t("fermato");
    case "error":
      return r.code === null ? t("errore") : t("errore (codice {0})", r.code);
  }
}
