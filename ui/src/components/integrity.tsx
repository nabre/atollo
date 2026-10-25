// «Verifica i file» of the Tools page (report v6/04, N-5, D-159): every
// installed component against the manifest written at its installation;
// the damaged ones with «Reinstalla», at their version.
import { useCallback, useState } from "react";
import { componentsIntegrity, componentsRepair, type ComponentIntegrity } from "../ipc";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction } from "../shared/hooks";
import { t } from "../i18n";
import { codicon, Dot, Notice } from "../workbench";

export interface IntegrityState {
  report: ComponentIntegrity[] | null;
  busy: string | null;
  error: string | null;
  check: () => void;
  repair: (items: ComponentIntegrity[]) => void;
  close: () => void;
}

export function useIntegrity(target: string | undefined): IntegrityState {
  const [report, setReport] = useState<ComponentIntegrity[] | null>(null);
  const { busy, error, run } = useAction();
  const check = useCallback(() => {
    run("check", async () => {
      setReport(await componentsIntegrity(target));
    });
  }, [run, target]);
  const repair = useCallback(
    (items: ComponentIntegrity[]) => {
      run("repair", async () => {
        await componentsRepair(
          items.map((i) => ({ id: i.id, line: i.line })),
          target,
        );
        // The preparation shows its progress; the next check reads again.
        setReport(null);
      });
    },
    [run, target],
  );
  const close = useCallback(() => {
    setReport(null);
  }, []);
  return { report, busy, error, check, repair, close };
}

function nameOf(i: ComponentIntegrity): string {
  return i.line ? `${i.name} ${i.line}` : i.name;
}

const damaged = (i: ComponentIntegrity) =>
  i.error !== null || (i.damage !== null && (i.damage.missing.length > 0 || i.damage.changed.length > 0));

/** The result, under the toolbar: the damaged folders first, with
 * «Reinstalla»; then how many are fine and how many have no manifest. */
export function IntegrityReport({ state, running }: { state: IntegrityState; running: boolean }) {
  const { report, busy, error } = state;
  if (report === null) return error ? <ErrorLine error={error} /> : null;
  const bad = report.filter(damaged);
  const ok = report.filter((i) => !damaged(i) && i.damage !== null);
  const unknown = report.filter((i) => i.damage === null && i.error === null);
  return (
    <section aria-label={t("Verifica i file")} className="space-y-2">
      {bad.length === 0 ? (
        <Notice tone="success">{t("Nessun file danneggiato: {0} cartelle verificate.", String(ok.length))}</Notice>
      ) : (
        <Notice tone="warning">
          <div className="space-y-2">
            <p>
              {t(
                "File mancanti o cambiati dall'installazione: la chiavetta può essersi danneggiata. Reinstallare rimette i file del programma, alla stessa versione; progetti e database non si toccano.",
              )}
            </p>
            <ul className="m-0 list-none space-y-1.5 p-0">
              {bad.map((i) => (
                <li key={`${i.id}@${i.line ?? ""}`} className="flex flex-wrap items-start gap-2">
                  <span className="pt-1">
                    <Dot tone="err" />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="font-semibold">{nameOf(i)}</span>
                    <span className="text-vsc-desc">
                      {" · "}
                      {i.error ??
                        t(
                          "{0} file cambiati, {1} mancanti",
                          String(i.damage?.changed.length ?? 0),
                          String(i.damage?.missing.length ?? 0),
                        )}
                    </span>
                    {i.damage && (
                      <span className="block font-mono text-[12px] break-all text-vsc-desc select-text">
                        {[...i.damage.changed, ...i.damage.missing].slice(0, 5).join("  ")}
                      </span>
                    )}
                  </span>
                  <ActionButton
                    icon={codicon("sync")}
                    label={t("Reinstalla")}
                    small
                    busy={busy === "repair"}
                    disabled={running || busy !== null}
                    onClick={() => {
                      state.repair([i]);
                    }}
                  />
                </li>
              ))}
            </ul>
            {bad.length > 1 && (
              <ActionButton
                icon={codicon("sync")}
                label={t("Reinstalla tutti")}
                primary
                busy={busy === "repair"}
                disabled={running || busy !== null}
                onClick={() => {
                  state.repair(bad);
                }}
              />
            )}
          </div>
        </Notice>
      )}
      {unknown.length > 0 && (
        <p className="text-[12px] text-vsc-desc">
          {t(
            "Senza manifesto, installati prima che Atollo lo tenesse: {0}. Si verificano dalla prossima installazione o aggiornamento.",
            unknown.map(nameOf).join(", "),
          )}
        </p>
      )}
      <div className="flex gap-2">
        <ActionButton icon={codicon("close")} label={t("Chiudi")} small onClick={state.close} />
      </div>
      <ErrorLine error={error} />
    </section>
  );
}
