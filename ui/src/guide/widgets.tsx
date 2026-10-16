// Reusable UI components for the in-app Guide: screenshot cards with
// full-screen zoom modal, copyable code/config blocks, step-by-step recipe
// cards and quick folder openers.
import { useEffect, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  VscArrowRight,
  VscCheck,
  VscClose,
  VscCopy,
  VscFileCode,
  VscFolderOpened,
  VscInfo,
  VscScreenFull,
} from "react-icons/vsc";
import { folderOpen } from "../api";
import { useCopy } from "../hooks";
import { t } from "../i18n";
import { Tag } from "../tags";
import { Button, IconButton } from "../workbench";
import { tr, type AppPageTarget } from "./model";

/** An application screenshot from /guide/*.webp with caption, practical
 * example callout and click-to-zoom modal. */
export function GuideShot({
  src,
  title,
  caption,
  example,
  scenario = "D:\\Atollo · @mario-rossi",
}: {
  src: string;
  title: string;
  caption: string;
  example?: ReactNode;
  scenario?: string;
}) {
  const [zoom, setZoom] = useState(false);
  useEffect(() => {
    if (!zoom) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setZoom(false);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [zoom]);

  return (
    <>
      <figure className="m-0 overflow-hidden rounded-md border border-vsc-border bg-vsc-sidebar/50">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-vsc-border bg-vsc-sidebar px-3 py-1.5 text-[12px]">
          <div className="flex min-w-0 items-center gap-2">
            <span className="truncate font-semibold text-vsc-settings-header">{title}</span>
            <Tag>{scenario}</Tag>
          </div>
          <Button
            small
            variant="secondary"
            onClick={() => {
              setZoom(true);
            }}
          >
            <span className="inline-flex items-center gap-1">
              <VscScreenFull size={12} aria-hidden="true" />
              {tr("Ingrandisci", "Zoom", "Agrandir", "Vergrößern")}
            </span>
          </Button>
        </div>
        <button
          type="button"
          onClick={() => {
            setZoom(true);
          }}
          title={tr("Clicca per ingrandire l'immagine", "Click to enlarge image")}
          className="group relative block w-full cursor-zoom-in overflow-hidden bg-black/20 text-left focus-visible:outline-1 focus-visible:outline-vsc-focus"
        >
          <img
            src={`/guide/${src}`}
            alt={title}
            loading="lazy"
            className="block max-h-[420px] w-full object-contain object-top transition-transform duration-150 group-hover:scale-[1.005]"
          />
        </button>
        <figcaption className="space-y-1.5 border-t border-vsc-border px-3.5 py-2.5 text-[12px]">
          <p className="m-0 text-vsc-desc">{caption}</p>
          {example && (
            <div className="rounded border-l-2 border-l-vsc-focus bg-vsc-bg/70 px-3 py-2 text-vsc-fg">
              <div className="mb-0.5 flex items-center gap-1.5 font-semibold text-vsc-settings-header">
                <VscInfo size={14} aria-hidden="true" className="text-vsc-info" />
                <span>{tr("Esempio pratico nello screenshot", "Practical example in screenshot")}</span>
              </div>
              <div className="leading-relaxed">{example}</div>
            </div>
          )}
        </figcaption>
      </figure>

      {zoom && (
        <div
          role="dialog"
          aria-modal="true"
          aria-label={title}
          onClick={() => {
            setZoom(false);
          }}
          className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-black/80 p-4"
        >
          <div
            onClick={(e) => {
              e.stopPropagation();
            }}
            className="flex max-h-[92vh] w-[min(1240px,95vw)] flex-col overflow-hidden rounded-lg border border-vsc-widget-border bg-vsc-widget shadow-[0_0_16px_4px_var(--vsc-shadow)]"
          >
            <div className="flex items-center justify-between gap-2 border-b border-vsc-border px-4 py-2">
              <div className="flex min-w-0 items-center gap-2">
                <span className="truncate font-semibold text-vsc-settings-header">{title}</span>
                <Tag>{scenario}</Tag>
              </div>
              <IconButton
                icon={VscClose}
                label={t("Chiudi")}
                onClick={() => {
                  setZoom(false);
                }}
              />
            </div>
            <div className="min-h-0 flex-1 overflow-auto bg-black/40 p-2">
              <img src={`/guide/${src}`} alt={title} className="mx-auto block max-w-full rounded" />
            </div>
            <div className="border-t border-vsc-border px-4 py-2 text-[12px] text-vsc-desc">{caption}</div>
          </div>
        </div>
      )}
    </>
  );
}

/** A code or configuration snippet with header bar and one-click copy. */
export function GuideCode({ title, path, code }: { title: string; path?: string; code: string }) {
  const { copied, copy } = useCopy(1500);
  return (
    <div className="overflow-hidden rounded border border-vsc-border bg-vsc-sidebar/60">
      <div className="flex items-center justify-between gap-2 border-b border-vsc-border bg-vsc-sidebar px-3 py-1 text-[12px]">
        <div className="flex min-w-0 items-center gap-2">
          <VscFileCode size={14} aria-hidden="true" className="shrink-0 text-vsc-desc" />
          <span className="truncate font-semibold text-vsc-settings-header">{title}</span>
          {path && <code className="truncate font-mono text-[11px] text-vsc-desc">{path}</code>}
        </div>
        <Button
          small
          variant="secondary"
          onClick={() => {
            copy(code);
          }}
        >
          <span className="inline-flex items-center gap-1">
            {copied ? <VscCheck size={12} aria-hidden="true" /> : <VscCopy size={12} aria-hidden="true" />}
            {copied ? t("Copiata") : t("Copia")}
          </span>
        </Button>
      </div>
      <pre className="m-0 overflow-x-auto p-3 font-mono text-[12px] leading-relaxed select-text">{code}</pre>
    </div>
  );
}

/** An architectural principle or feature card. */
export function GuideCard({
  icon: Icon,
  badge,
  title,
  children,
  actionLabel,
  onAction,
}: {
  icon: IconType;
  badge?: string;
  title: string;
  children: ReactNode;
  actionLabel?: string;
  onAction?: () => void;
}) {
  return (
    <div className="flex flex-col justify-between gap-2.5 rounded-md border border-vsc-border bg-vsc-sidebar/40 p-3.5">
      <div className="space-y-1.5">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <span className="flex size-6 shrink-0 items-center justify-center rounded bg-vsc-badge/20 text-vsc-focus">
              <Icon size={16} aria-hidden="true" />
            </span>
            <h4 className="m-0 text-[13px] font-semibold text-vsc-settings-header">{title}</h4>
          </div>
          {badge && <Tag tone="focus">{badge}</Tag>}
        </div>
        <div className="text-[12px] leading-relaxed text-vsc-fg/90">{children}</div>
      </div>
      {actionLabel && onAction && (
        <div className="pt-1">
          <Button small variant="secondary" onClick={onAction}>
            <span className="inline-flex items-center gap-1">
              {actionLabel}
              <VscArrowRight size={12} aria-hidden="true" />
            </span>
          </Button>
        </div>
      )}
    </div>
  );
}

/** Step-by-step configuration procedure card with "Dove si configura",
 * numbered steps, "Scopo tecnico e pratico" and direct navigation button. */
export function GuideRecipe({
  number,
  title,
  whereUi,
  whereFile,
  steps,
  purpose,
  targetPage,
  targetLabel,
  onNavigate,
  children,
}: {
  number: string;
  title: string;
  whereUi: string;
  whereFile: string;
  steps: ReactNode[];
  purpose: ReactNode;
  targetPage?: AppPageTarget;
  targetLabel?: string;
  onNavigate?: (page: AppPageTarget) => void;
  children?: ReactNode;
}) {
  return (
    <div className="space-y-3 rounded-md border border-vsc-border bg-vsc-sidebar/35 p-4">
      <div className="flex flex-wrap items-start justify-between gap-2 border-b border-vsc-border pb-2.5">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <Tag tone="focus">{number}</Tag>
            <h4 className="m-0 text-[14px] font-semibold text-vsc-settings-header">{title}</h4>
          </div>
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px] text-vsc-desc">
            <span>
              <strong>{tr("Dove si configura:", "Where to configure:")}</strong> {whereUi}
            </span>
            <span>·</span>
            <span>
              <strong>{tr("File su disco:", "File on disk:")}</strong>{" "}
              <code className="rounded bg-vsc-input px-1.5 py-0.5 font-mono text-[11px] text-vsc-fg select-text">
                {whereFile}
              </code>
            </span>
          </div>
        </div>
        {targetPage && onNavigate && (
          <Button
            small
            variant="primary"
            onClick={() => {
              onNavigate(targetPage);
            }}
          >
            <span className="inline-flex items-center gap-1">
              {targetLabel ?? tr("Configura ora", "Configure now")}
              <VscArrowRight size={12} aria-hidden="true" />
            </span>
          </Button>
        )}
      </div>

      <div className="space-y-1.5 text-[12px]">
        <div className="font-semibold text-vsc-settings-header">
          {tr("Passaggi operativi:", "Step-by-step procedure:")}
        </div>
        <ol className="m-0 list-decimal space-y-1.5 pl-5 leading-relaxed">
          {steps.map((s, i) => (
            <li key={i}>{s}</li>
          ))}
        </ol>
      </div>

      <div className="rounded border-l-2 border-l-vsc-focus bg-vsc-bg/60 px-3 py-2 text-[12px]">
        <div className="mb-0.5 font-semibold text-vsc-settings-header">
          {tr("Scopo tecnico e perché va fatto:", "Technical purpose & why it matters:")}
        </div>
        <div className="leading-relaxed text-vsc-fg/90">{purpose}</div>
      </div>

      {children}
    </div>
  );
}

/** Quick buttons to open the kit's key folders in Windows Explorer. */
export function KitFoldersBar() {
  const items: { id: "projects" | "agents" | "db" | "backup"; label: string; path: string }[] = [
    { id: "projects", label: tr("Cartella progetti", "Projects folder"), path: "projects\\" },
    { id: "agents", label: tr("Agenti condivisi", "Shared agents"), path: "data\\agents\\" },
    { id: "db", label: tr("Dati database", "Database data"), path: "data\\db\\" },
    { id: "backup", label: tr("Backup database", "Database backups"), path: "data\\backup\\" },
  ];
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {items.map((x) => (
        <Button
          key={x.id}
          small
          variant="secondary"
          title={`${x.label} (${x.path})`}
          onClick={() => {
            void folderOpen(x.id);
          }}
        >
          <span className="inline-flex items-center gap-1">
            <VscFolderOpened size={12} aria-hidden="true" />
            <span>{x.label}</span>
            <code className="font-mono text-[10px] opacity-75">{x.path}</code>
          </span>
        </Button>
      ))}
    </div>
  );
}
