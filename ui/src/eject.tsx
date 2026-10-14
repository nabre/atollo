// "Rimuovi l'unità" (port of nabre/atollo Eject.ps1): lists what is still
// open from the kit, stops it, puts Antigravity's login back into the kit,
// then closes the app so the drive can be removed safely. The same list is
// shown when the window is closed with something open (CloseDialog).
import { useCallback, useEffect, useRef, useState } from "react";
import { VscClose, VscDebugDisconnect, VscRefresh } from "react-icons/vsc";

import { appQuit, ejectList, ejectRun, type OpenNow } from "./api";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { EditorBody, EditorSection, Notice, SingleEditor } from "./workbench";
import { t } from "./i18n";

function nothingOpen(o: OpenNow): boolean {
  return o.processes.length === 0 && !o.editor && o.terminals === 0 && o.actions === 0 && !o.setup && o.databases === 0;
}

export function OpenList({ open }: { open: OpenNow }) {
  if (nothingOpen(open)) return <p className="text-vsc-success">{t("Nessun programma del kit aperto.")}</p>;
  return (
    <ul className="list-disc space-y-1 pl-5">
      {open.setup && <li>{t("Un'installazione di componenti è in corso")}</li>}
      {open.editor && <li>{t("L'editor")}</li>}
      {open.terminals > 0 && <li>{t("Console del kit: {0}", open.terminals)}</li>}
      {open.actions > 0 && <li>{t("Progetti avviati: {0}", open.actions)}</li>}
      {open.databases > 0 && <li>{t("Database avviati: {0}", open.databases)}</li>}
      {open.processes.map((p) => (
        <li key={p.pid}>
          <span>{p.name}</span>{" "}
          <span className="font-mono text-[12px] text-vsc-desc select-text">
            {p.program} · PID {p.pid}
          </span>
        </li>
      ))}
    </ul>
  );
}

export function EjectPage() {
  const [open, setOpen] = useState<OpenNow | null>(null);
  const [done, setDone] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const { busy, error, run } = useAction();
  const load = useCallback(() => {
    ejectList().then(setOpen, (e: unknown) => {
      setLoadError(String(e));
    });
  }, []);
  useEffect(load, [load]);

  const title = t("Rimuovi l'unità");
  const icon = <VscDebugDisconnect size={16} aria-hidden="true" />;
  if (done) {
    return (
      <SingleEditor title={title} icon={icon}>
        <EditorBody narrow>
          <Notice tone="success">{t("Programmi del kit chiusi e accesso di Antigravity salvato nel kit.")}</Notice>
          {open && !nothingOpen(open) && (
            <>
              <p className="text-vsc-warning">{t("Ancora aperti (chiudili a mano):")}</p>
              <OpenList open={open} />
            </>
          )}
          <p>{t("Ora chiudi Atollo, poi rimuovi l'unità con «Rimozione sicura dell'hardware» di Windows.")}</p>
          <ActionButton
            icon={VscClose}
            label={t("Chiudi Atollo")}
            primary
            onClick={() => {
              run("quit", appQuit);
            }}
          />
          <ErrorLine error={error} />
        </EditorBody>
      </SingleEditor>
    );
  }

  return (
    <SingleEditor title={title} icon={icon}>
      <EditorBody narrow>
        <p>
          {t(
            "Prima di togliere la chiavetta o il disco del kit: chiude i programmi del kit ancora aperti e salva nel kit l'accesso di Antigravity, così non resta nulla in questo PC.",
          )}
        </p>
        <ErrorLine error={loadError} />
        {open === null ? (
          <p className="text-vsc-desc">{t("Caricamento…")}</p>
        ) : (
          <EditorSection id="eject-open" title={t("Aperti adesso")}>
            <OpenList open={open} />
            {!nothingOpen(open) && (
              <Notice tone="warning">{t("I file non salvati in questi programmi andranno persi.")}</Notice>
            )}
          </EditorSection>
        )}
        <div className="flex flex-wrap gap-2">
          <ActionButton icon={VscRefresh} label={t("Aggiorna")} onClick={load} />
          <ActionButton
            icon={VscDebugDisconnect}
            label={t("Chiudi tutto e prepara la rimozione")}
            primary
            busy={busy === "eject"}
            onClick={() => {
              run("eject", async () => {
                setOpen(await ejectRun());
                setDone(true);
              });
            }}
          />
        </div>
        <ErrorLine error={error} />
      </EditorBody>
    </SingleEditor>
  );
}

/** Asked when the window is closed with something still open. */
export function CloseDialog({ open, onCancel }: { open: OpenNow; onCancel: () => void }) {
  const { busy, error, run } = useAction();
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
        aria-labelledby="close-title"
        aria-describedby="close-desc"
        className="w-[min(32rem,90vw)] space-y-2 rounded border border-vsc-widget-border bg-vsc-widget p-3 shadow-[0_0_8px_2px_var(--vsc-shadow)]"
      >
        <h2 id="close-title" className="text-[14px] font-semibold">
          {t("Chiudere Atollo?")}
        </h2>
        <div id="close-desc" className="space-y-2">
          <p>{t("Chiudendo si chiudono anche:")}</p>
          <OpenList open={open} />
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <button
            ref={cancelRef}
            type="button"
            onClick={onCancel}
            className="min-h-[26px] rounded border border-vsc-button-border bg-vsc-button2 px-[11px] py-1 leading-[18px] text-vsc-button2-fg hover:bg-vsc-button2-hover"
          >
            {t("Annulla")}
          </button>
          <ActionButton
            icon={VscClose}
            label={t("Chiudi tutto")}
            primary
            busy={busy === "quit"}
            onClick={() => {
              run("quit", appQuit);
            }}
          />
        </div>
        <ErrorLine error={error} />
      </div>
    </div>
  );
}
