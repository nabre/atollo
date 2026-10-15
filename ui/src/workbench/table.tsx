// A table of data at 12px, as the matrices and the grids of the pages: an
// optional caption, a header row, the body rows from the caller.
import type { ReactNode } from "react";

export function DataTable({
  caption,
  captionVisible = false,
  columns,
  head,
  stickyHead = false,
  className = "",
  children,
}: {
  /** Hidden (screen readers only) unless `captionVisible`. */
  caption?: string;
  /** The caption as a small bold title above the table. */
  captionVisible?: boolean;
  /** Plain header cells, left-aligned in the description color; `null` is
   * an unlabelled column (actions). */
  columns?: (string | null)[];
  /** A header row of the caller's own cells, in place of `columns`. */
  head?: ReactNode;
  /** The header stays on top while the rows scroll. */
  stickyHead?: boolean;
  className?: string;
  /** The body rows. */
  children: ReactNode;
}) {
  const headClass = stickyHead ? "sticky top-0 z-10 bg-vsc-bg" : undefined;
  return (
    <table className={className}>
      {caption !== undefined && (
        <caption className={captionVisible ? "pb-1 text-left text-[12px] font-semibold" : "sr-only"}>{caption}</caption>
      )}
      {columns ? (
        <thead className={headClass}>
          <tr className="text-vsc-desc">
            {columns.map((c, i) =>
              c === null ? (
                <th key={i} scope="col" className="px-2" />
              ) : (
                <th key={i} scope="col" className="px-2 text-left font-normal">
                  {c}
                </th>
              ),
            )}
          </tr>
        </thead>
      ) : (
        head !== undefined && (
          <thead className={headClass}>
            <tr>{head}</tr>
          </thead>
        )
      )}
      <tbody>{children}</tbody>
    </table>
  );
}
