// Shared pieces of the pages (buttons, actions with their error) and the
// editor's state as a hint. The controls themselves are in workbench/.
import { useCallback, useState } from "react";
import type { IconType } from "react-icons";
import type { EditorStatus } from "./api";
import { t } from "./i18n";
import { Button, IconButton, Notice } from "./workbench";

/** Runs an action, keeping track of what is busy and of the last error. */
export function useAction(onDone?: () => void) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = useCallback(
    (key: string, action: () => Promise<void>) => {
      setBusy(key);
      setError(null);
      action().then(
        () => {
          setBusy(null);
          onDone?.();
        },
        (e: unknown) => {
          setBusy(null);
          setError(String(e));
        },
      );
    },
    [onDone],
  );
  return { busy, error, run };
}

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
  title?: string | undefined;
}) {
  if (compact) {
    return <IconButton icon={icon} label={label} onClick={onClick} busy={busy} disabled={disabled} title={title} />;
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
    >
      {busy ? `${label}…` : label}
    </Button>
  );
}

export function ErrorLine({ error }: { error: string | null }) {
  if (!error) return null;
  return <Notice tone="error">{error}</Notice>;
}

export function editorHint(status: EditorStatus | null): string | undefined {
  if (!status) return undefined;
  if (!status.installed) return t("L'editor non è installato: installalo da Strumenti.");
  if (status.emulated) return t("Su questo PC (ARM64) l'editor gira in emulazione x64: può essere più lento.");
  return undefined;
}
