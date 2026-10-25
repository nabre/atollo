// The data of the Tools page: its sections, the choice that follows the
// dependencies, the lines of a versioned tool, sizes as text.
import type { ComponentInfo, Group, VersionedTool } from "../ipc";
import { t } from "../i18n";

/** The kinds of tool, as the site's matrix (site/matrix.mjs, KINDS and
 * KIND_OF: a test keeps them equal). The catalog's groups are coarser:
 * "languages" holds almost everything. */
export const KINDS = ["system", "agents", "ai", "languages", "tools", "web", "databases", "dbviewers"] as const;
export type Kind = (typeof KINDS)[number];

/** A component's kind when it is not its group's. */
export const KIND_OF: Record<string, Kind> = {
  pwsh: "system",
  gh: "system",
  git: "system",
  "vscodium-server": "system",
  uv: "tools",
  pnpm: "tools",
  cmake: "tools",
  android: "tools",
  typst: "tools",
  tinytex: "tools",
  apache: "web",
  caddy: "web",
  phpmyadmin: "dbviewers",
  "mongo-express": "dbviewers",
};
export const KIND_OF_GROUP: Record<Group, Kind> = {
  required: "system",
  app: "system",
  agents: "agents",
  ai: "ai",
  languages: "languages",
  databases: "databases",
};

export function kindOf(c: { id: string; group: Group }): Kind {
  return KIND_OF[c.id] ?? KIND_OF_GROUP[c.group];
}

/** The sections of the Tools page, the first-start page and the matrix:
 * the site's kinds, each with its title and what to know about it. */
export function groups(): { id: Kind; title: string; hint: string; has: (c: ComponentInfo) => boolean }[] {
  const text: Record<Kind, [string, string]> = {
    system: [
      t("Sistema ed editor"),
      t(
        "Sempre installati: l'editor, la shell del kit e GitHub CLI. Git arriva con Claude Code e Pi, o si sceglie qui.",
      ),
    ],
    agents: [t("Agenti AI"), t("Facoltativi: il kit funziona anche senza. Sempre all'ultima versione.")],
    ai: [
      t("Modelli locali"),
      t(
        "Il server dei modelli linguistici sul PC (llama.cpp): li usano Pi e la chat nel browser, senza rete. I modelli si scaricano dalla pagina AI.",
      ),
    ],
    languages: [
      t("Linguaggi"),
      t(
        "Linguaggi e runtime. Ognuno tiene più versioni una accanto all'altra, una per versione minore, tranne Rust e C/C++ che restano all'ultima: il kit usa la più recente, ogni progetto la sua.",
      ),
    ],
    tools: [
      t("Strumenti di sviluppo"),
      t(
        "Gestori di pacchetti, build e documenti, sempre all'ultima versione. Quelli che servono a un agente o a un progetto si installano da soli.",
      ),
    ],
    web: [t("Server web"), t("Apache e Caddy, per i siti dei progetti e i Server Web condivisi.")],
    databases: [
      t("Database"),
      t("Server avviati dalla pagina Database, solo su questo PC (127.0.0.1); i dati restano in data\\db."),
    ],
    dbviewers: [
      t("Visualizzazione database"),
      t(
        "I database nel browser: phpMyAdmin per MySQL, mongo-express per MongoDB. Si aprono dalla pagina Database, che li installa al primo uso.",
      ),
    ],
  };
  return KINDS.map((id) => ({
    id,
    title: text[id][0],
    hint: text[id][1],
    has: (c: ComponentInfo) => kindOf(c) === id,
  }));
}

/** `c` has every word of `search` in its name, description or id. */
export function matches(c: ComponentInfo, search: string): boolean {
  const text = `${c.name} ${c.desc} ${c.id}`.toLowerCase();
  return search
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((w) => text.includes(w));
}

/** Turns `id` on or off following the dependencies both ways. */
export function toggle(list: ComponentInfo[], chosen: Set<string>, id: string, on: boolean): Set<string> {
  const next = new Set(chosen);
  const visit = (cid: string, value: boolean) => {
    const c = list.find((x) => x.id === cid);
    if (!c || c.locked) return;
    if (value === next.has(cid)) return;
    if (value) {
      next.add(cid);
      c.needs.forEach((n) => {
        visit(n, true);
      });
    } else {
      next.delete(cid);
      list
        .filter((x) => x.needs.includes(cid))
        .forEach((x) => {
          visit(x.id, false);
        });
    }
  };
  visit(id, on);
  return next;
}

/** C and C++ on a Mac without Xcode Command Line Tools (`xcode-select -p`
 * fails): C and C++ work, the programs that use the Apple frameworks do not.
 * Only a Mac can tell, so never for a kit inspected from another system. */
export function xcodeMissing(c: ComponentInfo): boolean {
  return c.xcodeMissing === true;
}

/** The versions on disk of a versioned tool: the default, then the saved. */
export function linesOf(tool: VersionedTool | undefined): string[] {
  if (!tool) return [];
  return [...(tool.default ? [tool.default.line] : []), ...tool.saved.map((s) => s.line)];
}

export { sizeText } from "../shared/format";
