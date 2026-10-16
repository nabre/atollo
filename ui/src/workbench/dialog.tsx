// Confirmations: the modal one over the whole window (VSCodium's dialog
// box) and the inline one under the button that asked it.
import { useEffect, useId, useRef, type ReactNode } from "react";
import { Button } from "./controls";

const FOCUSABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** A modal question: the cancel button (a vscode-button) has the focus,
 * Esc cancels. The confirming buttons and an error line come from the
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
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onCancel();
        return;
      }
      if (e.key === "Tab" && dialogRef.current) {
        const items = Array.from(dialogRef.current.querySelectorAll<HTMLElement>(FOCUSABLE));
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
