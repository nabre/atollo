// Confirmations: the modal one over the whole window (VSCodium's dialog
// box) and the inline one under the button that asked it.
import { Children, useEffect, useId, useRef, useState, type ReactNode, type RefObject } from "react";
import { Button } from "./controls";

// What Tab reaches: not a hidden control nor one out of the tab order (as
// the native submit button next to a Button with `submit`).
const REACHABLE = ':not([hidden]):not([tabindex="-1"])';
const FOCUSABLE = [
  "button:not([disabled])",
  "[href]",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]",
]
  .map((s) => s + REACHABLE)
  .join(", ");

/** The controls Tab moves among in a box, in order, without those inside a
 * hidden part. */
function tabbable(box: HTMLElement): HTMLElement[] {
  return Array.from(box.querySelectorAll<HTMLElement>(FOCUSABLE)).filter((el) => !el.closest("[hidden]"));
}

/**
 * The focus of a modal box over the window (`box`, mounted with the
 * component): when it opens the focus goes inside, to the control that
 * took it (autoFocus) or else the first one, or the box itself (give it
 * tabIndex -1); Tab and Shift+Tab stay inside; Esc calls `onClose`; once
 * closed, the focus goes back where it was. `onClose` can be a new
 * function at every render.
 */
export function useModalFocus(box: RefObject<HTMLElement | null>, onClose: () => void): void {
  // Where the focus was before the box took it (read while rendering,
  // before the children's autoFocus).
  const [opener] = useState(() => (document.activeElement instanceof HTMLElement ? document.activeElement : null));
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  });
  useEffect(() => {
    const node = box.current;
    if (node && !node.contains(document.activeElement)) (tabbable(node)[0] ?? node).focus();
    return () => {
      // Still in the page: a remount (StrictMode), not a close. Otherwise
      // the focus fell to the body with the box; a choice that moved it
      // elsewhere keeps it there.
      if (node?.isConnected) return;
      const at = document.activeElement;
      if (opener?.isConnected && (at === null || at === document.body)) opener.focus();
    };
  }, [box, opener]);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        close.current();
        return;
      }
      const node = box.current;
      if (e.key !== "Tab" || !node) return;
      const items = tabbable(node);
      if (items.length === 0) {
        // Nothing to move to: the focus stays on the box.
        e.preventDefault();
        return;
      }
      const first = items[0];
      const last = items[items.length - 1];
      const outside = !node.contains(document.activeElement);
      if (e.shiftKey) {
        if (document.activeElement === first || outside) {
          e.preventDefault();
          last?.focus();
        }
      } else if (document.activeElement === last || outside) {
        e.preventDefault();
        first?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [box]);
}

/** A modal question: the cancel button (a vscode-button) has the focus,
 * Tab stays inside, Esc cancels; once closed, the focus goes back where it
 * was (useModalFocus). The confirming buttons and an error line come from
 * the caller (`actions`, `footer`). */
export function ConfirmDialog({
  title,
  children,
  cancelLabel,
  onCancel,
  actions,
  footer,
}: {
  title: string;
  /** The description under the title. */
  children: ReactNode;
  cancelLabel: string;
  onCancel: () => void;
  actions: ReactNode;
  footer?: ReactNode;
}) {
  const id = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  useModalFocus(dialogRef, onCancel);
  return (
    // The dimmed window: no theme color for it in VSCodium either.
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div
        ref={dialogRef}
        role="alertdialog"
        aria-modal="true"
        tabIndex={-1}
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-desc`}
        className="w-[min(32rem,90vw)] space-y-2 rounded border border-vsc-widget-border bg-vsc-widget p-3 text-(--vscode-editorWidget-foreground) shadow-[0_0_8px_2px_var(--vsc-shadow)]"
      >
        <h2 id={`${id}-title`} className="text-[14px] font-semibold">
          {title}
        </h2>
        <div id={`${id}-desc`} className="space-y-2">
          {children}
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button autoFocus onClick={onCancel}>
            {cancelLabel}
          </Button>
          {actions}
        </div>
        {footer}
      </div>
    </div>
  );
}

/** Where the focus goes once an InlineConfirm closes: the control that
 * opened it when still in the page; otherwise the one rendered again in its
 * place (callers show the question instead of their button), found in the
 * part of the page around the question by its kind and its name. */
function focusBack(opener: HTMLElement | null, around: HTMLElement | null): HTMLElement | null {
  if (!opener) return null;
  if (opener.isConnected) return opener;
  if (!around?.isConnected) return null;
  const name = (el: HTMLElement) => el.getAttribute("aria-label") ?? el.textContent?.trim() ?? "";
  const wanted = name(opener);
  return (
    Array.from(around.querySelectorAll<HTMLElement>(opener.localName)).find((el) => name(el) === wanted) ?? null
  );
}

/** A question in the page's flow (deleting something, signing out): the
 * question shown first (`question`, its name; or `label` when the question
 * is already in the text), the text that explains it (`children`, its
 * description), then the buttons that answer it. The last action is the
 * safe one ("Annulla") and takes the focus when the question appears; Esc
 * calls `onCancel`. Once it closes, the focus goes back to the control
 * that asked it ([`focusBack`]). `row` puts it all on one line, as in a bar
 * or a list row; `textClassName` styles the text. */
export function InlineConfirm({
  question,
  label,
  children,
  actions,
  onCancel,
  row = false,
  className = "",
  textClassName = "",
}: {
  question?: string;
  label?: string;
  children?: ReactNode;
  actions: ReactNode;
  onCancel?: () => void;
  row?: boolean;
  className?: string;
  textClassName?: string;
}) {
  const id = useId();
  const box = useRef<HTMLDivElement>(null);
  const answers = useRef<HTMLDivElement>(null);
  // The control that asked the question (read while rendering, before the
  // question takes its place).
  const [opener] = useState(() =>
    document.activeElement instanceof HTMLElement && document.activeElement !== document.body
      ? document.activeElement
      : null,
  );
  useEffect(() => {
    const node = box.current;
    const around = node?.parentElement ?? null;
    if (answers.current) tabbable(answers.current).at(-1)?.focus();
    return () => {
      // Still in the page: a remount (StrictMode), not a close. A choice
      // that moved the focus elsewhere keeps it there.
      if (node?.isConnected) return;
      const at = document.activeElement;
      if (at === null || at === document.body) focusBack(opener, around)?.focus();
    };
  }, [opener]);
  const Text = row ? "span" : "div";
  // `false` and `null` children (a part of the text not needed) are no text.
  const described = Children.toArray(children).length > 0;
  return (
    <div
      ref={box}
      role="alertdialog"
      aria-labelledby={question ? `${id}-q` : undefined}
      aria-label={question ? undefined : label}
      aria-describedby={described ? `${id}-desc` : undefined}
      onKeyDown={(e) => {
        if (e.key !== "Escape" || !onCancel) return;
        e.stopPropagation();
        onCancel();
      }}
      className={`${row ? "flex flex-wrap items-center gap-2" : "space-y-2"} ${className}`}
    >
      {question && (
        <Text id={`${id}-q`} className={described ? "" : textClassName}>
          {question}
        </Text>
      )}
      {described && (
        <Text id={`${id}-desc`} className={textClassName}>
          {children}
        </Text>
      )}
      <div ref={answers} className="flex flex-wrap gap-2">
        {actions}
      </div>
    </div>
  );
}
