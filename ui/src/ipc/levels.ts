// The three levels (D-179, docs/app/LEVELS.md §4.1; Rust: tauri/src/levels/):
// where a program of Atollo runs, the label every terminal shows (§8.3),
// and the refusal of a command with its reason and what to do.
import type { AgentId } from "./agents";
import { query, subscribe } from "./core";
import { LEVELS_CHANGED } from "./events";
import type { IsolationBlocker } from "./isolation";

export type Level = "manual" | "project" | "agent";

/** The label shown on every terminal (8.3). */
export type LevelLabel = "pc" | "projectVm" | "agentVm";

export type Refusal =
  | { code: "agents_need_vm"; blockers: IsolationBlocker[] }
  | { code: "agent_outside_project" }
  | { code: "not_a_project" }
  | { code: "no_path_in_vm"; dir: string }
  | { code: "pools_only_without_vm" }
  | { code: "no_linux_build"; agent: AgentId }
  | { code: "daemon_restart_pending" };

export type Fix =
  "open_vm_notice" | "open_from_project" | "use_local_disk" | "start_from_project" | "restart_daemon" | "none";

/** The error of a refused command: `CmdError` with `refusal` set. Any
 * other error of a command is its plain text. */
export interface RefusedError {
  message: string;
  refusal: Refusal;
  fix: Fix;
}

/** True for the error of a command the levels refused. */
export function isRefused(e: unknown): e is RefusedError {
  if (typeof e !== "object" || e === null) return false;
  const r = e as Partial<RefusedError>;
  return typeof r.message === "string" && typeof r.fix === "string" && typeof r.refusal === "object";
}

/** The text of a command's error: a refusal's message, else as before
 * (`String(e)`: the text Rust sent). */
export function errorText(e: unknown): string {
  return isRefused(e) ? e.message : String(e);
}

/** The fix of a refused command's error; null for any other error. */
export function fixOf(e: unknown): Fix | null {
  return isRefused(e) ? e.fix : null;
}

/** The levels now (LEVELS.md §4.1): the VM on or off, why, and whether the
 * agents are enabled (I6). */
export interface LevelsState {
  vm: "on" | "off";
  /** Why the VM is off and what to do, null when on. */
  reason: string | null;
  /** False exactly when vm is "off". */
  agentsEnabled: boolean;
}

export function levelsState(): Promise<LevelsState> {
  return query<LevelsState>("levels_state", { vm: "on", reason: null, agentsEnabled: true });
}

/** Every flip of the agents' availability (`levels-changed`, §4.2): what
 * shows the agents is read again. */
export function onLevelsChanged(handler: (state: LevelsState) => void): Promise<() => void> {
  return subscribe<LevelsState>(LEVELS_CHANGED, handler);
}
