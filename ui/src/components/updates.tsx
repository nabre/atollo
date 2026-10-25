// The components with a new version (report v6/04, N-9, D-161): the badge
// on Tools and the pane «Aggiornamenti», each with what changes (its release
// notes) and «Aggiorna», instead of «Aggiorna tutto» blindly.
import { useCallback, useEffect, useRef } from "react";
import { componentLinkOpen, onUpdates, setupStart, updatesCheck, updatesList, type Updates } from "../ipc";
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
}: {
  expanded: boolean;
  onToggle: () => void;
  /** A preparation runs: «Aggiorna» waits. */
  running: boolean;
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
            {items.map((u) => (
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
                <ActionButton
                  icon={codicon("cloud-download")}
                  label={t("Aggiorna {0}", u.name)}
                  small
                  busy={busy === `update:${u.id}`}
                  disabled={running || busy !== null}
                  onClick={() => {
                    run(`update:${u.id}`, () => setupStart([u.id], true));
                  }}
                />
              </li>
            ))}
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
