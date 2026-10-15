// The instructions as data: the names of the slots, bases and additions,
// the state of the edited ones, the combination chosen and its plan.
import { useCallback, useEffect, useState } from "react";
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
    p.note?.trim() ?? "",
    on,
  ]);
}

/** The instructions as loaded and as edited, saved with the settings. */
export function useFrameworks() {
  const [view, setView] = useState<FrameworksView | null>(null);
  const [pieces, setPieces] = useState<FrameworkPiece[]>([]);
  const [dirty, setDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(() => {
    frameworksGet().then(
      (v) => {
        setView(v);
        setPieces(v.pieces);
        setDirty(false);
        setError(null);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  const change = useCallback((next: FrameworkPiece[]) => {
    setPieces(next);
    setDirty(true);
  }, []);
  const save = useCallback(async () => {
    if (!dirty) return;
    await frameworksSet(pieces);
    load();
  }, [dirty, pieces, load]);
  return { view, pieces, dirty, error, load, change, save };
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
