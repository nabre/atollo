// Saved versions of the languages (D-80) and the relations matrix (D-81).
// Rust: src-tauri/src/commands/versions.rs.

import { act, query } from "./core";

/** A minor line (`22.11`) of a language, with its full version when known. */
export interface VersionLine {
  line: string;
  version: string | null;
}

/** A language or tool whose versions can be kept side by side (versions.rs). */
export interface VersionedTool {
  id: string;
  name: string;
  installed: boolean;
  /** The default folder, replaced by updates. */
  default: VersionLine | null;
  /** The other lines kept, newest first. */
  saved: VersionLine[];
  /** The kit's preferred line; null = the default folder. */
  preferred: string | null;
  /** Other lines can be downloaded. */
  downloadable: boolean;
  /** The lines its source offered the last time it was asked, newest first
   * (also those on disk). */
  available: string[];
  /** When the source was asked (Unix seconds); null = never. */
  checked: number | null;
  /** The projects that chose each line. */
  projects: Record<string, string[]>;
}

export interface ProjectVersions {
  tools: VersionedTool[];
  /** Lines chosen for the project, by component id; the others follow the kit. */
  chosen: Record<string, string>;
}

const SAMPLE_VERSIONS: VersionedTool[] = [
  {
    id: "node",
    name: "Node.js",
    installed: true,
    default: { line: "24.11", version: "v24.11.0" },
    saved: [
      { line: "22.21", version: "v22.21.1" },
      { line: "20.19", version: "v20.19.5" },
    ],
    preferred: null,
    downloadable: true,
    available: ["25.1", "24.11", "24.10", "22.21", "22.20", "20.19", "18.20"],
    checked: 1760500000,
    projects: { "20.19": ["shop"] },
  },
  {
    id: "php",
    name: "PHP",
    installed: true,
    default: { line: "8.4", version: "8.4.12 (Composer 2.8.12)" },
    saved: [{ line: "8.3", version: "8.3.26 (Composer 2.8.12)" }],
    preferred: null,
    downloadable: true,
    available: [],
    checked: null,
    projects: {},
  },
  {
    id: "python",
    name: "Python",
    installed: false,
    default: null,
    saved: [],
    preferred: null,
    downloadable: true,
    available: [],
    checked: null,
    projects: {},
  },
];

export function versionsList(): Promise<VersionedTool[]> {
  return query<VersionedTool[]>("versions_list", SAMPLE_VERSIONS);
}

/** The lines of `id` that can be downloaded, newest first (asks its source;
 * versionsList keeps them in `available`). */
export function versionsAvailable(id: string): Promise<string[]> {
  return query<string[]>(
    "versions_available",
    id === "php" ? ["8.5", "8.4", "8.3", "8.2", "8.1"] : ["24.11", "22.21", "20.19", "18.20"],
    { id },
  );
}

/** Downloads `line` next to the default folder; progress as `setup` events. */
export function versionsAdd(id: string, line: string): Promise<void> {
  return act("versions_add", { id, line });
}

export function versionsRemove(id: string, line: string): Promise<VersionedTool[]> {
  return query<VersionedTool[]>("versions_remove", SAMPLE_VERSIONS, { id, line });
}

export function projectVersions(project: string): Promise<ProjectVersions> {
  return query<ProjectVersions>("project_versions", { tools: SAMPLE_VERSIONS, chosen: { node: "20.19" } }, { project });
}

/** The line of `id` for the project; null = the kit's. One not on disk is
 * downloaded (progress as `setup` events). */
/** A relation of the matrix (relations.rs, D-81): `from` needs or uses
 * `to`, at the versions `range`; parts are `component/part`. */
export interface RelationLink {
  from: string;
  fromName: string;
  to: string;
  toName: string;
  kind: "needs" | "uses" | "runs-on" | "loads";
  optional: boolean;
  range: string | null;
  note: string | null;
  status: "inactive" | "ok" | "missing" | "optional" | "incompatible";
  /** The version of `to` the kit uses. */
  uses: string | null;
  /** Lines of `to` on disk outside `range`. */
  outside: string[];
}

/** What the PC must provide for some components. */
export interface RelationRuntime {
  id: string;
  name: string;
  note: string | null;
  neededBy: string[];
  present: boolean;
  detail: string | null;
}

/** A version a project declares, and what the kit can do for it. */
export interface RelationNeed {
  component: string;
  name: string;
  /** `package.json engines.node`. */
  source: string;
  declared: string;
  status: "ok" | "choose" | "download" | "unavailable" | "notInstalled" | "unknown";
  uses: string | null;
  /** For choose and download: the line that fits. */
  line: string | null;
  /** A database server (one for every project): choose sets the version
   * the server runs. */
  server: boolean;
}

export interface RelationsMatrix {
  fingerprint: string;
  generated: number;
  platform: string;
  links: RelationLink[];
  runtimes: RelationRuntime[];
  projects: { project: string; needs: RelationNeed[] }[];
}

const SAMPLE_RELATIONS: RelationsMatrix = {
  fingerprint: "sample",
  generated: 1760500000,
  platform: "win32-x64",
  links: [
    {
      from: "php/composer",
      fromName: "Composer",
      to: "php",
      toName: "PHP",
      kind: "runs-on",
      optional: false,
      range: ">=7.2.5",
      note: "Composer 2.3 e successivi; per PHP più vecchi il kit sceglie il 2.2 LTS",
      status: "ok",
      uses: "8.4.12 (Composer 2.8.12)",
      outside: [],
    },
    {
      from: "pnpm",
      fromName: "pnpm, yarn",
      to: "node",
      toName: "Node.js",
      kind: "runs-on",
      optional: false,
      range: ">=18.12",
      note: "pnpm e yarn usano il Node.js del kit",
      status: "inactive",
      uses: "v24.11.0",
      outside: [],
    },
    {
      from: "claude",
      fromName: "Claude Code",
      to: "git",
      toName: "Git",
      kind: "uses",
      optional: true,
      range: null,
      note: "Git Bash per lo strumento Bash; senza, Claude usa PowerShell",
      status: "ok",
      uses: "2.56.0",
      outside: [],
    },
  ],
  runtimes: [
    {
      id: "vcruntime",
      name: "Visual C++ 2015-2026 x64",
      note: "DLL copiate dal kit accanto ai programmi",
      neededBy: ["PHP"],
      present: true,
      detail: null,
    },
    {
      id: "webview2",
      name: "WebView2",
      note: "la finestra di NEXKORA",
      neededBy: ["NEXKORA"],
      present: true,
      detail: "141.0.3537.71",
    },
  ],
  projects: [
    {
      project: "shop",
      needs: [
        {
          component: "node",
          name: "Node.js",
          source: "package.json engines.node",
          declared: ">=22",
          status: "choose",
          uses: "20.19",
          line: "22.21",
          server: false,
        },
        {
          component: "php",
          name: "PHP",
          source: "composer.json require.php",
          declared: "^8.2",
          status: "ok",
          uses: "8.4",
          line: null,
          server: false,
        },
      ],
    },
  ],
};

/** The relations matrix: the saved one while nothing it is made from
 * changed; `force` builds it again. */
export function relationsMatrix(force = false): Promise<RelationsMatrix> {
  return query<RelationsMatrix>("relations_matrix", SAMPLE_RELATIONS, { force });
}

export function projectVersionsSet(project: string, id: string, line: string | null): Promise<ProjectVersions> {
  return query<ProjectVersions>(
    "project_versions_set",
    { tools: SAMPLE_VERSIONS, chosen: line ? { [id]: line } : {} },
    { project, id, line },
  );
}
