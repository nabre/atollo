// The long work of the app as the interface follows it (LEVELS.md §8.6,
// tauri/src/common/jobs.rs): installs and updates of the setup task, each
// with its place in the queue, and the preparation of a project's VM. Rust:
// tauri/src/ipc/setup.rs (`jobs_list`, `job_cancel`), the event `job`.
import { query, subscribe } from "./core";
import { JOB } from "./events";

export type JobKind = "install" | "update" | "vm-prepare" | "vm-check";

export type JobState = "queued" | "running" | "done" | "error" | "cancelled";

/** One step of a job: `index` of `total`, from 1. */
export interface JobStep {
  index: number;
  total: number;
  label: string;
  state: JobState;
  error: string | null;
}

export interface Job {
  id: string;
  kind: JobKind;
  /** «Installo Node.js», «Preparazione VM · shop», already translated. */
  title: string;
  /** The project of a VM's preparation. */
  project: string | null;
  /** The components a setup job touches; empty: the whole choice. */
  items: string[];
  /** The step running, or the last one. */
  step: JobStep | null;
  steps: JobStep[];
  /** The bytes of the download running, `total` when known. */
  bytes: { done: number; total: number | null } | null;
  /** The installer's last note, or the tool a VM's preparation downloads. */
  note: string | null;
  state: JobState;
  error: string | null;
  /** When it was asked (Unix ms). */
  created: number;
  /** Started by the app, not by a click. */
  automatic: boolean;
}

/** The jobs kept: those running and queued, and the last ones that ended. */
export function jobsList(): Promise<Job[]> {
  return query<Job[]>("jobs_list", []);
}

/** Cancels one job: a queued install leaves the queue, the running one
 * stops; false when it cannot be cancelled. */
export function jobCancel(id: string): Promise<boolean> {
  return query<boolean>("job_cancel", false, { id });
}

/** Every change of a job. */
export function onJob(handler: (job: Job) => void): Promise<() => void> {
  return subscribe<Job>(JOB, handler);
}
