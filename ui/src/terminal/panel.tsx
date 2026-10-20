// The terminal panel of macOS and Linux (D-109): at the bottom of every
// page, as VSCodium's terminal, a tab per session opened by «Terminale» or
// an agent's button; closing a tab whose program still runs asks first.
// Its own chunk with xterm.js, read the first time a session opens: on
// Windows no session opens (the consoles are windows of their own), so
// the panel and xterm.js are never loaded there.
import { useState } from "react";
import type { PtyBackend, PtyInfo } from "./api";
import type { Terminals } from "./state";
import { TerminalView } from "./view";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { codicon, ConfirmDialog, IconButton, Panel } from "../workbench";

export function TerminalPanel({
  terminals,
  backend,
  height,
}: {
  terminals: Terminals;
  backend: PtyBackend;
  height: number;
}) {
  const { sessions, active, setActive, setShown, close } = terminals;
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
        }))}
        active={current ? String(current.id) : ""}
        onSelect={(id) => {
          setActive(Number(id));
        }}
        actions={
          <>
            {current && (
              <IconButton
                icon={codicon("close")}
                label={t("Chiudi il terminale")}
                onClick={() => {
                  ask(current);
                }}
              />
            )}
            <IconButton
              icon={codicon("chrome-minimize")}
              label={t("Nascondi i terminali")}
              onClick={() => {
                setShown(false);
              }}
            />
          </>
        }
      >
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {/* A tab whose program ended closes without the dialog: its
              error shows here. */}
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
