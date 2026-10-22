// The building blocks of the Guide's text: screenshots with a zoom, code
// blocks to copy, cards, the "in short" box, steps, tips, the folded
// technical part, links between topics and to the app's pages, flows,
// tables and quick folder openers.
import { createContext, useContext, useId, useRef, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { folderOpen, type AppInfo, type ComponentInfo } from "../api";
import { useCopy } from "../shared/hooks";
import { lang, t } from "../i18n";
import { Tag } from "../shared/tags";
import { Button, Codicon, codicon, DataTable, EditorSection, IconButton, Link, useModalFocus } from "../workbench";
import { sectionCode, topicOf, tr, type AppPageTarget, type GuideTopicId } from "./model";

/** A screenshot of the app from /guide/<lang>/*.webp (taken in the browser preview,
 * on its sample data) with a caption, an optional example and a zoom. */
export function GuideShot({
  src,
  title,
  caption,
  example,
  scenario = tr("dati di esempio", "sample data", "données d'exemple", "Beispieldaten"),
}: {
  src: string;
  title: string;
  caption: string;
  example?: ReactNode;
  scenario?: string;
}) {
  const [zoom, setZoom] = useState(false);

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
              <Codicon name="screen-full" size={12} />
              {tr("Ingrandisci", "Zoom", "Agrandir", "Vergrößern")}
            </span>
          </Button>
        </div>
        <button
          type="button"
          onClick={() => {
            setZoom(true);
          }}
          title={tr(
            "Clicca per ingrandire l'immagine",
            "Click to enlarge image",
            "Cliquer pour agrandir l'image",
            "Klicken, um das Bild zu vergrößern",
          )}
          className="group relative block w-full cursor-zoom-in overflow-hidden bg-black/20 text-left focus-visible:outline-1 focus-visible:outline-vsc-focus"
        >
          <img
            src={`/guide/${lang()}/${src}`}
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
                <Codicon name="info" size={14} className="text-vsc-info" />
                <span>
                  {tr(
                    "Esempio pratico nello screenshot",
                    "Practical example in screenshot",
                    "Exemple pratique dans la capture d'écran",
                    "Praktisches Beispiel im Screenshot",
                  )}
                </span>
              </div>
              <div className="leading-relaxed">{example}</div>
            </div>
          )}
        </figcaption>
      </figure>

      {zoom && (
        <ZoomDialog
          src={src}
          title={title}
          caption={caption}
          scenario={scenario}
          onClose={() => {
            setZoom(false);
          }}
        />
      )}
    </>
  );
}

/** The zoomed screenshot over the window: the focus goes to its close
 * button, Tab stays inside, Esc or a click outside closes it, and the focus
 * goes back to the button that opened it (useModalFocus). */
function ZoomDialog({
  src,
  title,
  caption,
  scenario,
  onClose,
}: {
  src: string;
  title: string;
  caption: string;
  scenario: string;
  onClose: () => void;
}) {
  const box = useRef<HTMLDivElement>(null);
  useModalFocus(box, onClose);
  return (
    <div
      ref={box}
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabIndex={-1}
      onClick={onClose}
      className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-black/80 p-4 outline-none"
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
          <IconButton icon={codicon("close")} label={t("Chiudi")} onClick={onClose} />
        </div>
        <div className="min-h-0 flex-1 overflow-auto bg-black/40 p-2">
          <img src={`/guide/${lang()}/${src}`} alt={title} className="mx-auto block max-w-full rounded" />
        </div>
        <div className="border-t border-vsc-border px-4 py-2 text-[12px] text-vsc-desc">{caption}</div>
      </div>
    </div>
  );
}

/** A code or configuration snippet with header bar and one-click copy. */
export function GuideCode({ title, path, code }: { title: string; path?: string; code: string }) {
  const { copied, copy } = useCopy(1500);
  return (
    <div className="overflow-hidden rounded border border-vsc-border bg-vsc-sidebar/60">
      <div className="flex items-center justify-between gap-2 border-b border-vsc-border bg-vsc-sidebar px-3 py-1 text-[12px]">
        <div className="flex min-w-0 items-center gap-2">
          <Codicon name="file-code" size={14} className="shrink-0 text-vsc-desc" />
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
            {copied ? <Codicon name="check" size={12} /> : <Codicon name="copy" size={12} />}
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
              <Codicon name="arrow-right" size={12} />
            </span>
          </Button>
        </div>
      )}
    </div>
  );
}

/** The opening box of a topic: what it is for and the few things to keep in
 * mind, before the details. */
export function InBrief({ points }: { points: ReactNode[] }) {
  return (
    <div className="rounded-md border border-vsc-focus/50 bg-vsc-badge/10 px-3.5 py-2.5 text-[12px]">
      <div className="mb-1 flex items-center gap-1.5 font-semibold text-vsc-settings-header">
        <Codicon name="lightbulb" size={14} className="text-vsc-focus" />
        {tr("In breve", "In short", "En bref", "Kurz gesagt")}
      </div>
      <ul className="m-0 list-disc space-y-1 pl-5 leading-relaxed">
        {points.map((p, i) => (
          <li key={i}>{p}</li>
        ))}
      </ul>
    </div>
  );
}

