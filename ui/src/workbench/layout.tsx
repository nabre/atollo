// VSCodium's parts and views (workbench/browser/parts): the side bar with
// its title and collapsible panes, the sash between parts, the editor with
// its tabs, the header and navigation bar of a details editor (as an
// extension's page), the panel, and the welcome view of an empty view.
// Parts float 4px apart on the workbench color, as VSCodium 1.135's modern
// layout; the app composes them per page (ViewContainer).
// Not vscode-elements here: vscode-collapsible and vscode-tabs draw the
// classic 22px uppercase pane and panel headers, not the modern ones; the
// collapsible's header has no button role nor aria-expanded, the tabs keep
// their tablist inside the shadow root and have no close button for the
// editor's tabs, and the details editor's bar is navigation, not tabs.
import { useCallback, useEffect, useId, useMemo, useRef, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { t } from "../i18n";
import { ActionBar } from "./buttons";
import { CountBadge } from "./indicators";
import { Codicon } from "./codicon";

/** A floating part: rounded, its own border, its own background. */
export function Part({
  as: Tag = "div",
  ref,
  id,
  label,
  className = "",
  style,
  children,
}: {
  as?: "div" | "aside" | "main" | "section";
  /** The element, for a size dragged with a sash (SizeDrag). */
  ref?: ((el: HTMLElement | null) => void) | undefined;
  id?: string;
  label?: string | undefined;
  className?: string;
  style?: React.CSSProperties;
  children: ReactNode;
}) {
  return (
    <Tag
      ref={ref}
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

function writeSize(key: string, size: number) {
  try {
    localStorage.setItem(`size:${key}`, String(size));
  } catch {
    // Not remembered: the size still changes.
  }
}

/** How long a size stays still before it is written. */
const STORE_DELAY_MS = 150;

/** The live drag of a size (`useStoredSize`): `attach` as the ref of the
 * element that has the size, `move` at each pointer move, `end` when the
 * pointer lets go. */
export interface SizeDrag {
  attach: (el: HTMLElement | null) => void;
  move: (v: (prev: number) => number) => void;
  end: () => void;
}

/** A size the user drags with a sash, remembered in this browser. The
 * updater only clamps (it can run twice); the size is written once the
 * drag pauses, not at every pixel, and when the part goes away before
 * that. During a drag (`drag`, with a Sash's onDragEnd) the size goes to
 * the element's style at each pointer move, `dimension` its width or its
 * height, and to the state once, when the pointer lets go: the page under
 * it renders once, not at every pixel. */
export function useStoredSize(
  key: string,
  fallback: number,
  min: number,
  max: number,
  dimension: "width" | "height" = "width",
) {
  const [size, setSize] = useState(() => Math.min(max, Math.max(min, readSize(key, fallback))));
  const clamp = useCallback((v: number) => Math.round(Math.min(max, Math.max(min, v))), [min, max]);
  const set = useCallback(
    (v: number | ((prev: number) => number)) => {
      setSize((prev) => clamp(typeof v === "function" ? v(prev) : v));
    },
    [clamp],
  );
  // The element that has the size; without one, a drag sets the state.
  const node = useRef<HTMLElement | null>(null);
  const attach = useCallback((el: HTMLElement | null) => {
    node.current = el;
  }, []);
  // The size being dragged, not in the state yet; null between drags.
  const draft = useRef<number | null>(null);
  const committed = useRef(size);
  useEffect(() => {
    committed.current = size;
  }, [size]);
  const move = useCallback(
    (v: (prev: number) => number) => {
      const el = node.current;
      if (!el) {
        set(v);
        return;
      }
      const next = clamp(v(draft.current ?? committed.current));
      draft.current = next;
      el.style[dimension] = `${String(next)}px`;
    },
    [clamp, set, dimension],
  );
  const end = useCallback(() => {
    const last = draft.current;
    draft.current = null;
    if (last !== null) set(last);
  }, [set]);
  const drag = useMemo<SizeDrag>(() => ({ attach, move, end }), [attach, move, end]);
  // The size not written yet; null when there is none (as at the start).
  const pending = useRef<{ key: string; size: number } | null>(null);
  const first = useRef(true);
  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    pending.current = { key, size };
    const timer = setTimeout(() => {
      pending.current = null;
      writeSize(key, size);
    }, STORE_DELAY_MS);
    return () => {
      clearTimeout(timer);
    };
  }, [key, size]);
  useEffect(
    () => () => {
      const last = pending.current;
      if (last) writeSize(last.key, last.size);
    },
    [],
  );
  return [size, set, drag] as const;
}

/** A box `height` pixels high, its content filling it, that the drag of a
 * sash resizes live (`useStoredSize` with "height"): the content renders
 * once, when the drag ends. */
export function SizedBox({
  drag: { attach },
  height,
  children,
}: {
  drag: SizeDrag;
  height: number;
  children: ReactNode;
}) {
  return (
    <div ref={attach} style={{ height }} className="flex shrink-0 flex-col">
      {children}
    </div>
  );
}

/** The sash between two parts: drag to resize, double click to reset;
 * arrows when focused. `delta` is the movement in pixels; `size`, `min`
 * and `max` are the part's, in pixels, read out by screen readers. */
export function Sash({
  vertical,
  label,
  size,
  min,
  max,
  onDrag,
  onDragEnd,
  onReset,
}: {
  /** Between parts side by side (drags left-right). */
  vertical: boolean;
  label: string;
  size: number;
  min: number;
  max: number;
  onDrag: (delta: number) => void;
  /** The drag is over (the pointer let go, or a key moved it): what
   * `onDrag` only showed is kept (SizeDrag's `end`). */
  onDragEnd?: () => void;
  onReset: () => void;
}) {
  const start = useRef<number | null>(null);
  // A drag that ends without a pointerup (the window loses the focus, a
  // touch cancelled) ends here too: else the next hover would resize.
  const end = () => {
    if (start.current === null) return;
    start.current = null;
    onDragEnd?.();
  };
  return (
    <div
      role="separator"
      aria-label={label}
      aria-orientation={vertical ? "vertical" : "horizontal"}
      aria-valuenow={size}
      aria-valuemin={min}
      aria-valuemax={max}
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
      onPointerUp={end}
      onPointerCancel={end}
      onLostPointerCapture={end}
      onDoubleClick={onReset}
      onKeyDown={(e) => {
        const step = e.shiftKey ? 40 : 10;
        const keys = vertical ? { ArrowLeft: -step, ArrowRight: step } : { ArrowUp: -step, ArrowDown: step };
        const d = (keys as Record<string, number>)[e.key];
        if (d !== undefined) {
          e.preventDefault();
          onDrag(d);
          onDragEnd?.();
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
  const [width, setWidth, { attach, move, end }] = useStoredSize(`side:${id}`, 300, 170, 640);
  return (
    <div className="flex min-h-0 min-w-0 flex-1 gap-1">
      {side && (
        <>
          <Part ref={attach} as="aside" className="shrink-0 bg-vsc-sidebar text-vsc-sidebar-fg" style={{ width }}>
            {side}
          </Part>
          <Sash
            vertical
            label={t("Ridimensiona la barra laterale")}
            size={width}
            min={170}
            max={640}
            onDrag={(d) => {
              move((w) => w + d);
            }}
            onDragEnd={end}
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
      <h1 className="min-w-0 flex-1 truncate pl-2 text-[12px] leading-8 font-semibold text-vsc-sidebar-title">
        {title}
      </h1>
      {actions && <ActionBar className="pl-1">{actions}</ActionBar>}
    </div>
  );
}

/** The panes of a side bar, one over the other; `fill` panes share the
 * rest. Each pane sets its header and rows 4px in from the sides, so the
 * scroll bars stay on the edge. */
export function PaneStack({ children }: { children: ReactNode }) {
  return <div className="group/panes flex min-h-0 flex-1 flex-col pb-vsc-size40">{children}</div>;
}

/** The least body of a `fill` pane beside a `fit` one, as VSCodium's
 * panes' minimumBodySize: the `fit` pane never takes all the height. */
const FILL_BESIDE_FIT = "group-has-[[data-fit]]/panes:min-h-[calc(var(--vsc-size-pane-header)+120px)]";

/** A collapsible pane (Outline, Timeline), as the modern UI's: a 28px
 * header with the small radius, 4px in from the sides, its title 12px
 * semibold, list-hoverBackground on hover, a line 4px further in above all
 * but the first, its actions on hover, a badge; the body scrolls inside,
 * its rows 4px in from the sides. */
export function ViewPane({
  title,
  expanded,
  onToggle,
  fill = false,
  fit = false,
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
  /** As tall as its content, up to the height there is, scrolling only
   * beyond it; the rest goes to the other `fill` panes (as VSCodium's Open
   * Editors). Set by the view while other panes are open: alone, a `fill`
   * pane fills. */
  fit?: boolean;
  /** No border on top: the first pane of the side bar. */
  first?: boolean;
  actions?: ReactNode;
  badge?: ReactNode;
  children: ReactNode;
}) {
  const fits = expanded && fit;
  return (
    <section
      data-fit={fits ? "" : undefined}
      className={`flex min-h-vsc-pane-header flex-col ${
        fits ? "flex-[0_1_auto]" : expanded && fill ? `flex-1 basis-0 ${FILL_BESIDE_FIT}` : "shrink-0"
      } ${expanded && !fill && !fit ? "max-h-[60%]" : ""}`}
    >
      <h3
        className={`group/pane relative mx-vsc-size40 flex h-vsc-pane-header shrink-0 items-center rounded-vsc-small text-vsc-section-fg hover:bg-vsc-list-hover ${
          first
            ? ""
            : "before:absolute before:inset-x-vsc-size40 before:top-0 before:h-(--vscode-strokeThickness) before:bg-vsc-section-border"
        }`}
      >
        <button
          type="button"
          aria-expanded={expanded}
          onClick={onToggle}
          className="flex h-full min-w-0 flex-1 items-center rounded-vsc-small pl-vsc-size40 text-left text-vsc-label1 font-vsc-semibold outline-none focus-visible:outline-1 focus-visible:-outline-offset-2 focus-visible:outline-vsc-focus"
        >
          <Codicon
            name="chevron-right"
            size={16}
            className={`mx-vsc-size20 shrink-0 transition-transform duration-100 ${expanded ? "rotate-90" : ""}`}
          />
          <span className="truncate">{title}</span>
        </button>
        {badge !== undefined && badge !== null && <CountBadge className="mx-vsc-size40">{badge}</CountBadge>}
        {actions && expanded && (
          <ActionBar className="invisible ml-vsc-size40 gap-vsc-size40 pr-vsc-size40 group-focus-within/pane:visible group-hover/pane:visible">
            {actions}
          </ActionBar>
        )}
      </h3>
      {expanded && (
        <div className={`min-h-0 overflow-y-auto px-vsc-size40 ${fits ? "flex-[0_1_auto]" : "flex-1"}`}>{children}</div>
      )}
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
  /** Always open: no close button (the Tools page's table). */
  pinned?: boolean;
}

/** The tab bar of the modern UI: 32px, on the editor color, no border. */
const TAB_BAR = "flex h-vsc-tabs shrink-0 pl-vsc-size20";

/** A tab: its fill is a 24px pill 2px in from the sides, rounded; the active
 * one on the list's selection color, the others transparent
 * (modernEditorTab.inactiveBackground) and on the list's hover color under
 * the pointer, the active one too. Text 13px, the inactive tabs' at half the
 * foreground. */
const TAB =
  "group/tab relative flex h-full max-w-60 min-w-0 shrink-0 items-center text-vsc-body1 before:absolute before:inset-x-vsc-size20 before:inset-y-vsc-size40 before:rounded-vsc-small";
const TAB_ACTIVE = "text-vsc-fg before:bg-vsc-list-inactive";
const TAB_INACTIVE = "text-vsc-fg/50 hover:text-vsc-fg";

/** The editor's tabs (modern UI): pills, the active one filled; the close
 * button over the tab's right end (.tab-actions, 24px), on its fill with a
 * fade before it, shown on hover or focus only, the active tab's too. */
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
      <div
        role="tablist"
        aria-label={t("Editor")}
        className="flex min-w-0 flex-1 overflow-x-auto [scrollbar-width:none]"
      >
        {tabs.map((tab) => {
          const on = tab.id === active;
          const close = tab.pinned ? undefined : onClose;
          return (
            <div key={tab.id} className={`${TAB} ${on ? TAB_ACTIVE : TAB_INACTIVE} hover:before:bg-vsc-list-hover`}>
              <button
                type="button"
                role="tab"
                aria-selected={on}
                title={tab.title ?? tab.label}
                onClick={() => {
                  onSelect(tab.id);
                }}
                onAuxClick={(e) => {
                  if (e.button === 1 && close) close(tab.id);
                }}
                className="relative flex h-full min-w-0 items-center gap-vsc-size60 pr-vsc-size80 pl-vsc-size60 outline-none focus-visible:outline-1 focus-visible:-outline-offset-4 focus-visible:outline-vsc-focus"
              >
                {tab.icon && <span className="flex w-vsc-size160 shrink-0 justify-center">{tab.icon}</span>}
                <span className="truncate">{tab.label}</span>
                {tab.description && <span className="truncate text-vsc-body-small opacity-70">{tab.description}</span>}
              </button>
              {close && (
                <span
                  className={`pointer-events-none absolute inset-y-vsc-size40 right-vsc-size20 flex w-vsc-size240 items-center justify-center rounded-r-vsc-small opacity-0 group-hover/tab:pointer-events-auto group-hover/tab:opacity-100 before:pointer-events-none before:absolute before:inset-y-0 before:right-full before:w-vsc-size160 before:bg-linear-to-r before:from-transparent focus-within:pointer-events-auto focus-within:opacity-100 ${
                    on
                      ? "bg-vsc-list-inactive group-hover/tab:bg-vsc-list-hover before:to-vsc-list-inactive group-hover/tab:before:to-vsc-list-hover"
                      : "bg-vsc-list-hover before:to-vsc-list-hover"
                  }`}
                >
                  <button
                    type="button"
                    aria-label={t("Chiudi {0}", tab.label)}
                    title={t("Chiudi")}
                    onClick={() => {
                      close(tab.id);
                    }}
                    className="flex size-vsc-size200 items-center justify-center rounded-vsc-small text-vsc-fg outline-none hover:bg-vsc-toolbar-hover focus-visible:outline-1 focus-visible:outline-vsc-focus"
                  >
                    <Codicon name="close" size={16} />
                  </button>
                </span>
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
        <div className={`${TAB} ${TAB_ACTIVE} gap-vsc-size60 pr-vsc-size80 pl-vsc-size60`}>
          <span className="relative flex w-vsc-size160 justify-center">{icon}</span>
          <h1 className="relative text-vsc-body1 font-normal">{title}</h1>
        </div>
      </div>
      {children}
    </ViewContainer>
  );
}

/** The header of a details editor, as an extension's page
 * (.extension-editor>.header): a 128px icon, 20px from the top and the left,
 * 12px above the bar, its text 14px; the name on a 30px line, a line of
 * facts 6px under it with a bar 14px from each, a description and the
 * buttons 10px apart. The icon given (a logo, a codicon) is drawn at the
 * full 128px. */
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
    <header className="flex shrink-0 gap-vsc-size120 px-vsc-size200 pt-vsc-size200 pb-vsc-size120 text-(length:--vsc-size-extension-header)">
      <span className="flex size-vsc-extension-icon shrink-0 items-center justify-center [&>.codicon]:text-(length:--vsc-size-extension-icon)! [&>svg]:size-full">
        {icon}
      </span>
      <div className="min-w-0 flex-1">
        <h2
          id={titleId}
          className="truncate text-vsc-heading1 leading-(--vsc-size-extension-name) font-semibold text-vsc-settings-header select-text"
        >
          {title}
        </h2>
        {facts.length > 0 && (
          <div className="flex flex-wrap items-center pt-vsc-size60 leading-5">
            {facts.map((f, i) => (
              <span
                key={i}
                // The bar's color is fixed in VSCodium's CSS (.subtitle-entry),
                // not a theme color.
                className={`flex items-center gap-vsc-size60 ${i > 0 ? "ml-vsc-extension-fact border-l border-[rgba(128,128,128,0.7)] pl-vsc-extension-fact" : ""}`}
              >
                {f}
              </span>
            ))}
          </div>
        )}
        {description && <div className="mt-vsc-size100 text-vsc-desc">{description}</div>}
        {actions && <div className="mt-vsc-size100 flex flex-wrap items-center gap-vsc-size60">{actions}</div>}
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

/** The navigation bar of a details editor (Details, Features, Changelog),
 * as .extension-editor>.body>.navbar: 36px, 20px in, a panelSection.border
 * line under it, 11px uppercase items 10px wide on each side, the active one
 * underlined. */
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
    <nav aria-label={label} className="h-vsc-size360 shrink-0 border-b border-vsc-panel-section-border pl-vsc-size200">
      <ul className="m-0 flex h-full list-none overflow-x-auto p-0 [scrollbar-width:none]">
        {items.map((x) => {
          const on = x.id === active;
          const Icon = x.icon;
          return (
            <li key={x.id} className="shrink-0">
              <button
                type="button"
                aria-current={on ? "page" : undefined}
                onClick={() => {
                  onSelect(x.id);
                }}
                className={`relative flex h-full items-center gap-vsc-size60 px-vsc-size100 text-vsc-body-xsmall font-normal uppercase outline-none focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-focus ${
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

/** The editor's empty state: the next steps on the brand cover
 * (hero-{dark,light}.svg), filling the editor. */
export function Watermark({ children }: { children: ReactNode }) {
  return (
    <div className="bg-hero flex min-h-full flex-1 flex-col items-center justify-center gap-6 p-6 text-center">
      {children}
    </div>
  );
}

/** The panel at the bottom (Problems, Output, Terminal): its 32px title bar
 * with the tabs and its actions. The tabs are VSCodium's modern ones
 * (modern-ui-tabs, on with the modern UI): 10px beside the label, the
 * active one on a 24px pill 2px in from its sides, as the editor's tabs. */
export function Panel({
  tabs,
  active,
  onSelect,
  actions,
  height,
  children,
}: {
  /** `tag`: a short note after the label, as a terminal's level (D-179);
   * `hint`: the tab's tooltip. */
  tabs: { id: string; label: string; badge?: ReactNode; tag?: string; hint?: string | undefined }[];
  active: string;
  onSelect: (id: string) => void;
  actions?: ReactNode;
  /** Its height in pixels; none: it fills its window (a detached
   * terminal's). */
  height?: number | undefined;
  children: ReactNode;
}) {
  const uid = useId();
  return (
    <Part
      as="section"
      label={tabs.find((x) => x.id === active)?.label}
      className={`bg-vsc-panel ${height === undefined ? "min-h-0 flex-1" : "shrink-0"}`}
      {...(height === undefined ? {} : { style: { height } })}
    >
      <div className="flex h-vsc-title shrink-0 items-center pr-vsc-size40 pl-vsc-size20">
        <div role="tablist" className="flex min-w-0 flex-1 items-center self-stretch">
          {tabs.map((x) => {
            const on = x.id === active;
            // The tag describes the tab: its name stays the label.
            const tagId = `${uid}-tag-${x.id}`;
            return (
              <button
                key={x.id}
                type="button"
                role="tab"
                aria-selected={on}
                title={x.hint}
                aria-describedby={x.tag ? tagId : undefined}
                onClick={() => {
                  onSelect(x.id);
                }}
                className={`relative isolate flex h-full items-center gap-vsc-size40 px-vsc-size100 text-vsc-body1 outline-hidden before:absolute before:inset-x-vsc-size20 before:inset-y-vsc-size40 before:-z-10 before:rounded-vsc-small focus-visible:before:outline-1 focus-visible:before:-outline-offset-1 focus-visible:before:outline-vsc-focus ${
                  on
                    ? "text-vsc-panel-title before:bg-vsc-list-inactive"
                    : "text-vsc-panel-title-inactive hover:text-vsc-panel-title hover:before:bg-vsc-list-hover"
                }`}
              >
                {x.label}
                {x.tag && (
                  <span id={tagId} aria-hidden="true" className="text-vsc-desc">
                    {x.tag}
                  </span>
                )}
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

/** Scrolls the content of an editor across the editor's full width. `fill`
 * fits it to the editor's height, so a `fill` EditorSection scrolls inside
 * and the rest stays in view. */
export function EditorBody({ children, fill = false }: { children: ReactNode; fill?: boolean }) {
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className={`px-5 py-4 ${fill ? "flex h-full flex-col gap-6" : "space-y-6"}`}>{children}</div>
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
