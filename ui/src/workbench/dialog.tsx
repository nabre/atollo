// Confirmations: the modal one over the whole window (VSCodium's dialog
// box) and the inline one under the button that asked it.
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
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

/** A modal question: the cancel button (a vscode-button) has the focus,
 * Tab stays inside, Esc cancels; once closed, the focus goes back where it
 * was. The confirming buttons and an error line come from the
 * caller (`actions`, `footer`). */
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
  // Where the focus was before the dialog took it (read while rendering,
  // before the cancel button's autoFocus).
  const [opener] = useState(() => (document.activeElement instanceof HTMLElement ? document.activeElement : null));
  useEffect(() => {
    const node = dialogRef.current;
    return () => {
      // Still in the page: a remount (StrictMode), not a close. Otherwise
      // the focus fell to the body with the dialog; a confirm that moved it
      // elsewhere keeps it there.
      if (node?.isConnected) return;
      const at = document.activeElement;
      if (opener?.isConnected && (at === null || at === document.body)) opener.focus();
    };
  }, [opener]);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onCancel();
        return;
      }
      if (e.key === "Tab" && dialogRef.current) {
        const items = tabbable(dialogRef.current);
        if (items.length === 0) return;
        const first = items[0];
        const last = items[items.length - 1];
        if (e.shiftKey) {
          if (document.activeElement === first || !dialogRef.current.contains(document.activeElement)) {
            e.preventDefault();
            last?.focus();
          }
        } else {
          if (document.activeElement === last || !dialogRef.current.contains(document.activeElement)) {
            e.preventDefault();
            first?.focus();
          }
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [onCancel]);
  return (
    // The dimmed window: no theme color for it in VSCodium either.
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div
        ref={dialogRef}
        role="alertdialog"
        aria-modal="true"
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

/** A question in the page's flow (deleting something): the message, then
 * the buttons that answer it. */
export function InlineConfirm({
  label,
  children,
  actions,
}: {
  label: string;
  children: ReactNode;
  actions: ReactNode;
}) {
  return (
    <div role="alertdialog" aria-label={label} className="space-y-2">
      {children}
      <div className="flex flex-wrap gap-2">{actions}</div>
    </div>
  );
}
