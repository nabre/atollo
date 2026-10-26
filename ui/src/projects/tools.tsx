// The quick panel's «Strumenti»: the components this PC can get, as a grid
// of tiles in the groups of the Tools page, each with its state and the one
// thing to do now (install it, update it). The same catalog, states and
// actions as the Tools page (../components): «Installa» installs at once as
// the row of «Tutti gli strumenti», «Aggiorna» as the pane «Aggiornamenti»;
// both run as a preparation, shown in Activity. The app's own parts and the
// required ones (locked) are always there and stay out. Each tile shows its
// own install (waiting, its bar, «Riprova», «Annulla»); asked while another
// runs, it queues; a tile that ends is read again alone, the grid stays.
import { useCallback, useState } from "react";
import {
  componentsInstall,
  componentsList,
  onLevelsChanged,
  setupStart,
  type ComponentInfo,
  type SetupSnapshot,
} from "../ipc";
import { groups } from "../components/model";
import { ItemBar, ItemCancel, itemBusy, itemState, useStepsDone, type ItemState } from "../components/setup";
import { shownStatus, ToolStateIcon, toolStateLabel, type ToolStatus } from "../components/status";
import { useUpdates } from "../components/updates";
import { t } from "../i18n";
import { ErrorLine } from "../shared/common";
import { useAction, useLoad, useSubscription } from "../shared/hooks";
import { ComponentLogo } from "../shared/logos";
import { composerOf, splitVersion } from "../versions/model";
import { Button, EditorSection, Link } from "../workbench";

/** The tiles shown before «Mostra tutti». */
export const TOOLS_SHOWN = 12;

/** A tile of the grid: what can be chosen, not the app's own parts. */
export function inGrid(c: ComponentInfo): boolean {
  return !c.locked && c.group !== "app";
}

/** What the tile's button does now, if anything. */
type Act = "install" | "update" | null;

function actOf(c: ComponentInfo, st: ToolStatus): Act {
  if (st.state === "update") return "update";
  // An agent (LEVELS.md §7): «fornito dalla VM», «serve la VM».
  if (st.state !== "missing" || c.agent !== null) return null;
  // As the row of «Tutti gli strumenti»: nothing when not made or not
  // needed here (D-177).
  if (!c.available || c.unavailableReason !== null || c.notNeeded !== null) return null;
  return "install";
}

export function HomeTools({ setup, onOpen }: { setup: SetupSnapshot | null; onOpen: (id: string) => void }) {
  const running = setup?.running ?? false;
  const {
    data: list,
    setData: setList,
    error: loadError,
    reload,
  } = useLoad<ComponentInfo[] | null>(componentsList, null, {
    onError: "show",
  });
  // The agents' tiles follow the VM (LEVELS.md §7).
  useSubscription(onLevelsChanged, reload);
  const { updates, setUpdates } = useUpdates(running);
  const { busy, error, run } = useAction();
  const [all, setAll] = useState(false);
  // As each component of a preparation ends: its tile alone read again (the
  // others and the grid stay), and an update done leaves the list of the
  // new versions.
  useStepsDone(setup, (ids) => {
    if (ids.length === 0) return;
    componentsList(undefined, ids).then(
      (fresh) => {
        const byId = new Map(fresh.map((c) => [c.id, c]));
        setList((current) => current?.map((c) => byId.get(c.id) ?? c) ?? current);
        setUpdates((current) =>
          current
            ? {
                ...current,
                items: current.items.filter(
                  (u) => !fresh.some((c) => c.id === u.id && c.version !== null && c.version !== u.installed),
                ),
              }
            : current,
        );
      },
      () => undefined,
    );
  });

  const status = useCallback(
    (c: ComponentInfo): ToolStatus => {
      const st = shownStatus(c, undefined, c.chosen || c.locked, itemState(setup, c.id, c.name), setup?.target ?? "");
      const upd = updates?.items.find((u) => u.id === c.id);
      if (st.state === "installed" && upd) return { state: "update", detail: `${upd.installed} → ${upd.latest}` };
      return st;
    },
    [setup, updates],
  );

  if (list === null && !loadError) return null;
  const tools = (list ?? []).filter(inGrid);
  const kinds = groups()
    .map((g) => ({ ...g, items: tools.filter((c) => g.has(c)) }))
    .filter((g) => g.items.length > 0);
  // Folded, the first tiles in the order of the groups.
  const visible = new Set(
    kinds
      .flatMap((g) => g.items)
      .slice(0, all ? tools.length : TOOLS_SHOWN)
      .map((c) => c.id),
  );
  const shown = kinds
    .map((g) => ({ ...g, items: g.items.filter((c) => visible.has(c.id)) }))
    .filter((g) => g.items.length > 0);
  const installed = tools.filter((c) => c.installed || Boolean(c.bootstrapStaged)).length;

  return (
    <EditorSection
      id="home-tools"
      title={t("Strumenti")}
      actions={
        tools.length > 0 && (
          <span className="text-[12px] text-vsc-desc tabular-nums">
            {t("{0} installati su {1}", installed, tools.length)}
          </span>
        )
      }
    >
      <ErrorLine error={loadError} />
      <ErrorLine error={error} />
      {shown.map((g) => (
        <section key={g.id} aria-labelledby={`home-tools-${g.id}`} className="space-y-1.5">
          <h4 id={`home-tools-${g.id}`} className="text-[12px] text-vsc-desc">
            {g.title}
          </h4>
          <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 p-0">
            {g.items.map((c) => (
              <ToolTile
                key={c.id}
                c={c}
                st={status(c)}
                item={itemState(setup, c.id, c.name)}
                setup={setup}
                busy={busy}
                disabled={itemBusy(setup, c.id) || (busy?.endsWith(`:${c.id}`) ?? false)}
                onOpen={onOpen}
                onInstall={() => {
                  run(`install:${c.id}`, () => componentsInstall(c.id));
                }}
                onUpdate={() => {
                  run(`update:${c.id}`, () => setupStart([c.id], true));
                }}
              />
            ))}
          </ul>
        </section>
      ))}
      {tools.length > TOOLS_SHOWN && (
        <Link
          onClick={() => {
            setAll((v) => !v);
          }}
        >
          {all ? t("Mostra meno") : t("Mostra tutti ({0})", tools.length)}
        </Link>
      )}
    </EditorSection>
  );
}

