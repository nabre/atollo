// The actions started in this session and their state, for the app, the
// Activity panel and the projects' pages.
import { useCallback, useEffect, useState } from "react";
import { onRun, runClear, runList, runLog, type RunInfo, type RunState } from "../api";
import { t } from "../i18n";
import { useSubscription } from "../hooks";

const MAX_LINES = 3000;

export const alive = (s: RunState | undefined) => s === "starting" || s === "running";

/** The actions started in this session, kept current by the "run" events. */
export function useRuns() {
  const [runs, setRuns] = useState<Record<string, RunInfo>>({});
  const [logs, setLogs] = useState<Record<string, string[]>>({});
  const reload = useCallback(() => {
    runList().then(
      (list) => {
        setRuns(Object.fromEntries(list.map((r) => [r.key, r])));
      },
      () => undefined,
    );
  }, []);
  useEffect(reload, [reload]);
  useSubscription(onRun, (e) => {
    if (e.kind === "state") {
      setRuns((r) => ({ ...r, [e.key]: e }));
    } else {
      setLogs((l) => {
        const lines = [...(l[e.key] ?? []), ...e.lines];
        return { ...l, [e.key]: lines.length > MAX_LINES ? lines.slice(-MAX_LINES) : lines };
      });
    }
  });
  const loadLog = useCallback((key: string) => {
    runLog(key).then(
      (lines) => {
        setLogs((l) => ({ ...l, [key]: lines }));
      },
      () => undefined,
    );
  }, []);
  const forget = useCallback((key: string) => {
    void runClear(key).then(() => {
      setRuns((r) => Object.fromEntries(Object.entries(r).filter(([k]) => k !== key)));
    });
  }, []);
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
