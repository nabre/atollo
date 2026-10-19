// The projects' actions in the browser preview (core.ts: no Rust there): a
// start of a sample project goes from «starting» to «running» on a port of
// the kit's range, with a few lines of output, through the same "run"
// events the page gets from Rust (tauri/src/projects/run.rs); a stop ends it.
// The page and the Activity panel can be designed and shown with what runs.
import { previewEmit } from "./core";
import type { RunEvent, RunInfo } from "./runs";
import { SAMPLE_PROJECTS } from "./projects";

const runs = new Map<string, RunInfo>();
const logs = new Map<string, string[]>();
/** The first port the kit gives a web action (PORT_FROM in run.rs). */
let nextPort = 7100;

function emit(e: RunEvent) {
  previewEmit("run", e);
}

function log(key: string, lines: string[]) {
  logs.set(key, [...(logs.get(key) ?? []), ...lines]);
  emit({ kind: "log", key, lines });
}

/** What the page shows of a start: the sample action, its port, its output. */
export function previewStart(project: string, action: string): RunInfo {
  const key = `${project}|${action}`;
  const live = runs.get(key);
  if (live && (live.state === "starting" || live.state === "running")) return live;
  const a = SAMPLE_PROJECTS.find((p) => p.id === project)?.actions.find((x) => x.id === action);
  const web = (a?.type ?? "web") === "web";
  const port = web ? (live?.port ?? nextPort++) : null;
  const command =
    a?.static != null
      ? `Atollo serve ${a.static} ${String(port)}`
      : (a?.command ?? action).replace("{port}", String(port));
  const info: RunInfo = {
    key,
    project,
    action,
    label: a?.label ?? action,
    type: a?.type ?? "web",
    command,
    port,
    url: port === null ? null : `http://localhost:${String(port)}/`,
    state: "starting",
    start: Date.now(),
    end: null,
    code: null,
    pid: 4000 + runs.size,
    skippedAliases: [],
  };
  runs.set(key, info);
  logs.set(key, []);
  emit({ kind: "state", ...info });
  log(key, [`> ${command}`]);
  window.setTimeout(() => {
    const now = runs.get(key);
    if (now?.state !== "starting") return;
    const running = { ...now, state: "running" as const };
    runs.set(key, running);
    emit({ kind: "state", ...running });
    log(
      key,
      a?.static != null
        ? [`File server su http://localhost:${String(port)}/ (cartella ${a.static})`, "GET / 200", "GET /style.css 200"]
        : [
            "INFO:     Will watch for changes in these directories: ['.']",
            `INFO:     Uvicorn running on http://127.0.0.1:${String(port)} (Press CTRL+C to quit)`,
            "INFO:     Started reloader process",
            "INFO:     Application startup complete.",
          ],
    );
  }, 700);
  return info;
}

/** A stop of a preview run. */
export function previewStop(key: string): void {
  const now = runs.get(key);
  if (!now || (now.state !== "starting" && now.state !== "running")) return;
  const stopped = { ...now, state: "stopped" as const, end: Date.now() };
  runs.set(key, stopped);
  emit({ kind: "state", ...stopped });
  log(key, ["Fermato."]);
}

export function previewRuns(): RunInfo[] {
  return [...runs.values()];
}

export function previewLog(key: string): string[] {
  return logs.get(key) ?? [];
}
