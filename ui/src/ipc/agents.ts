// Claude Code, Codex, Antigravity and Pi: state, plan limits and usage
// (tauri/src/stats), launch. Rust: tauri/src/ipc/agents.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, engineText, query } from "./core";
import type { McpServer, McpView } from "./projects";

export type AgentId = "claude" | "codex" | "antigravity" | "pi";

export interface AgentInfo {
  id: AgentId;
  name: string;
  /** It can start in the VM: Claude Code and Codex always (the `sbx`
   * images bring them), Pi and Antigravity once their build for the VM's
   * Linux is in the kit. */
  installed: boolean;
  /** Claude Code, Codex: brought by the VM, nothing of the kit's. */
  provided: boolean;
  /** The version of its build for the VM's Linux (Pi, Antigravity). */
  version: string | null;
  /** Signed in for the VM, as far as the PC can tell (LEVELS.md §13 item
   * 12): otherwise the agent signs in inside the VM at its first start. */
  signedIn: boolean;
  /** "api key" (Antigravity's Gemini key through `sbx`'s proxy) or "local"
   * (Pi on the kit's local models). */
  signInKind: string | null;
  /** A sign-in of the PC an earlier Atollo saved in the kit's data, never
   * given to the agents in the VM: «Esci» removes it. */
  kitLogin: boolean;
  /** It can start now: the VM is on (LEVELS.md §7). */
  available: boolean;
  /** Why not: the VM's blockers and what to do. */
  unavailableReason: string | null;
  runsIn: "sbxSandbox" | "projectVm";
}

const sample = (a: Partial<AgentInfo> & Pick<AgentInfo, "id" | "name">): AgentInfo => ({
  installed: true,
  provided: false,
  version: null,
  signedIn: false,
  signInKind: null,
  kitLogin: false,
  available: true,
  unavailableReason: null,
  runsIn: "projectVm",
  ...a,
});

export const SAMPLE_AGENTS: AgentInfo[] = [
  sample({ id: "claude", name: "Claude Code", provided: true, runsIn: "sbxSandbox" }),
  sample({ id: "codex", name: "Codex", provided: true, runsIn: "sbxSandbox" }),
  sample({ id: "antigravity", name: "Antigravity", installed: false }),
  // Installed on a local model, so the screenshots show the fourth agent
  // (DOC-V7-05).
  sample({ id: "pi", name: "Pi", version: "0.70.2", signedIn: true, signInKind: "local" }),
];

export function agentsStatus(): Promise<AgentInfo[]> {
  return query<AgentInfo[]>("agents_status", SAMPLE_AGENTS);
}

export type Level = "none" | "ok" | "warning" | "critical";

export interface LimitWindow {
  label: string;
  /** % used, 0-100. */
  percent: number | null;
  /** Milliseconds since 1970. */
  resetsAt: number | null;
  level: Level;
}

export interface Limits {
  plan: string | null;
  windows: LimitWindow[];
  credits: { used: number; limit: number | null; currency: string; percent: number | null; level: Level } | null;
  source: string | null;
  note: string | null;
  error: string | null;
  updated: number | null;
  limited: boolean;
  retryAt: number | null;
}

export interface Tokens {
  input: number;
  cached: number;
  output: number;
  total: number;
}

export interface Usage {
  /** False when the agent saves no token counts (Antigravity). */
  tokens: boolean;
  since: number;
  today: Tokens;
  week: Tokens;
  sessionsToday: number;
  sessionsWeek: number;
  last: number | null;
  models: Record<string, number>;
  days: (Tokens & { start: number; sessions: number })[];
  error?: string;
}

export interface AgentStats {
  id: AgentId;
  name: string;
  account: "none" | "expired" | "apikey" | "ok" | "local";
  limits: Limits;
  usage: Usage;
  peak: number | null;
  peakLabel: string | null;
  level: Level;
}

export interface StatsReport {
  updated: number;
  warning: number;
  critical: number;
  agents: AgentStats[];
}

