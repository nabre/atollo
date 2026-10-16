// The kit's terminals (tauri/src/terminals.rs): the default one, every tool
// at its newest version, and the user's, with their own tools and versions,
// opened in any folder of the PC. Rust: tauri/src/commands/terminals.rs.

import { act, query } from "./core";

export type TerminalToolGroup = "agents" | "languages" | "databases";

export interface Terminal {
  /** Empty for a new one. */
  id: string;
  name: string;
  /** The folder it starts in; null: projects\. */
  dir: string | null;
  /** The tools it includes, each at its line (null: the newest on disk). */
  tools: Record<string, string | null>;
}

export interface TerminalInfo extends Terminal {
  /** The default terminal: neither changed nor removed. */
  builtin: boolean;
  /** Tools it includes that are not on disk (any more): left out. */
  missing: string[];
}

/** A tool a terminal can include: installed, with its lines on disk. */
export interface TerminalTool {
  id: string;
  name: string;
  group: TerminalToolGroup;
  /** Newest first; empty for a tool with one version. */
  lines: string[];
  /** The version of each line, when known. */
  versions: Record<string, string>;
}

export interface TerminalsView {
  terminals: TerminalInfo[];
  tools: TerminalTool[];
  /** Where a terminal without a folder starts. */
  projects: string;
}

const SAMPLE_TOOLS: TerminalTool[] = [
  { id: "claude", name: "Claude Code", group: "agents", lines: [], versions: {} },
  { id: "git", name: "Git", group: "languages", lines: [], versions: {} },
  {
    id: "python",
    name: "Python",
    group: "languages",
    lines: ["3.14", "3.12"],
    versions: { "3.14": "3.14.0", "3.12": "3.12.10" },
  },
  {
    id: "node",
    name: "Node.js",
    group: "languages",
    lines: ["24.11", "22.21", "20.19"],
    versions: { "24.11": "v24.11.0", "22.21": "v22.21.1", "20.19": "v20.19.5" },
  },
  {
    id: "php",
    name: "PHP",
    group: "languages",
    lines: ["8.4", "7.4"],
    versions: { "8.4": "8.4.13", "7.4": "7.4.33" },
  },
  { id: "mariadb", name: "MariaDB", group: "databases", lines: ["12.3"], versions: { "12.3": "12.3.1" } },
];

const SAMPLE_TERMINALS: TerminalsView = {
  terminals: [
    {
      id: "default",
      name: "Tutti gli strumenti",
      dir: null,
      tools: Object.fromEntries(SAMPLE_TOOLS.map((x) => [x.id, null])),
      builtin: true,
      missing: [],
    },
    {
      id: "legacy-php",
      name: "PHP 7.4 legacy",
      dir: "D:\\Siti\\gestionale",
      tools: { git: null, php: "7.4", mariadb: null },
      builtin: false,
      missing: [],
    },
  ],
  tools: SAMPLE_TOOLS,
  projects: "E:\\Dev-Kit\\projects",
};

export function terminalsList(): Promise<TerminalsView> {
  return query<TerminalsView>("terminals_list", SAMPLE_TERMINALS);
}

/** Saves a new terminal (empty id) or the changes of one; resolves to its id. */
export function terminalSave(terminal: Terminal): Promise<string> {
  return query<string>("terminal_save", terminal.id || "new", { terminal });
}

export function terminalRemove(id: string): Promise<void> {
  return act("terminal_remove", { id });
}

/** Opens the terminal in `dir` (null: its own folder). */
export function terminalLaunch(id: string, dir: string | null = null): Promise<void> {
  return act("terminal_launch", { id, dir });
}

/** Windows' "Select folder" window, starting in `start`; null when closed
 * without a choice. */
export function terminalDirPick(start: string | null): Promise<string | null> {
  return query<string | null>("terminal_dir_pick", null, { start });
}
