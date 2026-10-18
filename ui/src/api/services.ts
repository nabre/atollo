// Database servers of the kit (tauri/src/db/services/, D-73).
// Rust: tauri/src/commands/services.rs.

import { query } from "./core";

export type ServiceState = "stopped" | "starting" | "running" | "error";

export interface ServiceInfo {
  id: "mysql" | "mongodb";
  name: string;
  installed: boolean;
  state: ServiceState;
  port: number;
  autostart: boolean;
  dataDir: string;
  log: string;
  url: string;
  user: string;
  password: string | null;
  message: string | null;
  /** The version it runs, and those installed, newest first (D-81). */
  line: string | null;
  lines: string[];
  /** The other installed versions, each on its own port, newest first
   * (D-102). */
  others: LineInfo[];
}

/** Another version of a server, started beside the one it runs. */
export interface LineInfo {
  line: string;
  state: ServiceState;
  port: number;
  dataDir: string;
  log: string;
  url: string;
  message: string | null;
}

export const SAMPLE_SERVICES: ServiceInfo[] = [
  {
    id: "mysql",
    name: "MySQL",
    installed: true,
    state: "running",
    port: 3306,
    autostart: true,
    dataDir: "E:\\Dev-Kit\\data\\db\\mysql\\9.7",
    log: "E:\\Dev-Kit\\data\\app\\logs\\mysql.log",
    url: "mysql://root@127.0.0.1:3306/",
    user: "root",
    password: "3f9c2a71d8e04b6a9c51e2d7f0a8b346",
    message: null,
    line: "9.7",
    lines: ["9.7", "8.4"],
    others: [
      {
        line: "8.4",
        state: "stopped",
        port: 3307,
        dataDir: "E:\\Dev-Kit\\data\\db\\mysql\\8.4",
        log: "E:\\Dev-Kit\\data\\app\\logs\\mysql-8.4.log",
        url: "mysql://root@127.0.0.1:3307/",
        message: null,
      },
    ],
  },
  {
    id: "mongodb",
    name: "MongoDB",
    installed: false,
    state: "stopped",
    port: 27017,
    autostart: false,
    dataDir: "E:\\Dev-Kit\\data\\db\\mongodb",
    log: "E:\\Dev-Kit\\data\\app\\logs\\mongodb.log",
    url: "mongodb://admin@127.0.0.1:27017/?authSource=admin",
    user: "admin",
    password: null,
    message: null,
    line: null,
    lines: [],
    others: [],
  },
];

export function servicesList(): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("services_list", SAMPLE_SERVICES);
}

/** Starts a server (installing it first if needed) and waits until it
 * answers; `line`: another installed version, on its own port. */
export function serviceStart(id: ServiceInfo["id"], line?: string): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_start", SAMPLE_SERVICES, line === undefined ? { id } : { id, line });
}

export function serviceStop(id: ServiceInfo["id"], line?: string): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_stop", SAMPLE_SERVICES, line === undefined ? { id } : { id, line });
}

/** The port of another version (null: the first free one after the
 * server's), from its next start. */
export function serviceLinePort(id: ServiceInfo["id"], line: string, port: number | null): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_line_port", SAMPLE_SERVICES, { id, line, port });
}

/** Port (null: the default) and "start with Dev-Kit"; the port applies at the next start. */
export function serviceSet(id: ServiceInfo["id"], port: number | null, autostart: boolean): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_set", SAMPLE_SERVICES, { id, port, autostart });
}

/** The version a server runs (null: the newest installed), while stopped;
 * each version keeps its own data, the first start copies the older one's. */
export function serviceLine(id: ServiceInfo["id"], line: string | null): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_line", SAMPLE_SERVICES, { id, line });
}