function sampleStats(): StatsReport {
  const day = 24 * 3600 * 1000;
  const fiveHours = engineText("5 ore", { en: "5 hours", fr: "5 heures", de: "5 Stunden" });
  const week = engineText("Settimana", { en: "Week", fr: "Semaine", de: "Woche" });
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const tokens = (n: number): Tokens => ({ input: n, cached: n * 8, output: n / 4, total: n * 9.25 });
  const days = Array.from({ length: 7 }, (_, i) => ({
    start: today.getTime() - (6 - i) * day,
    ...tokens((i + 1) * 20000),
    sessions: i % 3,
  }));
  const usage: Usage = {
    tokens: true,
    since: days[0]?.start ?? 0,
    today: tokens(140000),
    week: tokens(560000),
    sessionsToday: 2,
    sessionsWeek: 6,
    last: Date.now() - 600000,
    models: { "claude-opus": 4000000, "claude-haiku": 1180000 },
    days,
  };
  const limits = (windows: LimitWindow[]): Limits => ({
    plan: "max",
    windows,
    credits: null,
    source: engineText("server di Anthropic", {
      en: "Anthropic's server",
      fr: "serveur d'Anthropic",
      de: "Server von Anthropic",
    }),
    note: null,
    error: null,
    updated: Date.now(),
    limited: false,
    retryAt: null,
  });
  return {
    updated: Date.now(),
    warning: 70,
    critical: 90,
    agents: [
      {
        id: "claude",
        name: "Claude Code",
        account: "ok",
        limits: limits([
          { label: fiveHours, percent: 34, resetsAt: Date.now() + 2 * 3600000, level: "ok" },
          { label: week, percent: 76, resetsAt: Date.now() + 3 * day, level: "warning" },
        ]),
        usage,
        peak: 76,
        peakLabel: week,
        level: "warning",
      },
    ],
  };
}

/** Limits and usage of the installed agents; `force`: the Refresh button. */
export function agentsStats(force: boolean): Promise<StatsReport> {
  return browserPreview ? Promise.resolve(sampleStats()) : invoke<StatsReport>("agents_stats", { force });
}

/** Opens the agent on the project, in its VM (D-178, D-179): Claude Code
 * and Codex in their sandboxes, Pi and Antigravity in the project's; never
 * on the PC; without the VM the error says why. */
export function agentLaunch(agent: AgentId, project: string): Promise<void> {
  return act("agent_launch", { agent, project });
}

/** A ready MCP server (N-7, tauri/src/agents/presets.rs). */
export interface McpPreset {
  id: "github" | "playwright" | "fetch" | "filesystem" | "mysql" | "mongodb";
  /** `kit`: data\agents\mcp.json; `project`: its .mcp.json. */
  scope: "kit" | "project";
  /** The project's database it reads: offered only when the project has one. */
  engine: "mysql" | "mongodb" | null;
  server: McpServer;
}

const npx = (name: string, args: string[], env: Record<string, string> = {}): McpServer => ({
  name,
  kind: "stdio",
  command: "npx",
  args,
  url: null,
  env,
  headers: {},
});

export const SAMPLE_PRESETS: McpPreset[] = [
  {
    id: "github",
    scope: "kit",
    engine: null,
    // The app by its absolute path, which reads the token itself.
    server: {
      name: "github",
      kind: "stdio",
      command: "E:\\Atollo\\Atollo.exe",
      args: ["mcp", "github"],
      url: null,
      env: {},
      headers: {},
    },
  },
  { id: "playwright", scope: "kit", engine: null, server: npx("playwright", ["-y", "@playwright/mcp@latest"]) },
  {
    id: "fetch",
    scope: "kit",
    engine: null,
    server: { ...npx("fetch", ["mcp-server-fetch"]), command: "uvx" },
  },
  {
    id: "filesystem",
    scope: "project",
    engine: null,
    server: npx("filesystem", ["-y", "@modelcontextprotocol/server-filesystem", "."]),
  },
  {
    id: "mysql",
    scope: "project",
    engine: "mysql",
    server: npx("mysql", ["-y", "@bytebase/dbhub", "--transport", "stdio"], { DSN: "${DSN}" }),
  },
  {
    id: "mongodb",
    scope: "project",
    engine: "mongodb",
    server: npx("mongodb", ["-y", "mongodb-mcp-server@latest"], {
      MDB_MCP_CONNECTION_STRING: "${MDB_MCP_CONNECTION_STRING}",
    }),
  },
];

export function mcpPresets(): Promise<McpPreset[]> {
  return query<McpPreset[]>("mcp_presets", SAMPLE_PRESETS);
}

/** The kit's MCP servers (data\agents\mcp.json), every agent's. */
export function kitMcp(): Promise<McpView> {
  return query<McpView>("kit_mcp", {
    servers: SAMPLE_PRESETS.filter((p) => p.id === "playwright").map((p) => p.server),
    readOnly: false,
  });
}

/** Writes the kit's MCP servers and aligns the agents. */
export function kitMcpSet(servers: McpServer[]): Promise<void> {
  return act("kit_mcp_set", { servers });
}

/** «Esci»: the sign-in of the PC an earlier Atollo saved in the kit's data,
 * removed (files only). */
export function agentSignOut(agent: AgentId): Promise<void> {
  return act("agent_sign_out", { agent });
}
