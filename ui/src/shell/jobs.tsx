// The long work of the app in sight (LEVELS.md §8.6): the jobs (installs,
// updates, the preparation of a project's VM) kept up to date by the `job`
// events, shared by the status bar, the VM's pill, the Activity panel and
// the session waiting for its VM; the bar of a job, its details with their
// steps and errors, the card «Preparazione della VM…».
import { useState } from "react";
import { jobCancel, jobsList, onJob, setupStart, type Job, type JobStep } from "../ipc";
import { t } from "../i18n";
import { createFeed, useFeed } from "../shared/feed";
import { Button, Codicon } from "../workbench";

/** `job` in `list`, in place of the one with its id or added; the list in
 * the order the jobs were asked. */
export function upsertJob(list: Job[], job: Job): Job[] {
  const i = list.findIndex((j) => j.id === job.id);
  if (i < 0) return [...list, job].sort((a, b) => a.created - b.created);
  const next = list.slice();
  next[i] = job;
  return next;
}

/** The jobs of the window, read once, then kept by the events (once a
 * frame: a download sends its bytes many times a second); one subscription
 * for every component that shows them. */
const jobsFeed = createFeed<Job[], Job>({
  initial: [],
  listen: onJob,
  apply: upsertJob,
  // An event arrived meanwhile is newer than the list.
  read: () =>
    jobsList().then(
      (list) => (current) =>
        list.reduce((acc, j) => (acc.some((x) => x.id === j.id) ? acc : upsertJob(acc, j)), current),
    ),
});

/** The jobs, rendering the component when they change (at most once a
 * frame): each part that shows them asks here, so their progress renders
 * those parts, not the window. */
export function useJobs(): Job[] {
  return useFeed(jobsFeed);
}

/** Queued or running. */
export function isActive(j: Job): boolean {
  return j.state === "queued" || j.state === "running";
}

/** The jobs going on, the running ones first, then in the order asked. */
export function activeJobs(jobs: Job[]): Job[] {
  return jobs
    .filter(isActive)
    .sort((a, b) => Number(b.state === "running") - Number(a.state === "running") || a.created - b.created);
}

/** The download's percent of the step running, when its size is known. */
export function bytesPercent(j: Job): number | null {
  const b = j.bytes;
  if (!b?.total) return null;
  return Math.min(100, Math.round((b.done / b.total) * 100));
}

/** How far the job is, its steps done and the download of the one running;
 * null before its first step. */
export function overallPercent(j: Job): number | null {
  if (j.state === "done") return 100;
  const s = j.step;
  if (!s || s.total === 0) return null;
  const part = (bytesPercent(j) ?? 0) / 100;
  return Math.min(100, Math.round(((s.index - 1 + part) / s.total) * 100));
}

/** The step running in a line: «2/5 · Database del progetto · Node.js». */
export function stepText(j: Job): string {
  if (j.state === "queued") return t("In coda");
  const s = j.step;
  if (!s) return t("Avvio…");
  return [`${String(s.index)}/${String(s.total)}`, s.label, j.note].filter(Boolean).join(" · ");
}

/** A job's bar: its percent, or a part going across when not known; the
 * role, value and name for screen readers. */
export function JobBar({
  percent,
  label,
  className = "",
}: {
  percent: number | null;
  label: string;
  className?: string;
}) {
  return (
    <span
      role="progressbar"
      aria-label={label}
      aria-valuemin={percent === null ? undefined : 0}
      aria-valuemax={percent === null ? undefined : 100}
      aria-valuenow={percent ?? undefined}
      aria-busy={percent === null ? true : undefined}
      className={`relative block h-[3px] overflow-hidden rounded-vsc-circle bg-vsc-border ${className}`}
    >
      {percent === null ? (
        <span className="absolute inset-y-0 left-0 w-1/3 animate-[indeterminate_1.4s_ease-in-out_infinite] rounded-vsc-circle bg-vsc-progress" />
      ) : (
        <span className="block h-full rounded-vsc-circle bg-vsc-progress" style={{ width: `${String(percent)}%` }} />
      )}
    </span>
  );
}

function StepIcon({ step }: { step: JobStep }) {
  switch (step.state) {
    case "running":
      return <Codicon name="loading" className="animate-spin text-vsc-link" size={14} label={t("in corso")} />;
    case "queued":
      return <Codicon name="history" className="text-vsc-desc" size={14} label={t("in coda")} />;
    case "error":
      return <Codicon name="error" className="text-vsc-error" size={14} label={t("errore")} />;
    case "cancelled":
      return <Codicon name="circle-slash" className="text-vsc-desc" size={14} label={t("annullato")} />;
    case "done":
      return <Codicon name="check" className="text-vsc-success" size={14} label={t("fatto")} />;
  }
}

