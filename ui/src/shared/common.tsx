// Shared pieces of the pages (the button of an action, the line of its
// error) and the editor's state as a hint. The controls themselves are in
// workbench/.
import { useCallback, useEffect, useRef, useState } from "react";
import type { IconType } from "react-icons";
import type { EditorStatus } from "../ipc";
import { t } from "../i18n";
import { Button, codicon, IconButton, Notice, type MenuTrigger } from "../workbench";

/** A button of the pages: a text button (Button), or with `compact` an
 * icon action (IconButton) named by its label. */
export function ActionButton({
  icon,
  label,
  onClick,
  busy = false,
  disabled = false,
  primary = false,
  submit = false,
  compact = false,
  small = false,
  menu,
  autoFocus = false,
  title,
}: {
  icon: IconType;
  label: string;
  onClick: () => void;
  busy?: boolean;
  disabled?: boolean;
  primary?: boolean;
  /** The submit button of its form (onClick is not needed). */
  submit?: boolean;
  /** The icon alone; the label stays its accessible name. */
  compact?: boolean;
  /** The small button of a list row. */
  small?: boolean;
  /** Kept for the callers of the inline actions: compact is always ghost. */
  ghost?: boolean;
  /** With `compact`: it opens this menu (aria-haspopup, aria-expanded). */
  menu?: MenuTrigger | undefined;
  /** Without `compact`: takes the focus when shown. */
  autoFocus?: boolean;
  title?: string | undefined;
}) {
  if (compact) {
    return (
      <IconButton
        icon={icon}
        label={label}
        onClick={onClick}
        busy={busy}
        disabled={disabled}
        menu={menu}
        title={title}
      />
    );
  }
  return (
    <Button
      variant={primary ? "primary" : "secondary"}
      small={small}
      submit={submit}
      onClick={onClick}
      busy={busy}
      disabled={disabled}
      title={title}
      autoFocus={autoFocus}
    >
      {busy ? `${label}…` : label}
    </Button>
  );
}

export function ErrorLine({ error }: { error: string | null }) {
  if (!error) return null;
  return <Notice tone="error">{error}</Notice>;
}

/** A read that failed, with "Riprova" to read again. When `onRetry`
 * returns the read (useLoad's `retry`), the button is busy until it ends:
 * a read that fails again with the same error still shows it was done. */
export function LoadError({
  error,
  onRetry,
  className = "",
}: {
  error: string | null;
  onRetry: (() => void) | (() => Promise<unknown>);
  className?: string;
}) {
  const [busy, setBusy] = useState(false);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  if (!error) return null;
  const retry = () => {
    const read = onRetry();
    if (!(read instanceof Promise)) return;
    setBusy(true);
    const done = () => {
      if (mounted.current) setBusy(false);
    };
    read.then(done, done);
  };
  return (
    <Notice tone="error" className={className}>
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
        <span>{error}</span>
        <ActionButton icon={codicon("refresh")} label={t("Riprova")} small busy={busy} onClick={retry} />
      </div>
    </Notice>
  );
}

/** "Riprova" of a LoadError that, when the read works, puts the focus on
 * `target` (the heading of what came back): the button goes with the
 * error, and the focus would be lost on the page. `retry` is useLoad's. */
export function useRetryFocus<E extends HTMLElement>(retry: () => Promise<boolean>) {
  const target = useRef<E>(null);
  const [recovered, setRecovered] = useState(0);
  useEffect(() => {
    if (recovered === 0) return;
    const active = document.activeElement;
    // Only when the focus went with the button, not where the user moved it.
    if (active === null || active === document.body || !active.isConnected) target.current?.focus();
  }, [recovered]);
  const onRetry = useCallback(
    () =>
      retry().then((ok) => {
        if (ok) setRecovered((n) => n + 1);
      }),
    [retry],
  );
  return { target, onRetry };
}

export function editorHint(status: EditorStatus | null): string | undefined {
  if (!status) return undefined;
  if (!status.installed) return t("L'editor non è installato: installalo da Strumenti.");
  if (status.emulated) return t("Su questo Mac l'editor gira in emulazione x64 con Rosetta 2: può essere più lento.");
  return undefined;
}

/** The ports a page lets the user choose: above the system's own, up to the last. */
export const PORT_MIN = 1024;
export const PORT_MAX = 65535;

export function validPort(n: number): boolean {
  return Number.isInteger(n) && n >= PORT_MIN && n <= PORT_MAX;
}

/** What a port field says when the port is not valid. */
export function portRangeText(): string {
  return t("Una porta fra 1024 e 65535.");
}

/** A port typed in a field: the text, its number and whether it is valid. */
export function usePortField(initial: number) {
  const [text, setText] = useState(String(initial));
  const value = Number(text);
  return { text, setText, value, valid: validPort(value) };
}
