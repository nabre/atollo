// A TextField with suggestions, what a <datalist> gives a native input: a
// <datalist> must be in the input's own tree, and vscode-textfield's input
// is in its shadow root. The list is drawn here, under the field, as the
// list of a vscode-single-select (the dropdown's colors, 22px rows, the
// list's selection); the input inside the component is the combobox,
// and points at the list and its active option through ARIA's element
// references, which reach the page from a shadow root (ids do not).
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { TextField } from "./controls";

export interface Suggestion {
  value: string;
  /** Shown dimmed after the value (a name for an id). */
  label?: string | undefined;
}

/** The ARIA element references of an input, where the browser has them. */
type Referencing = HTMLInputElement & {
  ariaControlsElements?: Element[] | null;
  ariaActiveDescendantElement?: Element | null;
};

/** The suggestions that hold the typed text, in the value or the label;
 * none that is already the value. */
function matching(suggestions: Suggestion[], value: string): Suggestion[] {
  const needle = value.trim().toLowerCase();
  return suggestions.filter(
    (s) =>
      s.value !== value && (s.value.toLowerCase().includes(needle) || (s.label ?? "").toLowerCase().includes(needle)),
  );
}

/** A text field (TextField) that suggests values as one types: the arrows
 * move through them, Enter or a click takes one, Esc closes them. The
 * field keeps any text, as with a <datalist>. */
export function SuggestField({
  value,
  onChange,
  suggestions,
  label,
  placeholder,
  disabled = false,
  invalid = false,
  mono = false,
  className = "",
}: {
  value: string;
  onChange: (v: string) => void;
  suggestions: Suggestion[];
  /** Its accessible name, and the list's. */
  label: string;
  placeholder?: string | undefined;
  disabled?: boolean;
  invalid?: boolean;
  /** The editor's font, as TextField. */
  mono?: boolean;
  className?: string;
}) {
  const box = useRef<HTMLSpanElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const shown = matching(suggestions, value);
  const expanded = open && !disabled && shown.length > 0;
  const current = Math.min(active, shown.length - 1);

  const close = () => {
    setOpen(false);
    setActive(-1);
  };
  const take = (s: Suggestion) => {
    onChange(s.value);
    close();
  };

  // The combobox is the input inside the component, drawn after it.
  useEffect(() => {
    const field = box.current?.querySelector("vscode-textfield");
    if (!field) return;
    let live = true;
    void field.updateComplete.then(() => {
      const input = field.wrappedElement as Referencing | undefined;
      if (!live || !input) return;
      input.setAttribute("role", "combobox");
      input.setAttribute("aria-autocomplete", "list");
      input.setAttribute("aria-expanded", String(expanded));
      const option = current >= 0 ? (list.current?.children[current] ?? null) : null;
      if ("ariaControlsElements" in input)
        input.ariaControlsElements = expanded && list.current ? [list.current] : null;
      if ("ariaActiveDescendantElement" in input) input.ariaActiveDescendantElement = option;
    });
    return () => {
      live = false;
    };
  });

  // Before the component: Enter that takes a suggestion submits nothing,
  // Esc that closes the list closes nothing else (a dialog).
  const keys = (e: KeyboardEvent<HTMLElement>) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      if (shown.length === 0) return;
      e.preventDefault();
      if (!expanded) {
        setOpen(true);
        setActive(e.key === "ArrowDown" ? 0 : shown.length - 1);
        return;
      }
      const step = e.key === "ArrowDown" ? 1 : -1;
      setActive((current + step + shown.length) % shown.length);
    } else if (e.key === "Enter" && expanded && current >= 0) {
      const s = shown[current];
      if (!s) return;
      e.preventDefault();
      e.stopPropagation();
      take(s);
    } else if (e.key === "Escape" && expanded) {
      e.preventDefault();
      e.stopPropagation();
      close();
    }
  };

  return (
    <span ref={box} onKeyDownCapture={keys} className={`relative inline-block ${className}`}>
      <TextField
        value={value}
        onChange={(v) => {
          onChange(v);
          setOpen(true);
          setActive(-1);
        }}
        label={label}
        placeholder={placeholder}
        disabled={disabled}
        invalid={invalid}
        mono={mono}
        autoComplete="off"
        onBlur={close}
        className="w-full"
      />
      {expanded && (
        <ul
          ref={list}
          role="listbox"
          aria-label={label}
          className="absolute top-full left-0 z-20 m-0 mt-px max-h-[calc(var(--vsc-size-row)*12)] min-w-full list-none overflow-y-auto rounded-vsc-small border border-vsc-dropdown-border bg-vsc-dropdown-list p-0 shadow-vsc-lg"
        >
          {shown.map((s, i) => (
            <li
              key={s.value}
              role="option"
              aria-selected={i === current}
              // The focus stays in the field.
              onMouseDown={(e) => {
                e.preventDefault();
              }}
              onMouseMove={() => {
                setActive(i);
              }}
              onClick={() => {
                take(s);
              }}
              className={`flex h-vsc-row cursor-pointer items-center gap-vsc-size80 px-vsc-size60 whitespace-nowrap ${
                i === current ? "bg-vsc-list-active text-vsc-list-active-fg" : "hover:bg-vsc-list-hover"
              }`}
            >
              <span className={mono ? "font-mono" : undefined}>{s.value}</span>
              {s.label && <span className="text-vsc-body-small opacity-70">{s.label}</span>}
            </li>
          ))}
        </ul>
      )}
    </span>
  );
}
