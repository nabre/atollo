// The projects' actions started by the kit (tauri/src/projects/run.rs): state,
// output, ports. Rust: tauri/src/commands/runs.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, query, subscribe } from "./core";
import { previewLog, previewRuns, previewStart, previewStop } from "./preview-runs";
import type { ActionKind } from "./projects";

export type RunState = "starting" | "running" | "finished" | "error" | "stopped";

export interface RunInfo {
  /** "<project>|<action>". */
  key: string;
  project: string;
  action: string;
  label: string;
  type: ActionKind;
  command: string;
  port: number | null;
  url: string | null;
  state: RunState;
  start: number;
  end: number | null;
  code: number | null;
  pid: number | null;
  /** URLs of the site's aliases left out of its configuration. */
  skippedAliases: string[];
}

/** A log as read: `seq` is the number of the last line written to any log
 * then, over all the actions; an event's lines come after it when their
 * numbers are higher (BUG-V7-30). */
export interface RunLog {
  lines: string[];
  seq: number;
}

/** In a "log" event, `seq` is the number of the last of `lines`. */
export type RunEvent = ({ kind: "state" } & RunInfo) | { kind: "log"; key: string; lines: string[]; seq: number };

export interface Ports {
  ports: Record<string, number>;
  locked: string[];
}

export function runList(): Promise<RunInfo[]> {
  return query<RunInfo[]>("run_list", browserPreview ? previewRuns() : []);
}

export function runStart(project: string, action: string): Promise<RunInfo> {
  if (browserPreview) return Promise.resolve(previewStart(project, action));
  return invoke<RunInfo>("run_start", { project, action });
}

export function runStop(key: string): Promise<void> {
  if (browserPreview) previewStop(key);
  return act("run_stop", { key });
}

export function runLog(key: string): Promise<RunLog> {
  return query<RunLog>("run_log", browserPreview ? previewLog(key) : { lines: [], seq: 0 }, { key });
}

export function runClear(key: string): Promise<void> {
  return act("run_clear", { key });
}

export function runPorts(): Promise<Ports> {
  return query<Ports>("run_ports", { ports: {}, locked: [] });
}

/** The user's port for an action (`null`: automatic again). */
export function runSetPort(key: string, port: number | null): Promise<Ports> {
  return invoke<Ports>("run_set_port", { key, port });
}

/** State changes and output of the projects' actions. */
export function onRun(handler: (e: RunEvent) => void): Promise<() => void> {
  return subscribe<RunEvent>("run", handler);
}
