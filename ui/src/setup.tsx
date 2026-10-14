// Progress of the preparation (installing, updating, removing components),
// shared by the status bar, the Activity panel and the Tools page.
import { useEffect, useState } from "react";
import { VscCheck, VscCircleSlash, VscError, VscLoading, VscWarning } from "react-icons/vsc";
import { onSetup, setupCancel, setupState, type Outcome, type SetupSnapshot, type SetupStep } from "./api";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";

/** The current preparation, kept up to date by the `setup` events. */
export function useSetup(): SetupSnapshot | null {
  const [snapshot, setSnapshot] = useState<SetupSnapshot | null>(null);
  useEffect(() => {
    let stop: (() => void) | undefined;
    let alive = true;
    onSetup(setSnapshot).then(
      (unlisten) => {
        if (alive) stop = unlisten;
        else unlisten();
      },
      () => undefined,
    );
    setupState().then(
      (s) => {
        if (alive) setSnapshot((current) => current ?? s);
      },
      () => undefined,
    );
    return () => {
      alive = false;
      stop?.();
    };
  }, []);
  return snapshot;
}

export function outcomeText(o: Outcome): string {
  switch (o.kind) {
    case "installed":
      return t("installato {0}", o.version);
    case "upToDate":
      return o.version ? t("già aggiornato ({0})", o.version) : t("già presente");
    case "removed":
      return t("rimosso");
    case "inUse":
      return t("in uso: chiudi i programmi del kit e riprova");
    case "unsupported":
      return t("non ancora disponibile in questa versione");
    case "cancelled":
      return t("annullato");
    case "failed":
      return t("non riuscito: {0}", o.message);
  }
}

function isProblem(o: Outcome | null): boolean {
  return o !== null && (o.kind === "failed" || o.kind === "inUse" || o.kind === "unsupported");
}

/** Running, or finished with an error or a component not installed: an
 * installation that went well is not shown again. */
export function setupToShow(s: SetupSnapshot | null): s is SetupSnapshot {
  if (!s) return false;
  if (s.running) return true;
  return s.error !== null || s.steps.some((step) => isProblem(step.outcome));
}

function percent(step: SetupStep): number | null {
  if (step.done === null || !step.size) return null;
  return Math.min(100, Math.round((step.done / step.size) * 100));
}

/** One line for the status bar, or null when idle. */
export function setupStatusText(s: SetupSnapshot | null): string | null {
  if (!s?.running) return null;
  const current = s.steps.at(-1);
  if (!current) return t("Installazione in preparazione…");
  const p = current.outcome ? null : percent(current);
  return t("Installazione {0}/{1}: {2}", s.steps.length, s.total, current.name) + (p === null ? "" : ` ${String(p)}%`);
}

function StepIcon({ step, running }: { step: SetupStep; running: boolean }) {
  const o = step.outcome;
  if (!o) {
    return running ? (
      <VscLoading className="animate-spin text-vsc-link" size={16} aria-label={t("in corso")} />
    ) : (
      <VscCircleSlash className="text-vsc-desc" size={16} aria-label={t("interrotto")} />
    );
  }
  if (o.kind === "failed") return <VscError className="text-vsc-error" size={16} aria-label={t("errore")} />;
  if (isProblem(o)) return <VscWarning className="text-vsc-warning" size={16} aria-label={t("attenzione")} />;
  if (o.kind === "cancelled") return <VscCircleSlash className="text-vsc-desc" size={16} aria-label={t("annullato")} />;
  return <VscCheck className="text-vsc-success" size={16} aria-label={t("fatto")} />;
}

/** Steps of the current (or last) preparation, with cancel. */
export function SetupProgress({ snapshot, title }: { snapshot: SetupSnapshot; title?: string }) {
  const [cancelError, setCancelError] = useState<string | null>(null);
  const problems = snapshot.steps.filter((s) => isProblem(s.outcome));
  return (
    <section aria-labelledby="setup-title" aria-live="polite" className="border-t border-vsc-border">
      <header className="flex items-center gap-2 px-3 py-1.5">
        <h2 id="setup-title" className="text-[13px] font-semibold">
          {title ??
            (snapshot.running
              ? snapshot.automatic
                ? t("Installazione dei componenti del kit")
                : t("Installazione in corso")
              : t("Ultima installazione"))}
        </h2>
        {snapshot.running && (
          <button
            type="button"
            disabled={snapshot.cancelling}
            onClick={() => {
              setupCancel().catch((e: unknown) => {
                setCancelError(String(e));
              });
            }}
            className="ml-auto rounded-sm border border-vsc-button-border bg-vsc-button2 px-2 py-0.5 text-[12px] text-vsc-button2-fg enabled:hover:bg-vsc-button2-hover disabled:opacity-50"
          >
            {snapshot.cancelling ? t("Annullamento…") : t("Annulla")}
          </button>
        )}
      </header>
      <ol className="space-y-1 px-3 py-2">
        {snapshot.steps.map((step) => {
          const p = step.outcome ? null : percent(step);
          return (
            <li key={step.id} className="flex items-start gap-2">
              <span className="mt-0.5 shrink-0">
                <StepIcon step={step} running={snapshot.running} />
              </span>
              <span className="min-w-0 flex-1">
                <span className="inline-flex items-center gap-1.5 align-top">
                  <ComponentLogo id={step.id} size={14} />
                  {step.name}
                </span>
                <span className="ml-2 text-vsc-desc select-text">
                  {step.outcome ? outcomeText(step.outcome) : (step.note ?? "")}
                  {p !== null && ` (${String(p)}%)`}
                </span>
                {p !== null && (
                  <span
                    role="progressbar"
                    aria-valuenow={p}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-label={t("Download di {0}", step.name)}
                    className="mt-1 block h-0.5 w-full bg-vsc-border"
                  >
                    <span className="block h-full bg-vsc-progress" style={{ width: `${String(p)}%` }} />
                  </span>
                )}
              </span>
            </li>
          );
        })}
        {snapshot.running && snapshot.steps.length === 0 && <li className="text-vsc-desc">{t("Preparazione…")}</li>}
      </ol>
      {(snapshot.error ?? cancelError) && (
        <p role="alert" className="border-t border-vsc-border px-3 py-1.5 text-vsc-error select-text">
          {snapshot.error ?? cancelError}
        </p>
      )}
      {!snapshot.running && problems.length > 0 && (
        <p className="border-t border-vsc-border px-3 py-1.5 text-vsc-warning">
          {problems.length === 1
            ? t("Un componente non è stato installato. Riprova da Strumenti.")
            : t("{0} componenti non sono stati installati. Riprova da Strumenti.", problems.length)}
        </p>
      )}
    </section>
  );
}
