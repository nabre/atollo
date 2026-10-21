// Databases of a project (tauri/src/db/auth/, D-74), their backups, and
// what is inside a running server, read only (db/browse.rs, D-79).
// Rust: tauri/src/commands/databases.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, query } from "./core";
import { SAMPLE_SERVICES, type ServiceInfo } from "./services";

export interface DbTable {
  name: string;
  /** Estimated; null for views or when the server does not say. */
  rows: number | null;
  view: boolean;
}

export interface DbDatabase {
  name: string;
  /** The server's own (mysql, information_schema, admin, local…). */
  system: boolean;
  tables: DbTable[];
}

export interface DbRows {
  /** kind: MySQL's column type, empty for MongoDB. */
  columns: { name: string; kind: string }[];
  /** JSON values; binary data as {"$bytes": n}, MongoDB's as Extended JSON. */
  rows: unknown[][];
  total: number;
  offset: number;
}

/** Rows of one page (db_browse::PAGE). */
export const DB_PAGE = 100;

const SAMPLE_TREE: DbDatabase[] = [
  {
    name: "api_python",
    system: false,
    tables: [
      { name: "orders", rows: 1240, view: false },
      { name: "products", rows: 87, view: false },
      { name: "users", rows: 3, view: false },
      { name: "top_products", rows: null, view: true },
    ],
  },
  { name: "sito_vetrina", system: false, tables: [] },
  { name: "information_schema", system: true, tables: [{ name: "TABLES", rows: null, view: true }] },
  { name: "mysql", system: true, tables: [{ name: "user", rows: 5, view: false }] },
];

function sampleRows(table: string, offset: number): DbRows {
  const users = [
    [1, "ada", "ada@example.com", "2026-10-01 09:12:00", null, { $bytes: 2048 }],
    [2, "linus", "linus@example.com", "2026-10-03 18:40:12", "admin", null],
    [3, "grace", "grace@example.com", "2026-10-14 07:05:55", "editor", { $bytes: 512 }],
  ];
  if (table === "users")
    return {
      columns: [
        { name: "id", kind: "int(11)" },
        { name: "name", kind: "varchar(64)" },
        { name: "email", kind: "varchar(255)" },
        { name: "created_at", kind: "datetime" },
        { name: "role", kind: "varchar(16)" },
        { name: "avatar", kind: "blob" },
      ],
      rows: users,
      total: 3,
      offset: 0,
    };
  const total = table === "orders" ? 1240 : 87;
  const rows = Array.from({ length: Math.max(0, Math.min(DB_PAGE, total - offset)) }, (_, i) => [
    offset + i + 1,
    1 + ((offset + i) % 3),
    Math.round((offset + i) * 7.3 * 100) / 100,
    "2026-10-14",
  ]);
  return {
    columns: [
      { name: "id", kind: "int(11)" },
      { name: "user_id", kind: "int(11)" },
      { name: "total", kind: "decimal(10,2)" },
      { name: "day", kind: "date" },
    ],
    rows,
    total,
    offset,
  };
}

/** `line`: another version of the server, on its own port (D-102);
 * null: the version the server runs. */
export function dbBrowseTree(engine: ServiceInfo["id"], line: string | null = null): Promise<DbDatabase[]> {
  return query<DbDatabase[]>("db_browse_tree", SAMPLE_TREE, { engine, line });
}

export function dbBrowseRows(
  engine: ServiceInfo["id"],
  database: string,
  table: string,
  offset: number,
  line: string | null = null,
): Promise<DbRows> {
  return browserPreview
    ? Promise.resolve(sampleRows(table, offset))
    : invoke<DbRows>("db_browse_rows", { engine, line, database, table, offset });
}

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