/** A paragraph of the guide's running text. */
export function P({ children }: { children: ReactNode }) {
  return <p className="m-0 text-[12px] leading-relaxed text-vsc-fg/90">{children}</p>;
}

/** Numbered steps: what to click, in order. */
export function Steps({ steps }: { steps: ReactNode[] }) {
  return (
    <ol className="m-0 list-decimal space-y-1.5 pl-5 text-[12px] leading-relaxed">
      {steps.map((s, i) => (
        <li key={i}>{s}</li>
      ))}
    </ol>
  );
}

/** A bulleted list of the running text. */
export function Bullets({ items }: { items: ReactNode[] }) {
  return (
    <ul className="m-0 list-disc space-y-1 pl-5 text-[12px] leading-relaxed">
      {items.map((s, i) => (
        <li key={i}>{s}</li>
      ))}
    </ul>
  );
}

const CALLOUT = {
  tip: { icon: codicon("lightbulb"), color: "text-vsc-success", border: "border-l-vsc-success" },
  info: { icon: codicon("info"), color: "text-vsc-info", border: "border-l-vsc-info" },
  warn: { icon: codicon("warning"), color: "text-vsc-warning", border: "border-l-vsc-warning" },
} as const;

/** A tip, a note or a warning beside the text. */
export function Callout({
  tone,
  title,
  children,
}: {
  tone: keyof typeof CALLOUT;
  title?: string;
  children: ReactNode;
}) {
  const { icon: Icon, color, border } = CALLOUT[tone];
  const fallback = {
    tip: tr("Consiglio", "Tip", "Conseil", "Tipp"),
    info: tr("Da sapere", "Good to know", "À savoir", "Gut zu wissen"),
    warn: tr("Attenzione", "Watch out", "Attention", "Achtung"),
  }[tone];
  return (
    <div className={`rounded border-l-2 ${border} bg-vsc-bg/60 px-3 py-2 text-[12px]`}>
      <div className="mb-0.5 flex items-center gap-1.5 font-semibold text-vsc-settings-header">
        <Icon size={14} aria-hidden="true" className={color} />
        {title ?? fallback}
      </div>
      <div className="leading-relaxed text-vsc-fg/90 select-text">{children}</div>
    </div>
  );
}

/** The technical side of a topic, folded: who wants it opens it. */
export function Under({ title, children }: { title?: string; children: ReactNode }) {
  return (
    <details className="group rounded-md border border-vsc-border bg-vsc-sidebar/35 text-[12px]">
      <summary className="flex cursor-pointer list-none items-center gap-1.5 px-3 py-2 font-semibold text-vsc-settings-header select-none">
        <Codicon name="chevron-right" size={14} className="transition-transform group-open:rotate-90" />
        <Codicon name="tools" size={14} className="text-vsc-desc" />
        {title ?? tr("Sotto il cofano", "Under the hood", "Sous le capot", "Unter der Haube")}
      </summary>
      <div className="space-y-2 border-t border-vsc-border px-3.5 py-2.5 leading-relaxed">{children}</div>
    </details>
  );
}

/** A label of the interface as the user sees it: «Avvia». */
export function Ui({ children }: { children: ReactNode }) {
  return <strong className="font-semibold text-vsc-settings-header">«{children}»</strong>;
}

/** A path, a file name, a command or a value to type. */
export function Code({ children }: { children: ReactNode }) {
  return (
    <code className="rounded bg-vsc-input px-1 py-px font-mono text-[11px] break-all text-vsc-fg select-text">
      {children}
    </code>
  );
}

/** A question and its answer (FAQ, problems). */
export function QA({ q, children }: { q: string; children: ReactNode }) {
  return (
    <div className="space-y-1 rounded-md border border-vsc-border bg-vsc-sidebar/35 px-3.5 py-2.5 text-[12px]">
      <div className="font-semibold text-vsc-settings-header">{q}</div>
      <div className="space-y-1.5 leading-relaxed text-vsc-fg/90">{children}</div>
    </div>
  );
}

/** A titled part of a topic. */
export function Section({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <EditorSection id={id} title={title}>
      <div className="space-y-3">{children}</div>
    </EditorSection>
  );
}

/** What a topic receives: the kit it describes, when known. */
export interface TopicProps {
  info: AppInfo | null;
  components: ComponentInfo[] | null;
}

/** The text of the topics of a chapter, by topic. */
export type TopicTexts = Partial<Record<GuideTopicId, (p: TopicProps) => ReactNode>>;

/** What a topic can do besides showing text: open another topic or a page. */
export interface GuideNav {
  select: (id: GuideTopicId) => void;
  open: (page: AppPageTarget) => void;
}

export const GuideNavContext = createContext<GuideNav>({
  select: () => undefined,
  open: () => undefined,
});

