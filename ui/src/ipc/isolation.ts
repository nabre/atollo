// How shut in the projects' programs are (D-167): the VM, or the kit's
// environment only with the reasons the VM is off.
import { act, query } from "./core";

/** Why the VM is off (tauri/src/isolation): the notice says it and what
 * to do about it, by `code`. */
export type IsolationBlocker =
  | { code: "old_windows"; build: number }
  | { code: "processor" }
  | { code: "old_macos"; version: string }
  | { code: "distribution"; name: string }
  | { code: "hypervisor_off" }
  | { code: "kvm_missing" }
  | { code: "kvm_denied" }
  | { code: "apparmor"; shim: string }
  | { code: "sbx_missing" }
  | { code: "sbx_signed_out" }
  | { code: "sbx_failed"; message: string };

export interface IsolationStatus {
  /** `vm`: a Docker Sandboxes microVM per project; `env`: the kit's
   * environment only, the programs on the PC. */
  level: "vm" | "env";
  /** Empty when the level is the VM. */
  blockers: IsolationBlocker[];
  /** The `sbx` found. */
  sbx: string | null;
}

const SAMPLE_ISOLATION: IsolationStatus = {
  level: "env",
  blockers: [{ code: "hypervisor_off" }, { code: "sbx_missing" }],
  sbx: null,
};

/** What the notice's «Lancia» runs in the app's terminal: `sbx login`,
 * `sbx ls`. */
export type IsolationFix = "login" | "ls";

/** Opens `fix` with the `sbx` found in a new session of the app's terminal. */
export function isolationRun(fix: IsolationFix): Promise<void> {
  return act("isolation_run", { fix });
}

/** `force`: ask `sbx` again now, not the answer of the last minute. */
export function isolationStatus(force = false): Promise<IsolationStatus> {
  return query<IsolationStatus>("isolation_status", SAMPLE_ISOLATION, { force });
}
