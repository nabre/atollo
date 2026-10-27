// How shut in the projects' programs are (D-167): the VM, or the kit's
// environment only with the reasons the VM is off.
import { act, query } from "./core";

/** Why the VM is off (tauri/src/isolation): the notice says it and what
 * to do about it, by `code`. */
export type IsolationBlocker =
  | { code: "old_windows"; build: number }
  | { code: "old_macos"; version: string }
  | { code: "distribution"; name: string }
  | { code: "hypervisor_off" }
  | { code: "kvm_missing" }
  | { code: "kvm_denied" }
  | { code: "apparmor"; shim: string }
  | { code: "sbx_missing"; failed?: string }
  | { code: "sbx_signed_out" }
  | { code: "sbx_failed"; message: string }
  | { code: "foreign_daemon" }
  | { code: "kit_disk"; fs: string };

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
 * `sbx ls`; never a command that asks for an administrator (D-178). */
export type IsolationFix = "login" | "ls";

/** Opens `fix` with the `sbx` found in a new session of the app's terminal. */
export function isolationRun(fix: IsolationFix): Promise<void> {
  return act("isolation_run", { fix });
}

/** Whose sandbox it is, from the kit Atollo wrote for it (D-169); the kit's
 * terminals have none since D-178 (the list never shows theirs). */
export type VmOwner = { kind: "project"; project: string } | { kind: "tools" };

/** One of Atollo's sandboxes. */
export interface VmInfo {
  name: string;
  /** Null for an agent's sandbox or one made before the kits. */
  owner: VmOwner | null;
  /** `running`, `stopped`… as `sbx` says it. */
  status: string;
  lastUsed: string | null;
}

/** Atollo's sandboxes and the room the kit daemon's state takes. */
export interface VmList {
  vms: VmInfo[];
  /** Bytes on the disk (sparse VM disks count what is written); null for
   * the PC's `sbx`. */
  disk: number | null;
}

export function vmsList(): Promise<VmList> {
  return query<VmList>("vms_list", { vms: [], disk: null });
}

/** Removes a sandbox with its disk (home, `/tmp`, the VM's databases) and
 * its kit; the project's files stay. */
export function vmRemove(name: string): Promise<void> {
  return act("vm_remove", { name });
}

/** `force`: ask `sbx` again now, not the answer of the last minute. */
export function isolationStatus(force = false): Promise<IsolationStatus> {
  return query<IsolationStatus>("isolation_status", SAMPLE_ISOLATION, { force });
}