/** A link to another topic, named by its title unless told otherwise. */
export function See({ id, children }: { id: GuideTopicId; children?: ReactNode }) {
  const nav = useContext(GuideNavContext);
  return (
    <Link
      onClick={() => {
        nav.select(id);
      }}
    >
      {children ?? `${topicOf(id).title()} (${sectionCode(topicOf(id))})`}
    </Link>
  );
}

/** "Read next": the topics close to this one. */
export function Related({ ids }: { ids: GuideTopicId[] }) {
  return (
    <div className="rounded-md border border-vsc-border bg-vsc-sidebar/35 px-3.5 py-2.5 text-[12px]">
      <div className="mb-1 font-semibold text-vsc-settings-header">
        {tr("Argomenti collegati", "Related topics", "Sujets liés", "Verwandte Themen")}
      </div>
      <ul className="m-0 list-none space-y-0.5 p-0">
        {ids.map((id) => (
          <li key={id} className="flex items-center gap-1.5">
            <Codicon name="arrow-right" size={12} className="text-vsc-desc" />
            <See id={id} />
          </li>
        ))}
      </ul>
    </div>
  );
}

/** A button that opens a page of the app. */
export function Go({ page, children }: { page: AppPageTarget; children: ReactNode }) {
  const nav = useContext(GuideNavContext);
  return (
    <Button
      small
      variant="secondary"
      onClick={() => {
        nav.open(page);
      }}
    >
      <span className="inline-flex items-center gap-1">
        {children}
        <Codicon name="arrow-right" size={12} />
      </span>
    </Button>
  );
}

/** A chain of steps drawn as boxes and arrows: how something flows. */
export function Flow({ steps }: { steps: { title: string; text: string }[] }) {
  return (
    <ol className="m-0 flex list-none flex-wrap items-stretch gap-1.5 p-0 text-[12px]">
      {steps.map((s, i) => (
        <li key={s.title} className="flex items-stretch gap-1.5">
          <div className="flex w-40 flex-col gap-0.5 rounded-md border border-vsc-border bg-vsc-sidebar/50 px-2.5 py-2">
            <div className="flex items-center gap-1.5 font-semibold text-vsc-settings-header">
              <span className="flex size-4 shrink-0 items-center justify-center rounded-full bg-vsc-badge text-[10px] text-vsc-badge-fg">
                {i + 1}
              </span>
              {s.title}
            </div>
            <div className="leading-snug text-vsc-desc">{s.text}</div>
          </div>
          {i < steps.length - 1 && (
            <Codicon name="arrow-right" size={14} className="shrink-0 self-center text-vsc-desc" />
          )}
        </li>
      ))}
    </ol>
  );
}

/** A plain table of the running text. */
export function Table({ columns, rows }: { columns: string[]; rows: ReactNode[][] }) {
  return (
    <div className="overflow-x-auto rounded border border-vsc-border">
      <DataTable columns={columns} className="w-full border-collapse text-[12px]">
        {rows.map((r, i) => (
          <tr key={i} className="border-t border-vsc-border hover:bg-vsc-list-hover/40">
            {r.map((c, j) => (
              <td key={j} className="px-2.5 py-2 align-top leading-relaxed">
                {c}
              </td>
            ))}
          </tr>
        ))}
      </DataTable>
    </div>
  );
}

/** A key of the keyboard: Ctrl, Invio. */
export function Key({ children }: { children: ReactNode }) {
  return (
    <kbd className="rounded border border-b-2 border-vsc-keybinding-border border-b-vsc-keybinding-bottom bg-vsc-keybinding px-1 font-mono text-[11px] text-vsc-keybinding-fg">
      {children}
    </kbd>
  );
}

/** A labelled group of cards, two per row on wide windows. */
export function Cards({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-1 gap-3 md:grid-cols-2">{children}</div>;
}

/** Quick buttons to open the kit's key folders in Windows Explorer. */
export function KitFoldersBar() {
  const items: { id: "projects" | "agents" | "db" | "backup"; label: string; path: string }[] = [
    {
      id: "projects",
      label: tr("Cartella progetti", "Projects folder", "Dossier des projets", "Projektordner"),
      path: "projects\\",
    },
    {
      id: "agents",
      label: tr("Agenti condivisi", "Shared agents", "Agents partagés", "Geteilte Agenten"),
      path: "data\\agents\\",
    },
    {
      id: "db",
      label: tr("Dati database", "Database data", "Données de bases de données", "Datenbankdaten"),
      path: "data\\db\\",
    },
    {
      id: "backup",
      label: tr("Backup database", "Database backups", "Sauvegardes de bases de données", "Datenbank-Backups"),
      path: tr(
        "data\\projects\\<progetto>\\private\\backups\\",
        "data\\projects\\<project>\\private\\backups\\",
        "data\\projects\\<projet>\\private\\backups\\",
        "data\\projects\\<projekt>\\private\\backups\\",
      ),
    },
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
            <Codicon name="folder-opened" size={12} />
            <span>{x.label}</span>
            <code className="font-mono text-[11px] opacity-75">{x.path}</code>
          </span>
        </Button>
      ))}
    </div>
  );
}
