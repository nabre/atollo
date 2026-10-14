// VSCodium's lists and trees (monaco-list): 22px rows, rounded 4px, with a twistie, an
// icon, the label and its description, decorations on the right and inline
// actions that show on hover, on focus and on the selected row. The arrows
// move between rows, as in the editor. ListItem is the taller row of the
// Extensions view: a big icon, name, description and a footer.
import { useRef, type KeyboardEvent, type ReactNode } from "react";
import { VscChevronRight } from "react-icons/vsc";

/** Moves the focus between the rows' main buttons with the arrows. */
function onListKeys(e: KeyboardEvent<HTMLUListElement>) {
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) return;
  const rows = [...e.currentTarget.querySelectorAll<HTMLElement>("[data-row-main]")];
  if (rows.length === 0) return;
  const at = rows.findIndex((r) => r.contains(document.activeElement));
  const next =
    e.key === "Home"
      ? 0
      : e.key === "End"
        ? rows.length - 1
        : e.key === "ArrowDown"
          ? Math.min(rows.length - 1, at + 1)
          : Math.max(0, at - 1);
  e.preventDefault();
  rows[next]?.focus();
}

export function List({
  label,
  children,
  className = "",
}: {
  /** Accessible name of the list. */
  label: string;
  children: ReactNode;
  className?: string;
}) {
  const ref = useRef<HTMLUListElement>(null);
  return (
    <ul ref={ref} aria-label={label} onKeyDown={onListKeys} className={`group/list m-0 list-none p-0 ${className}`}>
      {children}
    </ul>
  );
}

/** Selection colors: active while the list has the focus, inactive after. */
const SELECTED =
  "bg-vsc-list-inactive group-focus-within/list:bg-vsc-list-active group-focus-within/list:text-vsc-list-active-fg";

export function ListRow({
  icon,
  label,
  description,
  decorations,
  actions,
  selected = false,
  onSelect,
  twistie,
  depth = 0,
  title,
  current = "true",
  className = "",
  children,
}: {
  icon?: ReactNode;
  label: ReactNode;
  /** After the label, smaller and dimmed (the folder of a file). */
  description?: ReactNode;
  /** On the right, always shown (git state, problems). */
  decorations?: ReactNode;
  /** On the right, on hover, focus and selection. */
  actions?: ReactNode;
  selected?: boolean;
  onSelect?: () => void;
  /** A tree row: open or closed. */
  twistie?: "open" | "closed" | undefined;
  depth?: number;
  title?: string | undefined;
  /** aria-current of the selected row: "page" for navigation. */
  current?: "true" | "page";
  className?: string;
  /** Shown under the row when the twistie is open. */
  children?: ReactNode;
}) {
  const indent = 8 + depth * 8 + (twistie ? 0 : 16);
  return (
    <li>
      <div
        className={`group/row flex h-[22px] items-center rounded pr-1 ${selected ? SELECTED : "hover:bg-vsc-list-hover"} ${className}`}
      >
        <button
          type="button"
          data-row-main
          aria-current={selected ? current : undefined}
          aria-expanded={twistie ? twistie === "open" : undefined}
          title={title}
          onClick={onSelect}
          style={{ paddingLeft: indent }}
          className="flex h-full min-w-0 flex-1 items-center gap-1.5 text-left outline-none focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-list-focus"
        >
          {twistie && (
            <VscChevronRight
              size={16}
              aria-hidden="true"
              className={`-mr-1 shrink-0 transition-transform duration-100 ${twistie === "open" ? "rotate-90" : ""}`}
            />
          )}
          {icon && <span className="flex w-4 shrink-0 items-center justify-center">{icon}</span>}
          <span className="min-w-0 truncate">{label}</span>
          {description && <span className="min-w-0 shrink truncate text-[12px] opacity-70">{description}</span>}
        </button>
        {actions && (
          <span
            className={`flex shrink-0 items-center gap-0.5 pl-1 ${
              selected ? "" : "invisible group-focus-within/row:visible group-hover/row:visible"
            }`}
          >
            {actions}
          </span>
        )}
        {decorations && (
          <span className="flex shrink-0 items-center gap-1.5 pr-1 pl-1.5 text-[12px]">{decorations}</span>
        )}
      </div>
      {twistie === "open" && children}
    </li>
  );
}

/** A taller row, as the Extensions view's: a 42px icon, the name with its
 * version, a description line and a footer with small buttons. */
export function ListItem({
  icon,
  name,
  meta,
  description,
  footer,
  actions,
  selected = false,
  onSelect,
  title,
}: {
  icon: ReactNode;
  name: ReactNode;
  /** After the name: version, state. */
  meta?: ReactNode;
  description?: ReactNode;
  /** The publisher line on the left of the footer. */
  footer?: ReactNode;
  /** The small buttons on the right of the footer. */
  actions?: ReactNode;
  selected?: boolean;
  onSelect?: () => void;
  title?: string | undefined;
}) {
  return (
    <li>
      <div
        className={`group/row relative flex h-[62px] items-center gap-2.5 rounded pr-3 pl-4 ${selected ? SELECTED : "hover:bg-vsc-list-hover"}`}
      >
        <span className="flex size-[42px] shrink-0 items-center justify-center">{icon}</span>
        <div className="flex min-w-0 flex-1 flex-col justify-center gap-0.5 leading-[17px]">
          <button
            type="button"
            data-row-main
            aria-current={selected ? "true" : undefined}
            title={title}
            onClick={onSelect}
            className="flex min-w-0 items-baseline gap-1.5 text-left outline-none after:absolute after:inset-0 focus-visible:after:outline-1 focus-visible:after:-outline-offset-1 focus-visible:after:outline-vsc-list-focus"
          >
            <span className="truncate font-semibold">{name}</span>
            {meta && <span className="shrink-0 text-[12px] opacity-70">{meta}</span>}
          </button>
          {description && <span className="truncate text-[12px] opacity-90">{description}</span>}
          {(footer ?? actions) && (
            <span className="flex min-w-0 items-center gap-1.5">
              <span className="min-w-0 flex-1 truncate text-[12px] opacity-70">{footer}</span>
              {/* Above the row's button, so they keep their own clicks. */}
              {actions && <span className="relative z-10 flex shrink-0 items-center gap-1">{actions}</span>}
            </span>
          )}
        </div>
      </div>
    </li>
  );
}
