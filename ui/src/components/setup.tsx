// Progress of the preparation (installing, updating, removing components),
// shared by the status bar, the Activity panel and the Tools page.
import { useEffect, useRef, useState } from "react";
import {
  jobCancel,
  onSetup,
  setupCancel,
  setupState,
  type ByHand,
  type Outcome,
  type SetupSnapshot,
  type SetupStep,
} from "../ipc";
import { t } from "../i18n";
import { ComponentLogo } from "../shared/logos";
import { useSubscription } from "../shared/hooks";
import { Button, Codicon, codicon, IconButton } from "../workbench";

/** The current preparation, kept up to date by the `setup` events. */
export function useSetup(): SetupSnapshot | null {
  const [snapshot, setSnapshot] = useState<SetupSnapshot | null>(null);
  useSubscription(onSetup, setSnapshot);
  useEffect(() => {
    let alive = true;
    setupState().then(
      (s) => {
        if (alive) setSnapshot((current) => current ?? s);
      },
      () => undefined,
    );
    return () => {
      alive = false;
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

/** Where an element of a list stands in the preparation: waiting, running
 * (with the download's percent or the installer's note), or not done. */
export type ItemState =
  | { kind: "queued" }
  | { kind: "running"; percent: number | null; note: string | null }
  | { kind: "problem"; text: string; byHand?: ByHand | null };

/** The state of the step `name` of component `id`; null when the
 * preparation does not touch it or it went well. */
export function itemState(s: SetupSnapshot | null, id: string, name: string): ItemState | null {
  if (!s) return null;
  const step = [...s.steps].reverse().find((x) => x.id === id && x.name === name);
  // Asked again after it ended (in the queue of the runs): waiting, not
  // its last outcome.
  const again = s.running && (s.pending ?? []).some((p) => p.items.includes(id));
  if (!step || (step.outcome !== null && again)) {
    return s.running && (s.queue.some((q) => q.id === id && q.name === name) || again) ? { kind: "queued" } : null;
  }
  const o = step.outcome;
  if (o === null) {
    return s.running
      ? { kind: "running", percent: percent(step), note: step.note }
      : { kind: "problem", text: t("interrotto") };
  }
  if (!isProblem(o)) return null;
  return { kind: "problem", text: outcomeText(o), byHand: o.kind === "failed" ? o.byHand : null };
}

/** Component `id` is being worked on: its step runs, or it waits in the
 * run or in the queue of the runs. Only its own buttons wait: the other
 * rows can be installed meanwhile (they queue). */
export function itemBusy(s: SetupSnapshot | null, id: string): boolean {
  if (!s?.running) return false;
  const last = s.steps.at(-1);
  if (last?.id === id && last.outcome === null) return true;
  return s.queue.some((q) => q.id === id) || (s.pending ?? []).some((p) => p.items.includes(id));
}

/** The job that would cancel component `id` alone: the queued run that
 * asks for it, or the run going on while it runs or waits there. */
export function itemJob(s: SetupSnapshot | null, id: string): string | null {
  if (!s?.running) return null;
  const queued = (s.pending ?? []).find((p) => p.items.includes(id));
  if (queued) return queued.job;
  const last = s.steps.at(-1);
  const here = (last?.id === id && last.outcome === null) || s.queue.some((q) => q.id === id);
  return here ? (s.job ?? null) : null;
}

/** «Annulla» of one element: its job leaves the queue, or stops. */
export function ItemCancel({ setup, id, name }: { setup: SetupSnapshot | null; id: string; name: string }) {
  const job = itemJob(setup, id);
  const [asked, setAsked] = useState<string | null>(null);
  if (!job) return null;
  return (
    <IconButton
      icon={codicon("close")}
      label={t("Annulla {0}", name)}
      title={t("Annulla l'installazione di {0}", name)}
      disabled={asked === job || setup?.cancelling === true}
      onClick={() => {
        setAsked(job);
        jobCancel(job).catch(() => {
          setAsked(null);
        });
      }}
    />
  );
}

/** Calls `handler` with the components whose step just ended (each once),
 * and with `true` when the whole queue of runs ended: what a page reads
 * again, only those rows, instead of the whole list. */
export function useStepsDone(s: SetupSnapshot | null, handler: (ids: string[], ended: boolean) => void): void {
  const seen = useRef<{ steps: number; done: number; running: boolean }>({ steps: 0, done: 0, running: false });
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    if (!s) return;
    const before = seen.current;
    // The steps only append, each ending before the next begins: those
    // ended are the first ones. A new chain of runs starts them again.
    const from = s.steps.length < before.steps ? 0 : before.done;
    const finished = s.steps.filter((x) => x.outcome !== null);
    const ids = finished.slice(from).map((x) => x.id);
    const ended = before.running && !s.running;
    seen.current = { steps: s.steps.length, done: finished.length, running: s.running };
    if (ids.length > 0 || ended) latest.current([...new Set(ids)], ended);
  }, [s]);
}

/** The short state of an element, in its row. */
export function ItemStatus({ state }: { state: ItemState }) {
  if (state.kind === "queued") {
    return (
      <span className="text-vsc-desc">
        <Codicon name="history" className="mr-1" size={12} />
        {t("in coda")}
      </span>
    );
  }
  if (state.kind === "running") {
    return (
      <span className="text-vsc-link" title={state.note ?? undefined}>
        <Codicon name="loading" className="mr-1 animate-spin" size={12} />
        {state.percent !== null ? t("download {0}%", state.percent) : (state.note ?? t("in corso"))}
      </span>
    );
  }
  return (
    <span className="text-vsc-error" title={state.text}>
      <Codicon name="error" className="mr-1" size={12} />
      {t("non riuscito")}
    </span>
  );
}

/** A download's bar: vscode-progress-bar, determinate, 2px in the
 * progressBar color. The role, value and name sit on the span around it,
 * in the page; the component's own (inside its shadow root) is hidden. */
function DownloadBar({ percent, label, className = "" }: { percent: number; label: string; className?: string }) {
  return (
    <span
      role="progressbar"
      aria-valuenow={percent}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={t("Download di {0}", label)}
      className={`block ${className}`}
    >
      <vscode-progress-bar value={percent} max={100} aria-hidden="true" />
    </span>
  );
}

/** The download's bar under an element, while it runs. */
export function ItemBar({ state, label, className = "" }: { state: ItemState; label: string; className?: string }) {
  if (state.kind !== "running" || state.percent === null) return null;
  return <DownloadBar percent={state.percent} label={label} className={className} />;
}

function StepIcon({ step, running }: { step: SetupStep; running: boolean }) {
  const o = step.outcome;
  if (!o) {
    return running ? (
      <Codicon name="loading" className="animate-spin text-vsc-link" size={16} label={t("in corso")} />
    ) : (
      <Codicon name="circle-slash" className="text-vsc-desc" size={16} label={t("interrotto")} />
    );
  }
  if (o.kind === "failed") return <Codicon name="error" className="text-vsc-error" size={16} label={t("errore")} />;
  if (isProblem(o)) return <Codicon name="warning" className="text-vsc-warning" size={16} label={t("attenzione")} />;
  if (o.kind === "cancelled")
    return <Codicon name="circle-slash" className="text-vsc-desc" size={16} label={t("annullato")} />;
  return <Codicon name="check" className="text-vsc-success" size={16} label={t("fatto")} />;
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
          <Button
            disabled={snapshot.cancelling}
            onClick={() => {
              setupCancel().catch((e: unknown) => {
                setCancelError(String(e));
              });
            }}
            className="ml-auto"
          >
            {snapshot.cancelling ? t("Annullamento…") : t("Annulla")}
          </Button>
        )}
      </header>
      <ol className="space-y-1 px-3 py-2">
        {snapshot.steps.map((step, i) => {
          const p = step.outcome ? null : percent(step);
          // A run can touch a component twice (removed, then installed; two
          // lines patched): the steps only append, so their place tells
          // them apart.
          return (
            <li key={`${String(i)}:${step.id}`} className="flex items-start gap-2">
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
                {p !== null && <DownloadBar percent={p} label={step.name} className="mt-1 w-full" />}
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
