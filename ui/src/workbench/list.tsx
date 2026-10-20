// VSCodium's lists and trees (monaco-list): 22px rows with the modern UI's
// small radius, a twistie, an icon, the label and its description,
// decorations on the right and inline actions that show on hover, on focus
// and on the selected row. A row fills its list: in a pane, the pane's body
// sets it 4px in from the sides (ViewPane). The arrows move between rows,
// as in the editor. ListItem is the taller row of the Extensions view: a
// big icon, name, description and a footer.
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

/** The left padding of a row: 8px a level (workbench.tree.indent) and, on
 * a row with no twistie, the twistie's 16px. */
function indent(depth: number, twistie: boolean) {
  const level = `${String(depth + 1)} * var(--vscode-spacing-size80)`;
  return `calc(${twistie ? level : `${level} + var(--vscode-spacing-size160)`})`;
}

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
  return (
    <li>
      <div
        className={`group/row flex h-vsc-row items-center rounded-vsc-small pr-vsc-size40 ${selected ? SELECTED : "hover:bg-vsc-list-hover"} ${className}`}
      >
        <button
          type="button"
          data-row-main
          aria-current={selected ? current : undefined}
          aria-expanded={twistie ? twistie === "open" : undefined}
          title={title}
          onClick={onSelect}
          style={{ paddingLeft: indent(depth, twistie !== undefined) }}
          className="flex h-full min-w-0 flex-1 items-center gap-vsc-size60 rounded-vsc-small text-left outline-none focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-vsc-list-focus"
        >
          {twistie && (
            <VscChevronRight
              size={16}
              aria-hidden="true"
              className={`-mr-vsc-size40 shrink-0 transition-transform duration-100 ${twistie === "open" ? "rotate-90" : ""}`}
            />
          )}
          {icon && <span className="flex w-vsc-size160 shrink-0 items-center justify-center">{icon}</span>}
          <span className="min-w-0 truncate">{label}</span>
          {description && <span className="min-w-0 shrink truncate text-vsc-body-small opacity-70">{description}</span>}
        </button>
        {actions && (
          <span
            className={`flex shrink-0 items-center gap-vsc-size20 pl-vsc-size40 ${
              selected ? "" : "invisible group-focus-within/row:visible group-hover/row:visible"
            }`}
          >
            {actions}
          </span>
        )}
        {decorations && (
          <span className="flex shrink-0 items-center gap-vsc-size60 pr-vsc-size40 pl-vsc-size60 text-vsc-body-small">
            {decorations}
          </span>
        )}
      </div>
      {twistie === "open" && children}
    </li>
  );
}

/** A taller row, as the Extensions view's: 62px, 16px in, a 36px icon and
 * 16px after it, the name with its version on a 20px line, a description
 * line and a 24px footer with small buttons. */
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
        className={`group/row relative flex h-vsc-row-extension items-center gap-vsc-size160 rounded-vsc-small pr-vsc-size100 pl-vsc-size160 ${selected ? SELECTED : "hover:bg-vsc-list-hover"}`}
      >
        <span className="flex size-vsc-size360 shrink-0 items-center justify-center">{icon}</span>
        <div className="flex min-w-0 flex-1 flex-col justify-center">
          <button
            type="button"
            data-row-main
            aria-current={selected ? "true" : undefined}
            title={title}
            onClick={onSelect}
            className="flex h-vsc-size200 min-w-0 items-center gap-vsc-size60 text-left outline-none after:absolute after:inset-0 focus-visible:after:outline-1 focus-visible:after:-outline-offset-1 focus-visible:after:rounded-vsc-small focus-visible:after:outline-vsc-list-focus"
          >
            <span className="truncate font-vsc-semibold">{name}</span>
            {meta && <span className="shrink-0 text-vsc-body-small opacity-70">{meta}</span>}
          </button>
          {description && <span className="truncate text-vsc-body-small opacity-90">{description}</span>}
          {(footer ?? actions) && (
            <span className="flex h-vsc-size240 min-w-0 items-center gap-vsc-size60 pt-vsc-size20">
              <span className="min-w-0 flex-1 truncate text-vsc-body-small opacity-70">{footer}</span>
              {/* Above the row's button, so they keep their own clicks. */}
              {actions && <span className="relative z-10 flex shrink-0 items-center gap-vsc-size40">{actions}</span>}
            </span>
          )}
        </div>
      </div>
    </li>
  );
}
