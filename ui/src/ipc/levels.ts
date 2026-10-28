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
  | { code: "no_path_in_vm"; dir: string }
  | { code: "no_linux_build"; agent: AgentId }
  | { code: "daemon_restart_pending" };

export type Fix = "open_vm_notice" | "open_from_project" | "use_local_disk" | "restart_daemon" | "none";

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

/** The levels as `levels-changed` tells them (LEVELS.md §4.1): the VM on
 * or off, why, and whether the agents are enabled (I6). */
export interface LevelsState {
  vm: "on" | "off";
  /** Why the VM is off and what to do, null when on. */
  reason: string | null;
  /** False exactly when vm is "off". */
  agentsEnabled: boolean;
}

/** Every flip of the agents' availability (`levels-changed`, §4.2): what
 * shows the agents is read again. */
export function onLevelsChanged(handler: (state: LevelsState) => void): Promise<() => void> {
  return subscribe<LevelsState>(LEVELS_CHANGED, handler);
}

// The project's VM as the user can check it (LEVELS.md §8.1, §8.2, §8.5;
// Rust: tauri/src/levels/report.rs, the commands in tauri/src/ipc/levels.rs).

/** A sandbox's state as Docker Sandboxes says it. */
export type SandboxRun = "running" | "stopped" | "missing";

/** A tool as «Verifica strumenti» found it in a sandbox. */
export type Verdict = "ok" | "otherLine" | "missing" | "notFromKit" | "notInVm";

export interface ToolCheck {
  id: string;
  name: string;
  /** The project's line, else the kit's; null for an unversioned tool. */
  chosen: string | null;
  /** What ran, verbatim. */
  command: string;
  path: string | null;
  found: string | null;
  verdict: Verdict;
  /** The first 3 lines of its output. */
  output: string;
}

/** What the read-only `atollo` of the sandbox prints (§8.4). */
export interface SandboxInfo {
  v: number;
  level: Level;
  sandbox: string;
  project: string;
  agent: AgentId | null;
  workdir: string;
  tools: Record<string, string | null>;
  databases: { engine: string; database: string; user: string; address: string }[];
  hostPorts: number[];
}

export interface SandboxReport {
  name: string;
  /** The agent of an agent's own sandbox; null for the project's. */
  agent: AgentId | null;
  state: SandboxRun;
  lastUsed: string | null;
  mounts: { host: string; guest: string; mode: "rw" | "ro" }[];
  hostPorts: { port: number; what: string }[];
  info: SandboxInfo | null;
  /** The app's sessions running in it: «Riavvia» and «Ricrea» stop them. */
  sessions: string[];
  /** «Verifica strumenti», once it ran. */
  tools: ToolCheck[] | null;
  /** Why the check found nothing (the sandbox is off). */
  note: string | null;
}

export interface VmReport {
  project: string;
  vm: "on" | "off";
  /** Why the VM is off (the notice's text), null when on. */
  reason: string | null;
  /** The project's sandbox, then each agent's. */
  sandboxes: SandboxReport[];
  github: boolean;
}

/** An action on a sandbox (§8.5). */
export type VmAction = "restart" | "recreate" | "refresh";

const PREVIEW_REPORT: VmReport = {
  project: "shop",
  vm: "on",
  reason: null,
  sandboxes: [
    {
      name: "atollo-p-shop-0123456789abcdef01234567",
      agent: null,
      state: "running",
      lastUsed: null,
      mounts: [
        { host: "D:\kit\projects\shop", guest: "/d/kit/projects/shop", mode: "rw" },
        { host: "D:\kit\data\apps\linux-x64", guest: "/d/kit/data/apps/linux-x64", mode: "ro" },
      ],
      hostPorts: [{ port: 3307, what: "mysql shop" }],
      info: null,
      sessions: [],
      tools: null,
      note: null,
    },
  ],
  github: false,
};

/** The report of `project`'s VM; with `check`, «Verifica strumenti» in
 * each of its sandboxes. */
export function projectVmReport(project: string, check = false): Promise<VmReport> {
  return query<VmReport>("project_vm_report", PREVIEW_REPORT, { project, check });
}

/** «Verifica strumenti» in one sandbox: the project's, or `agent`'s own. */
export function projectVmCheck(project: string, agent: AgentId | null): Promise<SandboxReport> {
  const sandbox = PREVIEW_REPORT.sandboxes[0] as SandboxReport;
  return query<SandboxReport>("project_vm_check", { ...sandbox, tools: [] }, { project, agent });
}

/** An action on a sandbox of `project`; the sessions it stopped. */
export function projectVmAction(project: string, agent: AgentId | null, action: VmAction): Promise<string[]> {
  return query<string[]>("project_vm_action", [], { project, agent, action });
}
