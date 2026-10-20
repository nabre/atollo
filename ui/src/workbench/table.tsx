// A table of data at 12px, as the matrices and the grids of the pages: an
// optional caption, a header row, the body rows from the caller. Drawn as
// vscode-table (the Keyboard Shortcuts editor's table): the header on
// keybindingTable.headerBackground in bold, a line under it. A native
// <table>: vscode-table has no caption nor row headers, and gives every
// column the same share instead of fitting its content.
import type { CSSProperties, ReactNode } from "react";

const HEAD = "var(--vscode-keybindingTable-headerBackground)";
/** Translucent: on the editor's color when it stays over the rows. */
const STICKY: CSSProperties = { background: `linear-gradient(${HEAD}, ${HEAD}), var(--vsc-bg)` };
const PLAIN: CSSProperties = { background: HEAD };

export function DataTable({
  caption,
  captionVisible = false,
  columns,
  head,
  stickyHead = false,
  className = "",
  children,
  groups = false,
}: {
  /** Hidden (screen readers only) unless `captionVisible`. */
  caption?: string;
  /** The caption as a small bold title above the table. */
  captionVisible?: boolean;
  /** Plain header cells, left-aligned and bold; `null` is an unlabelled
   * column (actions). */
  columns?: (string | null)[];
  /** A header row of the caller's own cells, in place of `columns`. */
  head?: ReactNode;
  /** The header stays on top while the rows scroll. */
  stickyHead?: boolean;
  className?: string;
  /** The body rows; with `groups`, the caller's own `<tbody>` elements
   * (one per group, its title row first). */
  children: ReactNode;
  groups?: boolean;
}) {
  const headClass = `border-b border-(--vscode-editorGroup-border) ${stickyHead ? "sticky top-0 z-10" : ""}`;
  const headStyle = stickyHead ? STICKY : PLAIN;
  return (
    <table className={className}>
      {caption !== undefined && (
        <caption className={captionVisible ? "pb-1 text-left text-[12px] font-semibold" : "sr-only"}>{caption}</caption>
      )}
      {columns ? (
        <thead className={headClass} style={headStyle}>
          <tr>
            {columns.map((c, i) =>
              c === null ? (
                <th key={i} scope="col" className="px-2 py-[5px]" />
              ) : (
                <th key={i} scope="col" className="px-2 py-[5px] text-left leading-5 font-bold whitespace-nowrap">
                  {c}
                </th>
              ),
            )}
          </tr>
        </thead>
      ) : (
        head !== undefined && (
          <thead className={headClass} style={headStyle}>
            <tr>{head}</tr>
          </thead>
        )
      )}
      {groups ? children : <tbody>{children}</tbody>}
    </table>
  );
}
