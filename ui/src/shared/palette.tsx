// The editor's quick pick: a box at the top of the window, a list filtered
// by every word typed, ↑/↓ to move, Enter to run, Esc or a click outside to
// close. Quick Open of Projects (Ctrl+P) and the command palette
// (Ctrl+Shift+P, report v6/04 N-8) are made of it.
import { useRef, useState, type KeyboardEvent, type ReactNode } from "react";
import { TextField, useModalFocus } from "../workbench";

export interface PaletteItem {
  key: string;
  icon: ReactNode;
  label: string;
  detail?: string | undefined;
  decoration?: ReactNode;
  /** What the words are looked for in; the label and the detail without. */
  search?: string | undefined;
  run: () => void;
  /** Shift+Enter, when it has a second action (Quick Open: start). */
  alt?: (() => void) | undefined;
}

/** The items whose text has every word of `query`, in their order. */
export function filterItems(items: PaletteItem[], query: string): PaletteItem[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  return items.filter((item) => {
    const text = (item.search ?? `${item.label} ${item.detail ?? ""}`).toLowerCase();
    return words.every((w) => text.includes(w));
  });
}

export function Palette({
  label,
  placeholder,
  listLabel,
  items: all,
  empty,
  hint,
  onClose,
}: {
  /** The dialog's and the box's name. */
  label: string;
  placeholder: string;
  listLabel: string;
  items: PaletteItem[];
  empty: string;
  hint: string;
  onClose: () => void;
}) {
  const box = useRef<HTMLDivElement>(null);
  useModalFocus(box, onClose);
  const [query, setQuery] = useState("");
  const [at, setAt] = useState(0);
  const items = filterItems(all, query);
  const current = Math.min(at, items.length - 1);
  // Closed first: what runs may open a page or another dialog.
  const done = (f: () => void) => {
    onClose();
    f();
  };
  const keys = (e: KeyboardEvent<HTMLElement>) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const step = e.key === "ArrowDown" ? 1 : -1;
      setAt((current + step + items.length) % Math.max(items.length, 1));
    } else if (e.key === "Enter") {
      const item = items[current];
      if (!item) return;
      e.preventDefault();
      done(e.shiftKey && item.alt ? item.alt : item.run);
    }
  };
  return (
    <div
      className="fixed inset-0 z-50 flex justify-center pt-12"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={box}
        role="dialog"
        aria-modal="true"
        aria-label={label}
        tabIndex={-1}
        onKeyDown={keys}
        className="flex max-h-[60vh] w-[min(36rem,90vw)] flex-col gap-1 self-start rounded-md border border-vsc-widget-border bg-vsc-widget p-1.5 text-(--vscode-editorWidget-foreground) shadow-[0_0_8px_2px_var(--vsc-shadow)]"
      >
        <TextField
          label={label}
          placeholder={placeholder}
          autoFocus
          value={query}
          onChange={(v) => {
            setQuery(v);
            setAt(0);
          }}
          className="w-full"
        />
        <ul role="listbox" aria-label={listLabel} className="m-0 min-h-0 list-none overflow-y-auto p-0">
          {items.map((item, i) => (
            <li
              key={item.key}
              role="option"
              aria-selected={i === current}
              onMouseMove={() => {
                setAt(i);
              }}
              onClick={() => {
                done(item.run);
              }}
              className={`flex h-[22px] cursor-pointer items-center gap-1.5 rounded px-1.5 ${
                i === current ? "bg-vsc-list-active text-vsc-list-active-fg" : ""
              }`}
            >
              <span className="flex w-4 shrink-0 justify-center">{item.icon}</span>
              <span className="truncate">{item.label}</span>
              {item.detail && <span className="truncate text-[12px] opacity-70">{item.detail}</span>}
              <span className="flex-1" />
              {item.decoration}
            </li>
          ))}
          {items.length === 0 && <li className="px-1.5 text-vsc-desc">{empty}</li>}
        </ul>
        <p className="px-1.5 text-[11px] text-vsc-desc">{hint}</p>
      </div>
    </div>
  );
}
