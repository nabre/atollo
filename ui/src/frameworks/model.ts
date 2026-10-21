// The instructions as data: the names of the slots, bases and additions,
// the state of the edited ones, the combination chosen and its plan.
import { useCallback, useEffect, useRef, useState } from "react";
import {
  componentsList,
  frameworksGet,
  frameworksPlan,
  frameworksSet,
  type ComponentInfo,
  type EnvVar,
  type FrameworkAction,
  type FrameworkPiece,
  type FrameworkPlan,
  type FrameworkVariant,
  type FrameworksView,
} from "../api";
import { t } from "../i18n";

/** The name of a slot of additions (one of each goes in a project). */
export function slotLabel(slot: string | undefined): string {
  switch (slot) {
    case "frontend":
      return t("Interfaccia");
    case "css":
      return t("Stili");
    case "desktop":
      return t("App desktop");
    case "mobile":
      return t("App mobile");
    case "tests":
      return t("Prove");
    case "admin":
      return t("Amministrazione");
    case "format":
      return t("Formattazione");
    case "database":
      return t("Database");
    case "bundler":
      return t("Sviluppo");
    case undefined:
    case "":
      return t("Altro");
    default:
      return slot;
  }
}

/** A description or note in the app's language (custom ones stay as written). */
export function textOf(view: FrameworksView | null, source: string | undefined): string {
  if (!source) return "";
  return view?.texts[source] ?? source;
}

export function bases(pieces: FrameworkPiece[]): FrameworkPiece[] {
  return pieces.filter((p) => p.kind === "base");
}

/** The groups of the bases, in the order of the trees (CATEGORIES in
 * tauri/src/projects/frameworks/mod.rs). */
export const CATEGORIES = [
  "web-php",
  "web-js",
  "web-python",
  "web-other",
  "cms",
  "docs",
  "desktop",
  "mobile",
  "data",
  "cli",
] as const;

/** The name of a group of bases; one made in Settings without a group is «Altro». */
export function categoryLabel(category: string | undefined): string {
  switch (category) {
    case "web-php":
      return t("Web con PHP");
    case "web-js":
      return t("Web con JavaScript e TypeScript");
    case "web-python":
      return t("Web con Python");
    case "web-other":
      return t("Web con altri linguaggi");
    case "cms":
      return t("CMS");
    case "docs":
      return t("Siti statici e documentazione");
    case "desktop":
      return t("App desktop");
    case "mobile":
      return t("App mobile");
    case "data":
      return t("Dati e documenti");
    case "cli":
      return t("Riga di comando e librerie");
    default:
      return t("Altro");
  }
}

/** The bases by group, in the trees' order, the empty groups left out. */
export function basesByCategory(pieces: FrameworkPiece[]): { id: string; label: string; bases: FrameworkPiece[] }[] {
  const all = bases(pieces);
  const known = (c: string | undefined) => (CATEGORIES as readonly string[]).includes(c ?? "");
  return [...CATEGORIES, "other"]
    .map((id) => ({
      id,
      label: categoryLabel(id),
      bases: all.filter((b) => (id === "other" ? !known(b.category) : b.category === id)),
    }))
    .filter((g) => g.bases.length > 0);
}

/** The additions that work with `base`. */
export function addonsFor(pieces: FrameworkPiece[], base: string): FrameworkPiece[] {
  return pieces.filter((p) => p.kind === "addon" && p.on?.[base] !== undefined);
}

const lines = (v: string[] | undefined) => (v ?? []).map((l) => l.trim()).filter(Boolean);

const envs = (v: EnvVar[] | undefined) =>
  (v ?? []).map((e) => ({ key: e.key.trim(), value: e.value.trim() })).filter((e) => e.key || e.value);

/** A piece as it is saved: trimmed, no empty lines (to compare). */
export function cleaned(p: FrameworkPiece): string {
  const actions = (a: FrameworkAction[] | undefined) =>
    (a ?? []).map((x) => ({ name: x.name.trim(), command: x.command.trim() })).filter((x) => x.name || x.command);
  const variant = (v: FrameworkVariant) => ({
    needs: lines(v.needs),
    create: lines(v.create),
    steps: lines(v.steps),
    actions: actions(v.actions),
    env: envs(v.env),
    database: v.database?.trim() ?? "",
    note: v.note?.trim() ?? "",
  });
  const on = Object.keys(p.on ?? {})
    .sort()
    .map((k) => [k, variant(p.on?.[k] ?? {})]);
  return JSON.stringify([
    p.id,
    p.kind,
    p.name.trim(),
    p.desc.trim(),
    p.home.trim(),
    p.slot?.trim() ?? "",
    lines(p.needs),
    lines(p.create),
    lines(p.steps),
    actions(p.actions),
    envs(p.env),
    p.database?.trim() ?? "",
    p.note?.trim() ?? "",
    on,
  ]);
}

/** The instructions as loaded and as edited, saved as they change with the
 * settings (D-137). */
