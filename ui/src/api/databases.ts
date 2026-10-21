// Databases of a project (tauri/src/db/auth/, D-74), their backups, and
// the web admins of the servers (phpMyAdmin, mongo-express: D-107, D-147).
// Rust: tauri/src/commands/databases.rs.

import { act, query } from "./core";
import { SAMPLE_SERVICES, type ServiceInfo } from "./services";

/** The kit's database in the browser: phpMyAdmin on MySQL (D-107, D-117),
 * mongo-express on MongoDB (D-147); installed the first time with what it
 * needs, the server started when it is off; `line`: another version, on
 * its own port (null: the version the server runs). */
export function dbAdminOpen(line: string | null = null, engine: "mysql" | "mongodb" = "mysql"): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("db_admin_open", SAMPLE_SERVICES, { line, engine });
}

export interface ProjectDb {
  /** `mysql` for every SQL server (D-117). */
  engine: "mysql" | "mongodb";
  database: string;
  user: string;
  password: string;
  host: string;
  port: number;
  url: string;
}

export interface ProjectDatabases {
  dbs: ProjectDb[];
  /** "mysql", "mongodb", "sqlite" the project seems to use. */
  detected: string[];
  envExists: boolean;
  envIgnored: boolean;
  /** By engine: the version its database is on (D-102). */
  versions: Partial<Record<ProjectDb["engine"], DbVersions>>;
}

export interface DbVersions {
  /** Chosen for the project; null: the one the server runs (`server`). */
  chosen: string | null;
  server: string | null;
  /** Installed, newest first. */
  lines: string[];
}

export interface EnvChange {
  key: string;
  old: string | null;
  new: string;
}

const SAMPLE_PROJECT_DATABASES: ProjectDatabases = {
  dbs: [
    {
      engine: "mysql",
      database: "negozio_online",
      user: "negozio_online",
      password: "5be1c0d29a7f4e8b93c6d2a1f0e47b58",
      host: "127.0.0.1",
      port: 3306,
      url: "mysql://negozio_online:5be1c0d29a7f4e8b93c6d2a1f0e47b58@127.0.0.1:3306/negozio_online",
    },
  ],
  detected: ["mysql", "sqlite"],
  envExists: true,
  envIgnored: true,
  versions: {
    mysql: { chosen: null, server: "9.7", lines: ["9.7", "8.4"] },
    mongodb: { chosen: null, server: null, lines: [] },
  },
};

export function projectDatabases(project: string): Promise<ProjectDatabases> {
  return query<ProjectDatabases>("project_databases", SAMPLE_PROJECT_DATABASES, { project });
}

/** The version of a server for the project's database (null: the one the
 * server runs), on its own port; a database already created is created on
 * it too, the server installed and started for it. */
export function projectDbLine(project: string, engine: ProjectDb["engine"], line: string | null): Promise<void> {
  return act("project_db_line", { project, engine, line });
}

/** Creates the project's database and user, installing and starting the server if needed. */
export function projectDbCreate(project: string, engine: ProjectDb["engine"]): Promise<ProjectDb> {
  return query<ProjectDb>("project_db_create", SAMPLE_PROJECT_DATABASES.dbs[0] as ProjectDb, { project, engine });
}

export function projectEnvPreview(project: string, engine: ProjectDb["engine"]): Promise<EnvChange[]> {
  return query<EnvChange[]>("project_env_preview", [{ key: "DB_DATABASE", old: "laravel", new: "negozio_online" }], {
    project,
    engine,
  });
}

export function projectEnvApply(project: string, engine: ProjectDb["engine"]): Promise<void> {
  return act("project_env_apply", { project, engine });
}

/** A copy of a project's database (data\projects\<project>\private\backups, D-78). */
export interface Backup {
  file: string;
  engine: "mysql" | "mongodb";
  size: number;
}

export function projectBackups(project: string): Promise<Backup[]> {
  return query<Backup[]>(
    "project_backups",
    [{ file: "negozio_online-20261014-171500.sql", engine: "mysql", size: 48211 }],
    { project },
  );
}

/** A copy of the project's database; returns the file name. */
export function projectDbExport(project: string, engine: Backup["engine"]): Promise<string> {
  return query<string>("project_db_export", "copia.sql", { project, engine });
}

export function projectDbRestore(project: string, file: string): Promise<void> {
  return act("project_db_restore", { project, file });
}
