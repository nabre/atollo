// Database servers of the kit (tauri/src/db/services/, D-73).
// Rust: tauri/src/commands/services.rs.

import { query } from "./core";

export type ServiceState = "stopped" | "starting" | "running" | "error";

export interface ServiceInfo {
  id: "mariadb" | "mongodb";
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
}

export const SAMPLE_SERVICES: ServiceInfo[] = [
  {
    id: "mariadb",
    name: "MariaDB",
    installed: true,
    state: "running",
    port: 3306,
    autostart: true,
    dataDir: "E:\\Atollo\\data\\db\\mariadb",
    log: "E:\\Atollo\\data\\app\\logs\\mariadb.log",
    url: "mysql://root@127.0.0.1:3306/",
    user: "root",
    password: "3f9c2a71d8e04b6a9c51e2d7f0a8b346",
    message: null,
    line: "12.3",
    lines: ["12.3", "11.4"],
  },
  {
    id: "mongodb",
    name: "MongoDB",
    installed: false,
    state: "stopped",
    port: 27017,
    autostart: false,
    dataDir: "E:\\Atollo\\data\\db\\mongodb",
    log: "E:\\Atollo\\data\\app\\logs\\mongodb.log",
    url: "mongodb://admin@127.0.0.1:27017/?authSource=admin",
    user: "admin",
    password: null,
    message: null,
    line: null,
    lines: [],
  },
];

export function servicesList(): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("services_list", SAMPLE_SERVICES);
}

/** Starts a server (installing it first if needed) and waits until it answers. */
export function serviceStart(id: ServiceInfo["id"]): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_start", SAMPLE_SERVICES, { id });
}

export function serviceStop(id: ServiceInfo["id"]): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_stop", SAMPLE_SERVICES, { id });
}

/** Port (null: the default) and "start with Atollo"; the port applies at the next start. */
export function serviceSet(id: ServiceInfo["id"], port: number | null, autostart: boolean): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_set", SAMPLE_SERVICES, { id, port, autostart });
}

/** The version a server runs (null: the newest installed), while stopped;
 * each version keeps its own data, the first start copies the older one's. */
export function serviceLine(id: ServiceInfo["id"], line: string | null): Promise<ServiceInfo[]> {
  return query<ServiceInfo[]>("service_line", SAMPLE_SERVICES, { id, line });
}
