// The kit's terminals (tauri/src/terminals.rs): the default one, every tool
// at its newest version, and the user's, with their own tools and versions,
// opened in any folder of the PC. Rust: tauri/src/commands/terminals.rs.

import { act, engineText, query } from "./core";

export type TerminalToolGroup = "agents" | "ai" | "languages" | "databases";

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
  /** Lines it keeps that are not on disk (any more): the newest is used. */
  replaced: LineSwap[];
}

/** A line of a terminal's tool not on disk, and the one used in its place. */
export interface LineSwap {
  id: string;
  saved: string;
  used: string;
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
    versions: { "8.4": "8.4.12", "7.4": "7.4.33" },
  },
  {
    id: "mysql",
    name: "MySQL",
    group: "databases",
    lines: ["9.7", "8.4"],
    versions: { "9.7": "9.7.2", "8.4": "8.4.11" },
  },
];

const SAMPLE_TERMINALS: TerminalsView = {
  terminals: [
    {
      id: "default",
      name: engineText("Tutti gli strumenti", { en: "All tools", fr: "Tous les outils", de: "Alle Werkzeuge" }),
      dir: null,
      tools: Object.fromEntries(SAMPLE_TOOLS.map((x) => [x.id, null])),
      builtin: true,
      missing: [],
      replaced: [],
    },
    {
      id: "legacy-php",
      name: "PHP 7.4 legacy",
      dir: "D:\\Siti\\gestionale",
      tools: { git: null, php: "7.4", mysql: null },
      builtin: false,
      missing: [],
      replaced: [],
    },
  ],
  tools: SAMPLE_TOOLS,
  projects: "E:\\Atollo\\projects",
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