export function useFrameworks() {
  const [view, setView] = useState<FrameworksView | null>(null);
  const [pieces, setPieces] = useState<FrameworkPiece[]>([]);
  const [dirty, setDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Counts the edits: a save that ends after another edit leaves it unsaved.
  const edits = useRef(0);
  // The edits saved (or loaded over), and the pieces as last edited: save
  // reads these, so it saves the latest even from a page closing.
  const savedEdits = useRef(0);
  const latest = useRef<FrameworkPiece[]>([]);
  const load = useCallback(() => {
    const at = edits.current;
    frameworksGet().then(
      (v) => {
        setView(v);
        setPieces(v.pieces);
        latest.current = v.pieces;
        savedEdits.current = at;
        setDirty(false);
        setError(null);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  const change = useCallback((next: FrameworkPiece[]) => {
    edits.current += 1;
    latest.current = next;
    setPieces(next);
    setDirty(true);
  }, []);
  // One save at a time: a save waits for the one before, then sends the
  // latest pieces, so an older save never lands last.
  const saving = useRef<Promise<void>>(Promise.resolve());
  const save = useCallback(() => {
    const next = saving.current.then(async () => {
      const at = edits.current;
      if (at === savedEdits.current) return;
      await frameworksSet(latest.current);
      savedEdits.current = Math.max(savedEdits.current, at);
      if (edits.current === at) load();
    });
    saving.current = next.catch(() => undefined);
    return next;
  }, [load]);
  return { view, pieces, dirty, error, load, change, save };
}

/** The wait after an edit of the frameworks before saving it. */
const FRAMEWORKS_DELAY = 800;

/** Saves the frameworks a moment after the last edit, and what still waits
 * when the page closes (`run` shows the save as busy and its error). */
export function useFrameworksAutosave(fw: Frameworks, run: (key: string, action: () => Promise<void>) => void) {
  const { dirty, pieces, save } = fw;
  useEffect(() => {
    if (!dirty) return;
    const timer = window.setTimeout(() => {
      run("frameworks", save);
    }, FRAMEWORKS_DELAY);
    return () => {
      window.clearTimeout(timer);
    };
  }, [dirty, pieces, save, run]);
  useEffect(
    () => () => {
      // The page is gone: nothing left to show an error in.
      save().catch(() => undefined);
    },
    [save],
  );
}

export type Frameworks = ReturnType<typeof useFrameworks>;

let catalog: Promise<ComponentInfo[]> | null = null;

/** The kit's components, asked once: the names and logos of the tags. */
export function useComponents(): ComponentInfo[] {
  const [list, setList] = useState<ComponentInfo[]>([]);
  useEffect(() => {
    let gone = false;
    catalog ??= componentsList().catch(() => []);
    void catalog.then((l) => {
      if (!gone) setList(l);
    });
    return () => {
      gone = true;
    };
  }, []);
  return list;
}

function newId(pieces: FrameworkPiece[], prefix: string): string {
  for (let i = 1; ; i++) {
    const id = `${prefix}-${String(i)}`;
    if (!pieces.some((p) => p.id === id)) return id;
  }
}

/** Adds a new base or addition and returns its id. */
export function addPiece(fw: Frameworks, kind: FrameworkPiece["kind"]): string {
  const id = newId(fw.pieces, kind);
  const piece: FrameworkPiece =
    kind === "base"
      ? { id, kind, name: t("Nuova base"), desc: "", home: "", create: ["<comando> {name}"] }
      : { id, kind, name: t("Nuova aggiunta"), desc: "", home: "", on: {} };
  fw.change([...fw.pieces, piece]);
  return id;
}

/** The tab id of a piece in the Settings page. */
export const pieceTab = (id: string) => `fw:${id}`;

/** The tab of what addition `addon` does with base `base` (its `on` entry):
 * `@` is never in an id. */
export const integrationTab = (addon: string, base: string) => `fw:${addon}@${base}`;

export function parseIntegrationTab(tab: string): { addon: string; base: string } | null {
  const m = /^fw:([^@]+)@(.+)$/.exec(tab);
  return m?.[1] && m[2] ? { addon: m[1], base: m[2] } : null;
}

/** The additions that go with `base`, by slot in the order of the catalog. */
export function integrationsOf(pieces: FrameworkPiece[], base: string): { slot: string; addons: FrameworkPiece[] }[] {
  const list = addonsFor(pieces, base);
  return [...new Set(list.map((p) => p.slot ?? ""))].map((slot) => ({
    slot,
    addons: list.filter((p) => (p.slot ?? "") === slot),
  }));
}
export const TRY_TAB = "fw-try";

/** Chooses a base and its additions (one per slot). */
export function useCombination(pieces: FrameworkPiece[]) {
  const [base, setBase] = useState<string>("");
  const [addons, setAddons] = useState<string[]>([]);
  const chosen = bases(pieces).some((b) => b.id === base) ? base : (bases(pieces)[0]?.id ?? "");
  const available = addonsFor(pieces, chosen);
  const picked = addons.filter((a) => available.some((p) => p.id === a));
  const toggle = (id: string, on: boolean) => {
    const slot = pieces.find((p) => p.id === id)?.slot;
    const sameSlot = (a: string) => slot !== undefined && slot !== "" && pieces.find((p) => p.id === a)?.slot === slot;
    setAddons(on ? [...picked.filter((a) => !sameSlot(a)), id] : picked.filter((a) => a !== id));
  };
  return { base: chosen, setBase, addons: picked, toggle, available };
}

/** The plan of a combination, asked again when it changes. */
export function usePlan(pieces: FrameworkPiece[], base: string, addons: string[]) {
  const [plan, setPlan] = useState<FrameworkPlan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const key = JSON.stringify([base, addons]);
  useEffect(() => {
    if (!base) return;
    let gone = false;
    const timer = window.setTimeout(() => {
      frameworksPlan(pieces, base, addons).then(
        (p) => {
          if (gone) return;
          setPlan(p);
          setError(null);
        },
        (e: unknown) => {
          if (gone) return;
          setPlan(null);
          setError(String(e));
        },
      );
    }, 150);
    return () => {
      gone = true;
      window.clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- key stands for base and addons
  }, [pieces, key]);
  return { plan, error };
}
