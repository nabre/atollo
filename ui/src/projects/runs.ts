// The actions started in this session and their state, for the app, the
// Activity panel and the projects' pages.
import { useCallback, useRef, useState } from "react";
import { onRun, runClear, runLog, runList, type RunInfo, type RunLog, type RunState } from "../api";
import { t } from "../i18n";
import { useLoad, useSubscription } from "../hooks";

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

/** The actions started in this session, kept current by the "run" events. */
export function useRuns() {
  const [logs, setLogs] = useState<Record<string, string[]>>({});
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
      setLogs((l) => ({ ...l, [e.key]: capped([...(l[e.key] ?? []), ...e.lines]) }));
    }
  });
  const loadLog = useCallback((key: string) => {
    const arrived: LogBatch[] = [];
    const buffers = reading.current.get(key) ?? new Set<LogBatch[]>();
    reading.current.set(key, buffers.add(arrived));
    const done = () => {
      buffers.delete(arrived);
      if (buffers.size === 0) reading.current.delete(key);
    };
    runLog(key).then((read) => {
      done();
      const log = capped(mergeLog(read, arrived));
      setLogs((l) => ({ ...l, [key]: log }));
    }, done);
  }, []);
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
