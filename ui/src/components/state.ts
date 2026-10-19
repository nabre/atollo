// The state of the Tools page, shared by the first-start page and the
// explorer: the catalog as it is on disk, the choice not applied yet, what
// applying it installs and removes, the projects that need each component;
// on the Tools page also the component × system grid and the system shown
// («Piattaforma»), remembered between sessions (D-121).
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  componentsApply,
  componentsCheck,
  componentsList,
  componentsSync,
  projectsList,
  targetProfiles,
  toolsTargetSet,
  type ComponentCheck,
  type ComponentInfo,
  type ProjectInfo,
  type SetupSnapshot,
  type TargetGrid,
} from "../api";
import { toggle } from "./model";

export function useComponents(setup: SetupSnapshot | null, firstRun: boolean, onDone?: () => void) {
  const [target, setTargetState] = useState<string | undefined>(undefined);
  const [list, setList] = useState<ComponentInfo[] | null>(null);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [synced, setSynced] = useState<string[]>([]);
  const [checks, setChecks] = useState<Map<string, ComponentCheck>>(new Map());
  const [checking, setChecking] = useState(false);
  const [grid, setGrid] = useState<TargetGrid | null>(null);
  const [gridError, setGridError] = useState<string | null>(null);
  // The first-start page always shows this PC's system; the Tools page
  // waits for the system chosen last before reading the list.
  const [ready, setReady] = useState(firstRun);
  const host = grid?.profiles.find((p) => p.isHost)?.id;

  const loadGrid = useCallback(() => {
    if (firstRun) return;
    targetProfiles().then(
      (g) => {
        setGrid(g);
        setGridError(null);
      },
      (e: unknown) => {
        setGridError(String(e));
      },
    );
  }, [firstRun]);

  // Opening the Tools page: the grid, with «Piattaforma» chosen last.
  useEffect(() => {
    if (firstRun) return;
    targetProfiles().then(
      (g) => {
        setGrid(g);
        const own = g.profiles.find((p) => p.isHost)?.id;
        if (g.chosen && g.chosen !== own) setTargetState(g.chosen);
        setReady(true);
      },
      (e: unknown) => {
        setGridError(String(e));
        setReady(true);
      },
    );
  }, [firstRun]);

  /** Shows `next`, remembered in settings.json; this PC's system is saved
   * as none, so the kit opened on another PC shows that one. */
  const setTarget = useCallback(
    (next: string) => {
      setChecks(new Map());
      setTargetState(next === host ? undefined : next);
      // Remembering is a convenience: the page works without it.
      toolsTargetSet(next === host ? null : next).then(undefined, () => undefined);
    },
    [host],
  );

  const load = useCallback(() => {
    if (!ready) return;
    // Only for the highlight: the page works without the projects.
    projectsList().then(setProjects, () => {
      setProjects([]);
    });
    componentsList(target).then(
      (l) => {
        setList(l);
        setChosen(new Set(l.filter((c) => c.chosen || c.locked).map((c) => c.id)));
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, [target, ready]);

  const verifyTarget = useCallback(() => {
    setChecking(true);
    setError(null);
    componentsCheck(target).then(
      (res) => {
        setChecks(new Map(res.map((r) => [r.id, r])));
        setChecking(false);
      },
      (e: unknown) => {
        setError(String(e));
        setChecking(false);
      },
    );
  }, [target]);

  // Opening the page: first what was deleted by hand from the kit's
  // folder, then the list as it is on disk.
  useEffect(() => {
    if (firstRun) {
      load();
      return;
    }
    if (!ready) return;
    componentsSync(target).then(
      (off) => {
        setSynced(off);
        load();
      },
      () => {
        load();
      },
    );
  }, [firstRun, load, target, ready]);
  // Reload as each component of an installation ends, and when it ends:
  // versions and "installed" change.
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  const ended = setup !== null && !setup.running && setup.steps.length > 0;
  useEffect(() => {
    if (finished > 0 || ended) load();
  }, [finished, ended, load]);
  useEffect(() => {
    if (finished > 0 || ended) loadGrid();
  }, [finished, ended, loadGrid]);

  const plan = useMemo(() => {
    const items = list ?? [];
    const install = items.filter((c) => chosen.has(c.id) && !c.installed && !c.bootstrapStaged && c.available);
    // The choice is one for every system: a component taken out goes also
    // from the other systems that still have it (Rust `remove_elsewhere`).
    const shown = target ?? host;
    const elsewhere = (id: string) =>
      grid?.targets.some((s) => s.target !== shown && s.tools.some((x) => x.id === id && x.present)) ?? false;
    const remove = items.filter((c) => !chosen.has(c.id) && (c.present || elsewhere(c.id)));
    return { install, remove, mb: install.reduce((n, c) => n + c.mb, 0) };
  }, [list, chosen, grid, target, host]);

  /** Component id → names of the projects that need it. */
  const neededBy = useMemo(() => {
    const map = new Map<string, string[]>();
    for (const p of projects) {
      for (const id of p.requires) map.set(id, [...(map.get(id) ?? []), p.name]);
    }
    return map;
  }, [projects]);

  /** Turns `id` on or off, with what it needs and what needs it. */
  const choose = useCallback(
    (id: string, on: boolean) => {
      if (list) setChosen((current) => toggle(list, current, id, on));
    },
    [list],
  );

  const apply = useCallback(
    (updateAll: boolean) => {
      componentsApply([...chosen], updateAll, target).then(
        () => onDone?.(),
        (e: unknown) => {
          setError(String(e));
        },
      );
    },
    [chosen, onDone, target],
  );

  return {
    target,
    setTarget,
    /** This PC's system, once the grid is read. */
    host,
    grid,
    gridError,
    loadGrid,
    checks,
    checking,
    verifyTarget,
    list,
    chosen,
    choose,
    projects,
    error,
    synced,
    load,
    plan,
    neededBy,
    apply,
  };
}

export type ComponentsState = ReturnType<typeof useComponents>;
