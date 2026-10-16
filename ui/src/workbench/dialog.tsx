// Confirmations: the modal one over the whole window (VSCodium's dialog
// box) and the inline one under the button that asked it.
import { useEffect, useId, useRef, useState, type ReactNode, type RefObject } from "react";
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
