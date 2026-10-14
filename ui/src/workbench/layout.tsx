// VSCodium's parts and views (workbench/browser/parts): the side bar with
// its title and collapsible panes, the sash between parts, the editor with
// its tabs, the header and navigation bar of a details editor (as an
// extension's page), the panel, and the welcome view of an empty view.
// Parts float 4px apart on the workbench color, as VSCodium 1.140's modern
// layout; the app composes them per page (ViewContainer).
import { useCallback, useRef, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { VscChevronRight, VscClose } from "react-icons/vsc";
import { t } from "../i18n";
import { ActionBar, CountBadge } from "./controls";

/** A floating part: rounded, its own border, its own background. */
export function Part({
  as: Tag = "div",
  id,
  label,
  className = "",
  style,
  children,
}: {
  as?: "div" | "aside" | "main" | "section";
  id?: string;
  label?: string | undefined;
  className?: string;
  style?: React.CSSProperties;
  children: ReactNode;
}) {
  return (
    <Tag
      id={id}
      tabIndex={id ? -1 : undefined}
      aria-label={label}
      style={style}
      className={`flex min-h-0 min-w-0 flex-col overflow-hidden rounded-lg border border-vsc-surface-border ${className}`}
    >
      {children}
    </Tag>
  );
}

function readSize(key: string, fallback: number): number {
  try {
    const v = Number(localStorage.getItem(`size:${key}`));
    return Number.isFinite(v) && v > 0 ? v : fallback;
  } catch {
    return fallback;
  }
}

/** A size the user drags with a sash, remembered in this browser. */
export function useStoredSize(key: string, fallback: number, min: number, max: number) {
  const [size, setSize] = useState(() => Math.min(max, Math.max(min, readSize(key, fallback))));
  const set = useCallback(
    (v: number) => {
      const next = Math.round(Math.min(max, Math.max(min, v)));
      setSize(next);
      try {
        localStorage.setItem(`size:${key}`, String(next));
      } catch {
        // Not remembered: the size still changes.
      }
    },
    [key, min, max],
  );
  return [size, set] as const;
}

/** The sash between two parts: drag to resize, double click to reset;
 * arrows when focused. `delta` is the movement in pixels. */
export function Sash({
  vertical,
  label,
  onDrag,
  onReset,
}: {
  /** Between parts side by side (drags left-right). */
  vertical: boolean;
  label: string;
  onDrag: (delta: number) => void;
  onReset: () => void;
}) {
  const start = useRef<number | null>(null);
  return (
    <div
      role="separator"
      aria-label={label}
      aria-orientation={vertical ? "vertical" : "horizontal"}
      tabIndex={0}
      onPointerDown={(e) => {
        start.current = vertical ? e.clientX : e.clientY;
        e.currentTarget.setPointerCapture(e.pointerId);
      }}
      onPointerMove={(e) => {
        if (start.current === null) return;
        const at = vertical ? e.clientX : e.clientY;
        onDrag(at - start.current);
        start.current = at;
      }}
      onPointerUp={() => {
        start.current = null;
      }}
      onDoubleClick={onReset}
      onKeyDown={(e) => {
        const step = e.shiftKey ? 40 : 10;
        const keys = vertical ? { ArrowLeft: -step, ArrowRight: step } : { ArrowUp: -step, ArrowDown: step };
        const d = (keys as Record<string, number>)[e.key];
        if (d !== undefined) {
          e.preventDefault();
          onDrag(d);
        }
      }}
      className={[
        "relative z-20 shrink-0 outline-none",
        // 4px wide, as the gap; the line shows on hover and focus after a delay.
        vertical ? "-mx-1 w-1 cursor-ew-resize" : "-my-1 h-1 cursor-ns-resize",
        "before:absolute before:bg-transparent before:transition-colors before:delay-300 hover:before:bg-vsc-focus focus-visible:before:bg-vsc-focus",
        vertical
          ? "before:inset-y-0 before:left-1/2 before:w-1 before:-translate-x-1/2"
          : "before:inset-x-0 before:top-1/2 before:h-1 before:-translate-y-1/2",
      ].join(" ")}
    />
  );
}

/** A page: the side bar (if any) and the editor, with the sash between. */
export function ViewContainer({ side, id, children }: { side?: ReactNode; id: string; children: ReactNode }) {
  const [width, setWidth] = useStoredSize(`side:${id}`, 300, 170, 640);
  return (
    <div className="flex min-h-0 min-w-0 flex-1 gap-1">
      {side && (
        <>
          <Part as="aside" className="shrink-0 bg-vsc-sidebar text-vsc-sidebar-fg" style={{ width }}>
            {side}
          </Part>
          <Sash
            vertical
            label={t("Ridimensiona la barra laterale")}
            onDrag={(d) => {
              setWidth(width + d);
            }}
            onReset={() => {
              setWidth(300);
            }}
          />
        </>
      )}
      <Part as="main" id="content" className="flex-1 bg-vsc-bg outline-none">
        {children}
      </Part>
    </div>
  );
}

/** The side bar's title (Explorer): the view container's name and actions;
 * 32px, 12px semibold, the label 12px from the edge (modern UI). */
export function SideBarTitle({ title, actions }: { title: string; actions?: ReactNode }) {
  return (
    <div className="group/title flex h-8 shrink-0 items-center px-1">
      <h1 className="min-w-0 flex-1 truncate pl-2 text-[12px] leading-8 font-semibold text-vsc-sidebar-title capitalize">
        {title}
      </h1>
      {actions && <ActionBar className="pl-1">{actions}</ActionBar>}
    </div>
  );
}

/** The panes of a side bar, one over the other, 4px in from the sides;
 * `fill` panes share the rest. */
export function PaneStack({ children }: { children: ReactNode }) {
  return <div className="flex min-h-0 flex-1 flex-col px-1 pb-1">{children}</div>;
}

/** A collapsible pane (Outline, Timeline): a 28px rounded header, 12px
 * semibold, a line 4px in from the sides above all but the first, its
 * actions on hover, a badge, the body scrolling inside (modern UI). */
export function ViewPane({
  title,
  expanded,
  onToggle,
  fill = false,
  first = false,
  actions,
  badge,
  children,
}: {
  title: string;
  expanded: boolean;
  onToggle: () => void;
  /** Takes the height left by the others when open. */
  fill?: boolean;
  /** No border on top: the first pane of the side bar. */
  first?: boolean;
  actions?: ReactNode;
  badge?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section
      className={`flex min-h-7 flex-col ${expanded && fill ? "flex-1 basis-0" : "shrink-0"} ${
        expanded && !fill ? "max-h-[60%]" : ""
      }`}
    >
      <h3
        className={`group/pane relative flex h-7 shrink-0 items-center rounded text-vsc-section-fg hover:bg-vsc-list-hover ${
          first ? "" : "before:absolute before:inset-x-1 before:top-0 before:h-px before:bg-vsc-section-border"
        }`}
      >
        <button
          type="button"
          aria-expanded={expanded}
          onClick={onToggle}
          className="flex h-full min-w-0 flex-1 items-center gap-0.5 rounded pl-0.5 text-left text-[12px] font-semibold capitalize outline-none focus-visible:outline-1 focus-visible:-outline-offset-2 focus-visible:outline-vsc-focus"
        >
          <VscChevronRight
            size={16}
            aria-hidden="true"
            className={`shrink-0 transition-transform duration-100 ${expanded ? "rotate-90" : ""}`}
          />
          <span className="truncate">{title}</span>
        </button>
        {badge !== undefined && badge !== null && <CountBadge className="mx-1">{badge}</CountBadge>}
        {actions && expanded && (
          <ActionBar className="invisible ml-1 gap-1 pr-1 group-focus-within/pane:visible group-hover/pane:visible">
            {actions}
          </ActionBar>
        )}
      </h3>
      {expanded && <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>}
    </section>
  );
}

/** What an empty view shows (viewsWelcome): text and full-width buttons,
 * at most 300px wide. */
export function WelcomeView({ children }: { children: ReactNode }) {
  return <div className="max-w-[300px] space-y-3 px-5 py-2.5 [&_p]:leading-[1.4]">{children}</div>;
}

export interface Tab {
  id: string;
  label: string;
  icon?: ReactNode;
  /** A description after the label, as the folder of two files with one name. */
  description?: string;
  title?: string;
}

/** The tab bar of the modern UI: 32px, on the editor color, no border. */
const TAB_BAR = "flex h-8 shrink-0 pl-0.5";

/** A tab: its fill is a 24px pill 2px in from the sides, rounded; the active
 * one on the list's selection color, the others on hover (modern UI). */
const TAB = "group/tab relative flex h-full max-w-60 min-w-0 shrink-0 items-center before:absolute before:inset-x-0.5 before:inset-y-1 before:rounded";

/** The editor's tabs (modern UI): pills, the active one filled; close on
 * hover and on the active tab. */
export function EditorTabs({
  tabs,
  active,
  onSelect,
  onClose,
  actions,
}: {
  tabs: Tab[];
  active: string | null;
  onSelect: (id: string) => void;
  onClose?: (id: string) => void;
  actions?: ReactNode;
}) {
  return (
    <div className={TAB_BAR}>
      <div role="tablist" aria-label="Editor" className="flex min-w-0 flex-1 overflow-x-auto [scrollbar-width:none]">
        {tabs.map((tab) => {
          const on = tab.id === active;
          return (
            <div
              key={tab.id}
              className={`${TAB} ${
                on
                  ? "text-vsc-tab-active-fg before:bg-vsc-list-inactive"
                  : "text-vsc-tab-inactive-fg hover:before:bg-vsc-list-hover"
              }`}
            >
              <button
                type="button"
                role="tab"
                aria-selected={on}
                title={tab.title ?? tab.label}
                onClick={() => {
                  onSelect(tab.id);
                }}
                onAuxClick={(e) => {
                  if (e.button === 1 && onClose) onClose(tab.id);
                }}
                className={`relative flex h-full min-w-0 items-center gap-1.5 pl-1.5 outline-none focus-visible:outline-1 focus-visible:-outline-offset-4 focus-visible:outline-vsc-focus ${onClose ? "pr-1" : "pr-2"}`}
              >
                {tab.icon && <span className="flex w-4 shrink-0 justify-center">{tab.icon}</span>}
                <span className="truncate">{tab.label}</span>
                {tab.description && <span className="truncate text-[12px] opacity-70">{tab.description}</span>}
              </button>
              {onClose && (
                <button
                  type="button"
                  aria-label={t("Chiudi {0}", tab.label)}
                  title={t("Chiudi")}
                  onClick={() => {
                    onClose(tab.id);
                  }}
                  className={`relative mr-1 flex size-5 items-center justify-center rounded hover:bg-vsc-toolbar-hover ${
                    on ? "" : "invisible group-hover/tab:visible focus-visible:visible"
                  }`}
                >
                  <VscClose size={16} aria-hidden="true" />
                </button>
              )}
            </div>
          );
        })}
      </div>
      {actions && <ActionBar className="px-1">{actions}</ActionBar>}
    </div>
  );
}

/** An editor with one fixed tab, for a page without a side bar: the tab
 * names the page (its h1). */
export function SingleEditor({ title, icon, children }: { title: string; icon: ReactNode; children: ReactNode }) {
  return (
    <ViewContainer id={title}>
      <div className={TAB_BAR}>
        <div className={`${TAB} gap-1.5 pr-2 pl-1.5 text-vsc-tab-active-fg before:bg-vsc-list-inactive`}>
          <span className="relative flex w-4 justify-center">{icon}</span>
          <h1 className="relative text-[13px] font-normal">{title}</h1>
        </div>
      </div>
      {children}
    </ViewContainer>
  );
}

/** The header of a details editor, as an extension's page: a big icon, the
 * name, a line of facts, a description, the buttons. */
export function EditorHeader({
  icon,
  title,
  titleId,
  meta,
  description,
  actions,
}: {
  icon: ReactNode;
  title: string;
  titleId?: string;
  /** Facts after the name, separated by bars. */
  meta?: ReactNode[];
  description?: ReactNode;
  actions?: ReactNode;
}) {
  const facts = (meta ?? []).filter(Boolean);
  return (
    <header className="flex shrink-0 gap-5 px-5 pt-5 pb-3">
      <span className="flex size-16 shrink-0 items-center justify-center">{icon}</span>
      <div className="min-w-0 flex-1 space-y-1.5">
        <h2 id={titleId} className="truncate text-[26px] leading-8 font-semibold text-vsc-settings-header select-text">
          {title}
        </h2>
        {facts.length > 0 && (
          <div className="flex flex-wrap items-center text-[13px]">
            {facts.map((f, i) => (
              <span
                key={i}
                className={`flex items-center gap-1.5 ${i > 0 ? "ml-2.5 border-l border-vsc-border pl-2.5" : ""}`}
              >
                {f}
              </span>
            ))}
          </div>
        )}
        {description && <div className="text-vsc-desc">{description}</div>}
        {actions && <div className="flex flex-wrap items-center gap-1.5 pt-1">{actions}</div>}
      </div>
    </header>
  );
}

export interface NavItem<T extends string> {
  id: T;
  label: string;
  icon?: IconType;
  /** A mark after the label: a warning, a running dot, a count. */
  mark?: ReactNode;
}

/** The navigation bar of a details editor (Details, Features, Changelog):
 * the active item underlined. */
export function NavBar<T extends string>({
  label,
  items,
  active,
  onSelect,
}: {
  label: string;
  items: NavItem<T>[];
  active: T;
  onSelect: (id: T) => void;
}) {
  return (
    <nav aria-label={label} className="shrink-0 border-b border-vsc-border px-5">
      <ul className="m-0 flex list-none flex-wrap gap-x-5 p-0">
        {items.map((x) => {
          const on = x.id === active;
          const Icon = x.icon;
          return (
            <li key={x.id}>
              <button
                type="button"
                aria-current={on ? "page" : undefined}
                onClick={() => {
                  onSelect(x.id);
                }}
                className={`relative flex h-[30px] items-center gap-1.5 outline-none focus-visible:outline-1 focus-visible:outline-vsc-focus ${
                  on
                    ? "text-vsc-panel-title after:absolute after:inset-x-0 after:bottom-0 after:h-px after:bg-vsc-panel-title-border"
                    : "text-vsc-panel-title-inactive hover:text-vsc-panel-title"
                }`}
              >
                {Icon && <Icon size={16} aria-hidden="true" />}
                {x.label}
                {x.mark}
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

/** A titled block inside an editor, as the sections of an extension's
 * Features: 13px semibold title, a line under it. */
export function EditorSection({
  title,
  id,
  actions,
  fill = false,
  children,
}: {
  title: string;
  id: string;
  actions?: ReactNode;
  /** Takes the height left in a `fill` EditorBody, its content scrolling inside. */
  fill?: boolean;
  children: ReactNode;
}) {
  return (
    <section aria-labelledby={id} className={fill ? "flex min-h-24 flex-1 flex-col gap-2" : "space-y-2"}>
      <div className="flex min-h-[26px] shrink-0 items-center gap-2 border-b border-vsc-border pb-1">
        <h3 id={id} className="min-w-0 flex-1 text-[13px] font-semibold text-vsc-settings-header">
          {title}
        </h3>
        {actions}
      </div>
      {/* -mx-2: list rows reach past the title's edges, as outside a scroll. */}
      {fill ? <div className="-mx-2 min-h-0 flex-1 overflow-y-auto">{children}</div> : children}
    </section>
  );
}

/** Label and value pairs, as an extension's "More Info". */
export function Properties({ rows }: { rows: [string, ReactNode][] }) {
  return (
    <dl className="m-0 grid grid-cols-[max-content_minmax(0,1fr)] items-center gap-x-6 gap-y-1.5">
      {rows.map(([k, v]) => (
        <div key={k} className="contents">
          <dt className="text-vsc-desc">{k}</dt>
          <dd className="m-0 min-w-0">{v}</dd>
        </div>
      ))}
    </dl>
  );
}

/** The editor's empty state: the brand watermark and the next steps. */
export function Watermark({ children }: { children: ReactNode }) {
  return <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-6 p-6">{children}</div>;
}

/** The panel at the bottom (Problems, Output, Terminal): its 32px title bar
 * with the tabs, the active one underlined, and its actions. */
export function Panel({
  tabs,
  active,
  onSelect,
  actions,
  height,
  children,
}: {
  tabs: { id: string; label: string; badge?: ReactNode }[];
  active: string;
  onSelect: (id: string) => void;
  actions?: ReactNode;
  height: number;
  children: ReactNode;
}) {
  return (
    <Part
      as="section"
      label={tabs.find((x) => x.id === active)?.label}
      className="shrink-0 bg-vsc-panel"
      style={{ height }}
    >
      <div className="flex h-8 shrink-0 items-center pr-1 pl-0.5">
        <div role="tablist" className="flex min-w-0 flex-1 items-center gap-1">
          {tabs.map((x) => {
            const on = x.id === active;
            return (
              <button
                key={x.id}
                type="button"
                role="tab"
                aria-selected={on}
                onClick={() => {
                  onSelect(x.id);
                }}
                className={`relative flex h-8 items-center gap-1 px-2 text-[12px] capitalize ${
                  on
                    ? "text-vsc-panel-title after:absolute after:inset-x-2 after:bottom-1.5 after:h-px after:bg-vsc-panel-title-border"
                    : "text-vsc-panel-title-inactive hover:text-vsc-panel-title"
                }`}
              >
                {x.label}
                {x.badge !== undefined && x.badge !== null && <CountBadge>{x.badge}</CountBadge>}
              </button>
            );
          })}
        </div>
        {actions && <ActionBar>{actions}</ActionBar>}
      </div>
      <div className="flex min-h-0 flex-1">{children}</div>
    </Part>
  );
}

/** Scrolls the content of an editor; `narrow` keeps text readable. `fill`
 * fits it to the editor's height, so a `fill` EditorSection scrolls inside
 * and the rest stays in view. */
export function EditorBody({
  children,
  narrow = false,
  fill = false,
}: {
  children: ReactNode;
  narrow?: boolean;
  fill?: boolean;
}) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div
        className={`px-5 py-4 ${fill ? "flex h-full flex-col gap-6" : "space-y-6"} ${narrow ? "max-w-4xl" : ""}`}
      >
        {children}
      </div>
    </div>
  );
}

/** Keeps `id` among the open tabs; the active one when `open` changes. */
export function useTabs(initial: string[] = []) {
  const [open, setOpen] = useState<string[]>(initial);
  const [active, setActive] = useState<string | null>(initial[0] ?? null);
  const show = useCallback((id: string) => {
    setOpen((o) => (o.includes(id) ? o : [...o, id]));
    setActive(id);
  }, []);
  const close = useCallback(
    (id: string) => {
      setOpen((o) => {
        const i = o.indexOf(id);
        const next = o.filter((x) => x !== id);
        if (active === id) setActive(next[Math.min(i, next.length - 1)] ?? null);
        return next;
      });
    },
    [active],
  );
  /** Drops the tabs whose item is gone. */
  const keep = useCallback((ids: string[]) => {
    setOpen((o) => o.filter((x) => ids.includes(x)));
    setActive((a) => (a !== null && ids.includes(a) ? a : null));
  }, []);
  return { open, active, show, close, keep, setActive };
}