/** What a job's state is, in words. */
export function jobStateText(j: Job): string {
  switch (j.state) {
    case "queued":
      return t("In coda");
    case "running":
      return t("In corso");
    case "done":
      return t("Fatto");
    case "error":
      return t("Non riuscito");
    case "cancelled":
      return t("Annullato");
  }
}

/** A job in the Activity panel: its title and state, its bar, each step
 * with its error, «Annulla» while an install waits or runs, «Riprova» for
 * one that failed. */
export function JobDetails({ job }: { job: Job }) {
  const [error, setError] = useState<string | null>(null);
  const [asked, setAsked] = useState(false);
  const setup = job.kind === "install" || job.kind === "update";
  const percent = job.state === "running" ? (bytesPercent(job) ?? overallPercent(job)) : null;
  return (
    <section aria-label={job.title} aria-live="polite" className="space-y-2 px-3 py-2">
      <header className="flex flex-wrap items-center gap-2">
        <h2 className="text-[13px] font-semibold">{job.title}</h2>
        <span className="text-vsc-desc">{jobStateText(job)}</span>
        <span className="flex-1" />
        {setup && isActive(job) && (
          <Button
            small
            disabled={asked}
            onClick={() => {
              setAsked(true);
              jobCancel(job.id).catch((e: unknown) => {
                setAsked(false);
                setError(String(e));
              });
            }}
          >
            {asked ? t("Annullamento…") : t("Annulla")}
          </Button>
        )}
        {setup && job.state === "error" && job.items.length > 0 && (
          <Button
            small
            onClick={() => {
              setupStart(job.items, job.kind === "update").catch((e: unknown) => {
                setError(String(e));
              });
            }}
          >
            {t("Riprova")}
          </Button>
        )}
      </header>
      {isActive(job) && (
        <>
          <p className="text-vsc-desc">{stepText(job)}</p>
          <JobBar percent={percent} label={job.title} className="w-full max-w-[480px]" />
        </>
      )}
      {job.steps.length > 0 && (
        <ol className="m-0 list-none space-y-1 p-0">
          {job.steps.map((s) => (
            <li key={`${String(s.index)}:${s.label}`} className="flex items-start gap-2">
              <span className="mt-0.5 shrink-0">
                <StepIcon step={s} />
              </span>
              <span className="min-w-0 flex-1">
                <span>
                  {String(s.index)}/{String(s.total)} · {s.label}
                </span>
                {s.error && <span className="block text-vsc-error select-text">{s.error}</span>}
              </span>
            </li>
          ))}
        </ol>
      )}
      {job.error && !job.steps.some((s) => s.error === job.error) && (
        <p role="alert" className="text-vsc-error select-text">
          {job.error}
        </p>
      )}
      {error && (
        <p role="alert" className="text-vsc-error select-text">
          {error}
        </p>
      )}
    </section>
  );
}

/** The VM being prepared, where a session waits for it (the terminal, an
 * action, an agent): its title, step and bar; `project` only that
 * project's. Nothing when no VM is being prepared. */
export function VmPreparing({ project, onDetails }: { project?: string; onDetails?: (id: string) => void }) {
  const jobs = useJobs().filter((j) => j.kind === "vm-prepare" && isActive(j) && (!project || j.project === project));
  if (jobs.length === 0) return null;
  return (
    <div className="space-y-1.5" aria-live="polite">
      {jobs.map((j) => (
        <div
          key={j.id}
          role="status"
          aria-label={t("Preparazione della VM…")}
          className="flex flex-col gap-1 rounded-md border border-vsc-border bg-vsc-bg px-3 py-2"
        >
          <span className="flex items-center gap-2">
            <Codicon name="vm" size={16} className="text-vsc-pill-busy" />
            <span className="font-semibold">{t("Preparazione della VM…")}</span>
            <span className="min-w-0 flex-1 truncate text-vsc-desc">{j.title}</span>
            {onDetails && (
              <Button
                small
                variant="secondary"
                onClick={() => {
                  onDetails(j.id);
                }}
              >
                {t("Dettagli")}
              </Button>
            )}
          </span>
          <span className="truncate text-[12px] text-vsc-desc">{stepText(j)}</span>
          <JobBar percent={bytesPercent(j) ?? overallPercent(j)} label={j.title} />
        </div>
      ))}
    </div>
  );
}
