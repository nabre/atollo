// The terminal panel (D-109, D-148): at the bottom of every page, as
// VSCodium's terminal, a tab per session opened by «Terminale», a terminal
// of the Terminals page or an agent's button; closing a tab whose program
// still runs asks first. Its own chunk with xterm.js, read the first time a
// session opens. A tab moves into a window of its own (D-149), and comes
// back from it.
import { useState } from "react";
import type { PtyBackend, PtyInfo } from "../ipc/pty";
import type { Terminals } from "./state";
import { TerminalView } from "./view";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../shared/common";
import { labelText } from "../shared/levels";
import { codicon, ConfirmDialog, IconButton, Panel } from "../workbench";

export function TerminalPanel({
  terminals,
  backend,
  height,
  detached = false,
}: {
  terminals: Terminals;
  backend: PtyBackend;
  /** None: the panel fills its window. */
  height?: number | undefined;
  /** The window of a detached session (window.tsx): its one tab, which
   * goes back to the main window's panel. */
  detached?: boolean;
}) {
  const { active, setActive, setShown, close, detach } = terminals;
  const sessions = detached ? terminals.sessions : terminals.docked;
  const [asking, setAsking] = useState<PtyInfo | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const current = sessions.find((s) => s.id === active) ?? sessions.at(-1);

  const doClose = (id: number) => {
    setBusy(true);
    setError(null);
    close(id).then(
      () => {
        setBusy(false);
        setAsking(null);
      },
      (e: unknown) => {
        setBusy(false);
        setError(String(e));
      },
    );
  };
  const ask = (s: PtyInfo) => {
    setError(null);
    if (s.running) setAsking(s);
    else doClose(s.id);
  };

  return (
    <>
      <Panel
        height={height}
        tabs={sessions.map((s) => ({
          id: String(s.id),
          label: s.running ? s.title : t("{0} (terminato)", s.title),
          // Where it runs (D-179, LEVELS.md §8.3), the sandbox on hover.
          tag: labelText(s.label),
          hint: s.sandbox ? t("Gira in {0}", s.sandbox) : undefined,
        }))}
        active={current ? String(current.id) : ""}
        onSelect={(id) => {
          setActive(Number(id));
        }}
        actions={
          <>
            {current && (
              <IconButton
                icon={codicon(detached ? "layout-panel" : "multiple-windows")}
                label={detached ? t("Riporta nel pannello") : t("Sposta in una nuova finestra")}
                onClick={() => {
                  setError(null);
                  (detached ? backend.dock(current.id) : detach(current.id)).catch((e: unknown) => {
                    setError(String(e));
                  });
                }}
              />
            )}
            {current && (
              <IconButton
                icon={codicon("close")}
                label={t("Chiudi il terminale")}
                onClick={() => {
                  ask(current);
                }}
              />
            )}
            {!detached && (
              <IconButton
                icon={codicon("chrome-minimize")}
                label={t("Nascondi i terminali")}
                onClick={() => {
                  setShown(false);
                }}
              />
            )}
          </>
        }
      >
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {/* A tab whose program ended closes without the dialog, and a
              tab that could not move: the error shows here. */}
          {!asking && error && (
            <div className="shrink-0 px-2 pt-1">
              <ErrorLine error={error} />
            </div>
          )}
          <div className="flex min-h-0 flex-1">
            {sessions.map((s) => (
              <TerminalView key={s.id} session={s} backend={backend} active={s.id === current?.id} />
            ))}
          </div>
        </div>
      </Panel>
      {asking && (
        <ConfirmDialog
          title={t("Chiudere il terminale?")}
          cancelLabel={t("Annulla")}
          onCancel={() => {
            setAsking(null);
            setError(null);
          }}
          actions={
            <ActionButton
              icon={codicon("close")}
              label={t("Chiudi il terminale")}
              primary
              busy={busy}
              onClick={() => {
                doClose(asking.id);
              }}
            />
          }
          footer={<ErrorLine error={error} />}
        >
          <p>{t("In «{0}» un programma è ancora in esecuzione: chiudendo il terminale si ferma.", asking.title)}</p>
        </ConfirmDialog>
      )}
    </>
  );
}