/** One tool: its logo, name and state open its page on Tools; the button
 * beside does the one thing to do now. */
function ToolTile({
  c,
  st,
  item,
  setup,
  busy,
  disabled,
  onOpen,
  onInstall,
  onUpdate,
}: {
  c: ComponentInfo;
  st: ToolStatus;
  /** Its install: waiting, running, failed; null when none. */
  item: ItemState | null;
  setup: SetupSnapshot | null;
  busy: string | null;
  disabled: boolean;
  onOpen: (id: string) => void;
  onInstall: () => void;
  onUpdate: () => void;
}) {
  const act = actOf(c, st);
  const version = composerOf(c.version) ? splitVersion(c.version).main : c.version;
  const line =
    st.state === "installed"
      ? (version ?? toolStateLabel("installed"))
      : st.state === "update"
        ? (st.detail ?? toolStateLabel("update"))
        : st.state === "missing"
          ? (st.detail ?? t("non installato"))
          : st.state === "running" && st.detail
            ? `${toolStateLabel("running")} · ${st.detail}`
            : toolStateLabel(st.state);
  const hint = [c.desc, st.state === "missing" || st.state === "running" ? null : st.detail]
    .filter(Boolean)
    .join(" — ");
  const working = item?.kind === "queued" || item?.kind === "running";
  return (
    <li className="flex min-w-0 flex-wrap items-center gap-2 rounded-md border border-vsc-border p-2 hover:bg-vsc-list-hover">
      <button
        type="button"
        title={hint || undefined}
        aria-label={t("{0}: {1}. Apre la sua pagina in Strumenti", c.name, line)}
        onClick={() => {
          onOpen(c.id);
        }}
        className="flex min-w-0 flex-1 items-center gap-2 rounded-sm text-left"
      >
        <span className="shrink-0">
          <ComponentLogo id={c.id} size={24} />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate font-semibold">{c.name}</span>
          <span className="flex min-w-0 items-center gap-1 text-[12px] text-vsc-desc">
            <ToolStateIcon state={st.state} />
            <span className={`truncate ${st.state === "installed" || st.state === "update" ? "font-mono" : ""}`}>
              {line}
            </span>
          </span>
        </span>
      </button>
      {working && <ItemCancel setup={setup} id={c.id} name={c.name} />}
      {item?.kind === "problem" && !working && (
        <Button small title={item.text} disabled={disabled} onClick={act === "update" ? onUpdate : onInstall}>
          {t("Riprova")}
        </Button>
      )}
      {act === "install" && !working && item?.kind !== "problem" && (
        <Button
          small
          variant="primary"
          title={t("Installa {0} all'ultima versione, subito", c.name)}
          busy={busy === `install:${c.id}`}
          disabled={disabled}
          onClick={onInstall}
        >
          {t("Installa")}
        </Button>
      )}
      {act === "update" && !working && item?.kind !== "problem" && (
        <Button
          small
          title={t("Scarica la versione nuova di {0}, se c'è", c.name)}
          busy={busy === `update:${c.id}`}
          disabled={disabled}
          onClick={onUpdate}
        >
          {t("Aggiorna ora")}
        </Button>
      )}
      {item && <ItemBar state={item} label={c.name} className="w-full basis-full" />}
    </li>
  );
}
