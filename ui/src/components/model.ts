// The data of the Tools page: its sections, the choice that follows the
// dependencies, the lines of a versioned tool, sizes as text.
import type { ComponentInfo, Group, VersionedTool } from "../api";
import { locale, t } from "../i18n";

/** The sections of the page. The catalog's "languages" are split: those
 * with one folder per minor version, side by side, and the tools that are
 * always at their latest version. */
export function groups(): { id: string; title: string; hint: string; has: (c: ComponentInfo) => boolean }[] {
  const of = (group: Group) => (c: ComponentInfo) => c.group === group;
  return [
    {
      id: "required",
      title: t("Obbligatori"),
      hint: t("Sempre installati: l'editor, la shell del kit e GitHub CLI."),
      // The editor (part of the app) is listed with the required ones.
      has: (c) => c.group === "required" || c.group === "app",
    },
    {
      id: "agents",
      title: t("Agenti"),
      hint: t("Facoltativi: il kit funziona anche senza. Sempre all'ultima versione."),
      has: of("agents"),
    },
    {
      id: "languages",
      title: t("Linguaggi"),
      hint: t(
        "Più versioni una accanto all'altra, una per versione minore: «Aggiorna tutto» porta ognuna all'ultima patch, una versione minore nuova si aggiunge senza togliere le altre. Il kit usa la più recente; un progetto tiene quella che c'era quando è stato creato (o quella che chiede il suo codice) e può cambiarla nella sua pagina.",
      ),
      has: (c) => c.group === "languages" && c.versioned,
    },
    {
      id: "tools",
      title: t("Strumenti"),
      hint: t(
        "Una sola versione, sempre l'ultima: «Aggiorna tutto» la sostituisce. Quelli che servono a un agente o a un progetto si installano da soli.",
      ),
      has: (c) => c.group === "languages" && !c.versioned,
    },
    {
      id: "databases",
      title: t("Database"),
      hint: t("Server avviati dalla pagina Database, solo su questo PC (127.0.0.1); i dati restano in data\\db."),
      has: of("databases"),
    },
  ];
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
  return (c as ComponentInfo & { xcodeMissing?: boolean }).xcodeMissing === true;
}

/** The versions on disk of a versioned tool: the default, then the saved. */
export function linesOf(tool: VersionedTool | undefined): string[] {
  if (!tool) return [];
  return [...(tool.default ? [tool.default.line] : []), ...tool.saved.map((s) => s.line)];
}

export function sizeText(mb: number): string {
  if (mb < 1000) return `${String(mb)} MB`;
  const gb = new Intl.NumberFormat(locale(), { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(mb / 1000);
  return `${gb} GB`;
}
