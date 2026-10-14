// Shared pieces of the pages (buttons, actions with their error) and the
// Home actions: open the integrated editor and the kit's console on the
// projects folder. The Projects page is in projects.tsx.
import { useCallback, useState } from "react";
import type { IconType } from "react-icons";
import { VscCode, VscTerminal, VscWarning } from "react-icons/vsc";
import { editorOpen, terminalOpen, type EditorStatus } from "./api";
import { t } from "./i18n";

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

export function ActionButton({
  icon: Icon,
  label,
  onClick,
  busy = false,
  disabled = false,
  primary = false,
  submit = false,
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
  title?: string | undefined;
}) {
  return (
    <button
      type={submit ? "submit" : "button"}
      onClick={submit ? undefined : onClick}
      disabled={disabled || busy}
      title={title}
      aria-busy={busy}
      className={[
        "flex items-center gap-2 rounded-sm px-3 py-1.5 text-[13px] disabled:cursor-not-allowed disabled:opacity-50",
        primary
          ? "bg-vsc-button text-vsc-button-fg enabled:hover:bg-vsc-button-hover"
          : "bg-vsc-button2 text-vsc-button2-fg enabled:hover:bg-vsc-button2-hover",
      ].join(" ")}
    >
      <Icon size={16} aria-hidden="true" className={busy ? "animate-pulse" : undefined} />
      {busy ? `${label}…` : label}
    </button>
  );
}

export function ErrorLine({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <p role="alert" className="flex items-start gap-2 text-vsc-error">
      <VscWarning className="mt-0.5 shrink-0" size={16} aria-hidden="true" />
      <span className="select-text">{error}</span>
    </p>
  );
}

function editorHint(status: EditorStatus | null): string | undefined {
  if (!status) return undefined;
  if (!status.installed) return t("L'editor non è installato: installalo da Linguaggi e strumenti.");
  if (status.emulated) return t("Su questo PC (ARM64) l'editor gira in emulazione x64: può essere più lento.");
  return undefined;
}

export function HomeActions({
  status,
  setupRunning,
  onChanged,
}: {
  status: EditorStatus | null;
  setupRunning: boolean;
  onChanged: () => void;
}) {
  const { busy, error, run } = useAction(onChanged);
  const hint = setupRunning
    ? t("Installazione dei componenti del kit in corso: editor e terminale si aprono quando finisce.")
    : editorHint(status);
  return (
    <section aria-labelledby="start" className="space-y-3">
      <h2 id="start" className="text-[13px] font-semibold">
        {t("Inizia")}
      </h2>
      <div className="flex flex-wrap gap-2">
        <ActionButton
          icon={VscCode}
          label={t("Editor")}
          primary
          busy={busy === "editor"}
          title={hint ?? t("Apre VSCodium sulla cartella dei progetti")}
          onClick={() => {
            run("editor", () => editorOpen(null));
          }}
        />
        <ActionButton
          icon={VscTerminal}
          label={t("Terminale")}
          busy={busy === "terminal"}
          title={t("Console del kit con PowerShell 7 nella cartella dei progetti")}
          onClick={() => {
            run("terminal", () => terminalOpen(null));
          }}
        />
      </div>
      {hint && <p className="text-vsc-desc">{hint}</p>}
      <ErrorLine error={error} />
    </section>
  );
}
