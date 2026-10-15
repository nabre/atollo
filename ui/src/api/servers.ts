// Web Servers (tauri/src/servers/): shared Apache and Nginx VirtualHost pools,
// manual server profiles, and attached projects. Rust: tauri/src/commands/servers.rs.

import { act, query } from "./core";

export type WebServerEngine = "apache" | "nginx";

export interface AttachedSite {
  project: string;
  folder: string;
  serverName: string;
  domains: string[];
  aliases: Record<string, string>;
  url: string;
}

export interface WebServerPool {
  id: string;
  name: string;
  engine: WebServerEngine;
  serverLine: string | null;
  phpLine: string | null;
  port: number;
  running: boolean;
  pid: number | null;
  manual: boolean;
  sites: AttachedSite[];
  confDir: string;
}

export interface WebServerProfileInput {
  id: string;
  name: string;
  engine: WebServerEngine;
  serverLine: string | null;
  phpLine: string | null;
  port: number | null;
}

export interface WebServersView {
  pools: WebServerPool[];
  apacheLines: string[];
  nginxLines: string[];
  phpLines: string[];
}

const SAMPLE_SERVERS: WebServersView = {
  pools: [
    {
      id: "apache-2.4-php-8.4",
      name: "Apache 2.4 · PHP 8.4",
      engine: "apache",
      serverLine: "2.4",
      phpLine: "8.4",
      port: 8081,
      running: true,
      pid: 4120,
      manual: false,
      sites: [
        {
          project: "portale-legacy",
          folder: "public",
          serverName: "portale-legacy.localhost",
          domains: ["admin.portale-legacy.localhost"],
          aliases: { "/media": "storage/uploads" },
          url: "http://portale-legacy.localhost:8081",
        },
      ],
      confDir: "data\\servers\\apache-2.4-php-8.4",
    },
  ],
  apacheLines: ["2.4"],
  nginxLines: ["1.28"],
  phpLines: ["8.4", "8.3", "7.4", "5.6"],
};

export function serversList(): Promise<WebServersView> {
  return query<WebServersView>("servers_list", SAMPLE_SERVERS);
}

export function serverSave(profile: WebServerProfileInput): Promise<void> {
  return act("server_save", { profile });
}

export function serverRemove(id: string): Promise<void> {
  return act("server_remove", { id });
}

export function serverStart(id: string): Promise<void> {
  return act("server_start", { id });
}

export function serverStop(id: string): Promise<void> {
  return act("server_stop", { id });
}

export function serverReload(id: string): Promise<void> {
  return act("server_reload", { id });
}

export function serverAssignProject(
  project: string,
  folder: string,
  engine: WebServerEngine,
  serverId: string | null,
): Promise<void> {
  return act("server_assign_project", { project, folder, engine, serverId });
}
