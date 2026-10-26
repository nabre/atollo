// The components with a new version (report v6/04, N-9, D-161): the badge
// on Tools and the pane «Aggiornamenti», each with what changes (its release
// notes) and «Aggiorna», instead of «Aggiorna tutto» blindly.
import { useCallback, useEffect, useRef } from "react";
import {
  componentLinkOpen,
  onUpdates,
  setupStart,
  updatesCheck,
  updatesList,
  type SetupSnapshot,
  type Updates,
} from "../ipc";
import { ItemBar, ItemCancel, ItemStatus, itemBusy, itemState } from "./setup";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad, useSubscription } from "../shared/hooks";
import { t } from "../i18n";
import { codicon, IconButton, Link, ViewPane } from "../workbench";

/** The list, read again when the daily check ends and after a
 * preparation (an update makes an item go). */
export function useUpdates(running: boolean) {
  const { data, setData, reload } = useLoad<Updates | null>(updatesList, null, { onError: "keep" });
  useSubscription(onUpdates, () => {
    reload();
  });
  const wasRunning = useRef(running);
  useEffect(() => {
    if (wasRunning.current && !running) reload();
    wasRunning.current = running;
  }, [running, reload]);
  return { updates: data, setUpdates: setData };
}

function when(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString();
}

export function UpdatesPane({
  expanded,
  onToggle,
  running,
  setup = null,
}: {
  expanded: boolean;
  onToggle: () => void;
  /** A preparation runs: the list is read again when it ends. */
  running: boolean;
  /** The preparation: each row shows its own update (waiting, its bar,
   * «Annulla»); the others can be updated meanwhile, they queue. */
  setup?: SetupSnapshot | null;
}) {
  const { updates, setUpdates } = useUpdates(running);
  const { busy, error, run } = useAction();
  const check = useCallback(() => {
    run("check", async () => {
      setUpdates(await updatesCheck());
    });
  }, [run, setUpdates]);
  const items = updates?.items ?? [];
  return (
    <ViewPane
      title={t("Aggiornamenti")}
      badge={items.length > 0 ? String(items.length) : undefined}
      expanded={expanded}
      onToggle={onToggle}
      fit
      actions={
        <IconButton
          icon={codicon("refresh")}
          label={t("Controlla ora")}
          title={t("Chiede adesso l'ultima versione di ogni componente installato")}
          busy={busy === "check"}
          onClick={check}
        />
      }
    >
      <div className="space-y-1.5 px-3 py-1.5">
        {items.length === 0 ? (
          <p className="text-vsc-desc">
            {updates?.checked == null
              ? t(
                  "Non ancora controllato: «Controlla ora», o si controlla da solo una volta al giorno con l'aggiornamento automatico.",
                )
              : t("Tutto aggiornato.")}
          </p>
        ) : (
          <ul className="m-0 list-none space-y-1.5 p-0">
            {items.map((u) => {
              const item = itemState(setup, u.id, u.name);
              const working = itemBusy(setup, u.id);
              return (
                <li key={u.id} className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
                  <span className="font-semibold">{u.name}</span>
                  <span className="font-mono text-[12px] text-vsc-desc">
                    {u.installed} → {u.latest}
                  </span>
                  <span className="flex-1" />
                  <Link
                    onClick={() => {
                      run(`notes:${u.id}`, () => componentLinkOpen(u.id, "notes"));
                    }}
                  >
                    {t("Cosa cambia")}
                  </Link>
                  {item && <ItemStatus state={item} />}
                  {working ? (
                    <ItemCancel setup={setup} id={u.id} name={u.name} />
                  ) : (
                    <ActionButton
                      icon={codicon("cloud-download")}
                      label={item?.kind === "problem" ? t("Riprova") : t("Aggiorna {0}", u.name)}
                      small
                      busy={busy === `update:${u.id}`}
                      onClick={() => {
                        run(`update:${u.id}`, () => setupStart([u.id], true));
                      }}
                    />
                  )}
                  {item && <ItemBar state={item} label={u.name} className="w-full basis-full" />}
                </li>
              );
            })}
          </ul>
        )}
        {updates?.checked != null && (
          <p className="text-[11px] text-vsc-desc">{t("Controllato il {0}", when(updates.checked))}</p>
        )}
        <ErrorLine error={error} />
      </div>
    </ViewPane>
  );
}
