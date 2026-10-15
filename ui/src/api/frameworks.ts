// Frameworks of the new-project wizard (frameworks.rs, D-83).
// Rust: src-tauri/src/commands/frameworks.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, browserPreview, query, subscribe } from "./core";

export interface FrameworkAction {
  name: string;
  command: string;
}

/** A line of the project's .env; {name} and {package} are filled in. */
export interface EnvVar {
  key: string;
  value: string;
}

/** What an addition does with one base. */
export interface FrameworkVariant {
  needs?: string[];
  /** Replaces the base's creation (a starter kit, another template). */
  create?: string[];
  steps?: string[];
  actions?: FrameworkAction[];
  /** A database server of the kit (mariadb, mongodb): the project gets its database there. */
  database?: string;
  env?: EnvVar[];
  note?: string;
}

export interface FrameworkPiece {
  id: string;
  kind: "base" | "addon";
  name: string;
  /** Italian: shown through FrameworksView.texts. */
  desc: string;
  home: string;
  /** Additions of the same slot do not go together. */
  slot?: string;
  needs?: string[];
  /** Run in projects\: makes the folder {name}. */
  create?: string[];
  /** Run inside the project. */
  steps?: string[];
  actions?: FrameworkAction[];
  env?: EnvVar[];
  note?: string;
  /** Of an addition: what it does with each base, by base id. */
  on?: Record<string, FrameworkVariant>;
}

export interface FrameworksView {
  pieces: FrameworkPiece[];
  official: FrameworkPiece[];
  /** Descriptions and notes in the app's language, by Italian source. */
  texts: Record<string, string>;
  problem: string | null;
}

export interface FrameworkPlan {
  needs: string[];
  create: string[];
  steps: string[];
  actions: FrameworkAction[];
  /** The kit's database server where the project gets its database. */
  database: string | null;
  /** Variables of .env, {name} and {package} not filled in yet. */
  env: EnvVar[];
  /** Italian sources (FrameworksView.texts). */
  notes: string[];
}

/** A folder of projects\ for the wizard's name: an empty one or one with
 * files but no Git can be used; a Git repository cannot. */
export interface FrameworkTarget {
  name: string;
  state: "empty" | "files" | "git";
}

/** Progress of a creation: a command starting, or a line of its output. */
export interface FrameworkEvent {
  /** 1-based number of the command; 0 = installing the kit's tools. */
  step: number;
  total: number;
  command: string | null;
  line: string | null;
}

const SAMPLE_PIECES: FrameworkPiece[] = [
  {
    id: "laravel",
    kind: "base",
    name: "Laravel",
    desc: "Framework PHP per applicazioni web",
    home: "https://laravel.com/docs/13.x/installation",
    needs: ["php", "node"],
    create: ["composer create-project laravel/laravel:^13.0 {name} --prefer-dist"],
    steps: ["npm install"],
    actions: [{ name: "Avvia", command: "composer run dev" }],
  },
  {
    id: "vite",
    kind: "base",
    name: "Vite",
    desc: "Sito o applicazione web in TypeScript",
    home: "https://vite.dev/guide/",
    needs: ["node"],
    create: ["npm create vite@9 {name} -- --template vanilla-ts --no-interactive"],
    steps: ["npm install"],
    actions: [{ name: "Avvia", command: "npm run dev" }],
  },
  {
    id: "react",
    kind: "addon",
    slot: "frontend",
    name: "React",
    desc: "Interfaccia in React con TypeScript",
    home: "https://react.dev/learn/creating-a-react-app",
    on: {
      laravel: { create: ["composer create-project laravel/react-starter-kit {name} --prefer-dist --stability=dev"] },
      vite: { create: ["npm create vite@9 {name} -- --template react-ts --no-interactive"] },
    },
  },
  {
    id: "tailwind",
    kind: "addon",
    slot: "css",
    name: "Tailwind CSS",
    desc: "Stili con le classi di utilità di Tailwind CSS 4",
    home: "https://tailwindcss.com/docs/installation/using-vite",
    on: {
      laravel: { note: "Già incluso in Laravel e nei suoi starter kit." },
      vite: { steps: ["npm install -D tailwindcss@4 @tailwindcss/vite@4"] },
    },
  },
];

export function frameworksGet(): Promise<FrameworksView> {
  return query<FrameworksView>("frameworks_get", {
    pieces: SAMPLE_PIECES,
    official: SAMPLE_PIECES,
    texts: {},
    problem: null,
  });
}

/** Saves the instructions; those equal to the official ones are not kept. */
export function frameworksSet(pieces: FrameworkPiece[]): Promise<void> {
  return act("frameworks_set", { pieces });
}

/** What a base with its additions would run, with `pieces` as edited. */
export function frameworksPlan(pieces: FrameworkPiece[], base: string, addons: string[]): Promise<FrameworkPlan> {
  if (browserPreview) {
    const b = pieces.find((p) => p.id === base);
    return Promise.resolve({
      needs: b?.needs ?? [],
      create: b?.create ?? [],
      steps: b?.steps ?? [],
      actions: b?.actions ?? [],
      database: null,
      env: b?.env ?? [],
      notes: [],
    });
  }
  return invoke<FrameworkPlan>("frameworks_plan", { pieces, base, addons });
}

/** Creates projects\<name> (or fills an existing one) with `env` in its .env
 * and returns its name; progress comes as `framework` events. */
export function frameworksCreate(base: string, addons: string[], name: string, env: EnvVar[]): Promise<string> {
  return query<string>("frameworks_create", name, { base, addons, name, env });
}

export function frameworksTargets(): Promise<FrameworkTarget[]> {
  return query<FrameworkTarget[]>("frameworks_targets", [
    { name: "vuoto", state: "empty" },
    { name: "appunti", state: "files" },
  ]);
}

export function frameworksCancel(): Promise<void> {
  return act("frameworks_cancel");
}

export function onFramework(handler: (e: FrameworkEvent) => void): Promise<() => void> {
  return subscribe<FrameworkEvent>("framework", handler);
}
