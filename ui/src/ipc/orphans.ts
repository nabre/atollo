// Data of the kit left by projects that are gone from projects\ (renamed or
// deleted outside the app, D-89). Rust: tauri/src/ipc/orphans.rs.

import { act, query } from "./core";

/** What the kit keeps for a project name (never a password). */
export interface Holdings {
  project: string;
  /** `dev-settings.json` is there. */
  settings: boolean;
  /** Ports remembered for its actions. */
  ports: number;
  /** Engines with a user of the project (`mysql`, `mongodb`). */
  databases: string[];
  /** Copies of its databases. */
  backups: number;
}

export function projectsOrphans(): Promise<Holdings[]> {
  return query<Holdings[]>("projects_orphans", []);
}

/** Gives the data of `orphan` to `project`; refused when `project` already
 * has data of the kit. */
export function projectOrphanAdopt(orphan: string, project: string): Promise<void> {
  return act("project_orphan_adopt", { orphan, project });
}
