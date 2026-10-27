// The status bar's entries shared by its items (App.tsx): the entry
// itself, as VSCodium's label, the VM's pill and the jobs' item
// (LEVELS.md §8.6).
import { useEffect, useState, type ReactNode } from "react";
import type { Job } from "../ipc";
import { OPEN_VM_NOTICE } from "../shared/levels";
import { IsolationDialog, useIsolation, vmPill, VmPillBadge } from "./isolation";
import { t } from "../i18n";
import { Icon } from "../workbench";
import { activeJobs, bytesPercent, JobBar, overallPercent, stepText } from "./jobs";

/** A status bar entry, as VSCodium's label: the bar's height, 5px beside
 * the text and 3px apart, rounded, a hover and a pressed state; a button
 * when it acts. `last` is the right edge's item, 6px beside the text and 2px
 * from the edge. `tone` paints it as the editor's error or warning items. */
export function StatusItem({
  children,
  title,
  onClick,
  className = "",
  pressed,
  tone,
  last = false,
  ariaLabel,
}: {
  children: ReactNode;
  title?: string;
  /** The button's name when its text is not enough. */
  ariaLabel?: string;
  onClick?: () => void;
  className?: string;
  pressed?: boolean;
  tone?: "error" | "warning" | undefined;
  last?: boolean;
}) {
  const color =
    tone === "error"
      ? "bg-vsc-statusbar-error text-vsc-statusbar-error-fg"
      : tone === "warning"
        ? "bg-vsc-statusbar-warning text-vsc-statusbar-warning-fg"
        : "";
  const space = last ? "mr-vsc-size20 px-vsc-size60" : "mx-vsc-statusbar-item-margin px-vsc-statusbar-item-padding";
  const look = `flex h-full items-center gap-vsc-size40 rounded-vsc-small whitespace-nowrap ${space} ${color} ${className}`;
  return onClick ? (
    <button
      type="button"
      title={title}
      aria-label={ariaLabel}
      aria-pressed={pressed}
      onClick={onClick}
      className={[
        look,
        "focus-visible:outline-vsc-statusbar-fg",
        tone ? "hover:brightness-110" : pressed ? "bg-vsc-statusbar-active" : "hover:bg-vsc-statusbar-hover",
      ].join(" ")}
    >
      {children}
    </button>
  ) : (
    <span title={title} className={look}>
      {children}
    </span>
  );
}

/** The long work going on (LEVELS.md §8.6), always in sight while it
 * lasts: the main job's title, its step and a small bar with its percent
 * (going across when not known); with more, «3 lavori in corso» and the
 * main one. A click opens it in the Activity panel, its steps and errors. */
export function JobsItem({ jobs, onJob }: { jobs: Job[]; onJob: (id: string) => void }) {
  const going = activeJobs(jobs);
  const main = going[0];
  if (!main) return null;
  const percent = main.state === "running" ? (bytesPercent(main) ?? overallPercent(main)) : null;
  const count = going.length > 1 ? t("{0} lavori in corso", going.length) : null;
  const step = stepText(main);
  const said = [count, main.title, step, percent === null ? null : `${String(percent)}%`].filter(Boolean).join(" · ");
  return (
    <StatusItem
      title={`${said}
${t("Dettagli in Attività")}`}
      ariaLabel={`${said}. ${t("Dettagli in Attività")}`}
      onClick={() => {
        onJob(main.id);
      }}
      className="max-w-[480px] min-w-0"
    >
      {/* The item's color, as the editor's status-bar codicons. */}
      <Icon name="loading" spin size={14} className="[--vscode-icon-foreground:currentColor]" />
      {count && <span className="font-semibold">{count}:</span>}
      <span className="min-w-0 truncate">
        {main.title}
        <span className="opacity-80"> · {step}</span>
      </span>
      <JobBar percent={percent} label={main.title} className="w-[56px] shrink-0" />
      {percent !== null && <span className="tabular-nums">{percent}%</span>}
      <span role="status" className="sr-only">
        {said}
      </span>
    </StatusItem>
  );
}

/** How shut in the projects are (D-167), right after the Activity item,
 * as a pill (LEVELS.md §8.6): green «VM» ready, blue «VM…» with its percent
 * while a VM is prepared, orange «Senza VM» when it can be fixed, grey when
 * this PC cannot have it, red on an error; always in sight, its state said
 * in its name and tooltip. A click opens why and what to do. */
export function VmItem({ jobs }: { jobs: Job[] }) {
  const { status, checking, check } = useIsolation();
  const [open, setOpen] = useState(false);
  // A refused command's «Apri l'avviso VM» (D-179, shared/levels.tsx).
  useEffect(() => {
    const show = () => {
      setOpen(true);
    };
    window.addEventListener(OPEN_VM_NOTICE, show);
    return () => {
      window.removeEventListener(OPEN_VM_NOTICE, show);
    };
  }, []);
  if (!status) return null;
  // The pill (LEVELS.md §8.6): its color says the state at a glance, its
  // text and name say it in words.
  const pill = vmPill(status, jobs);
  return (
    <>
      <StatusItem
        title={`${pill.label}
${pill.todo}`}
        ariaLabel={`${pill.label}. ${pill.todo}`}
        onClick={() => {
          setOpen(true);
        }}
        pressed={open}
      >
        <VmPillBadge pill={pill} />
        <span role="status" className="sr-only">
          {pill.label}
        </span>
      </StatusItem>
      {open && (
        <IsolationDialog
          status={status}
          checking={checking}
          onCheck={check}
          onClose={() => {
            setOpen(false);
          }}
        />
      )}
    </>
  );
}
