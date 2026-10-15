// Confirmations: the modal one over the whole window (VSCodium's dialog
// box) and the inline one under the button that asked it.
import { useEffect, useId, useRef, type ReactNode } from "react";

/** A modal question: the cancel button has the focus, Esc cancels. The
 * confirming buttons and an error line come from the caller (`actions`,
 * `footer`). */
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
  const cancelRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    cancelRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [onCancel]);
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-desc`}
        className="w-[min(32rem,90vw)] space-y-2 rounded border border-vsc-widget-border bg-vsc-widget p-3 shadow-[0_0_8px_2px_var(--vsc-shadow)]"
      >
        <h2 id={`${id}-title`} className="text-[14px] font-semibold">
          {title}
        </h2>
        <div id={`${id}-desc`} className="space-y-2">
          {children}
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <button
            ref={cancelRef}
            type="button"
            onClick={onCancel}
            className="min-h-[26px] rounded border border-vsc-button-border bg-vsc-button2 px-[11px] py-1 leading-[18px] text-vsc-button2-fg hover:bg-vsc-button2-hover"
          >
            {cancelLabel}
          </button>
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
