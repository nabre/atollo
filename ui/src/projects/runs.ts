// The actions started in this session and their state, for the app, the
// Activity panel and the projects' pages.
import { useCallback, useRef, useState } from "react";
import { onRun, runClear, runLog, runList, type RunInfo, type RunState } from "../api";
import { t } from "../i18n";
import { useLoad, useSubscription } from "../hooks";

const MAX_LINES = 3000;

export const alive = (s: RunState | undefined) => s === "starting" || s === "running";

/** The actions started in this session, kept current by the "run" events. */
export function useRuns() {
  const [logs, setLogs] = useState<Record<string, string[]>>({});
  const live = useRef<Record<string, RunInfo>>({});
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
