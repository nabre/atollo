// "Rimuovi l'unità" (port of nabre/atollo Eject.ps1): lists what is still
// open from the kit, stops it, puts Antigravity's login back into the kit,
// then closes the app so the drive can be removed safely. The same list is
// shown when the window is closed with something open (CloseDialog).
import { useCallback, useEffect, useRef, useState } from "react";
import { VscCheck, VscClose, VscDebugDisconnect, VscRefresh, VscWarning } from "react-icons/vsc";
import { appQuit, ejectList, ejectRun, type OpenNow } from "./api";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { t } from "./i18n";

function nothingOpen(o: OpenNow): boolean {
  return o.processes.length === 0 && !o.editor && o.terminals === 0 && o.actions === 0 && !o.setup;
}

export function OpenList({ open }: { open: OpenNow }) {
  if (nothingOpen(open)) return <p className="text-vsc-success">{t("Nessun programma del kit aperto.")}</p>;
  return (
    <ul className="list-disc space-y-1 pl-5">
      {open.setup && <li>{t("Un'installazione di componenti è in corso")}</li>}
      {open.editor && <li>{t("L'editor")}</li>}
      {open.terminals > 0 && <li>{t("Console del kit: {0}", open.terminals)}</li>}
      {open.actions > 0 && <li>{t("Progetti avviati: {0}", open.actions)}</li>}
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

  if (done) {
    return (
      <div className="max-w-3xl space-y-4">
        <p role="status" className="flex items-start gap-2 text-vsc-success">
          <VscCheck size={16} aria-hidden="true" className="mt-0.5 shrink-0" />
          {t("Programmi del kit chiusi e accesso di Antigravity salvato nel kit.")}
        </p>
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
      </div>
    );
  }

  return (
    <div className="max-w-3xl space-y-4">
      <p>
        {t(
          "Prima di togliere la chiavetta o il disco del kit: chiude i programmi del kit ancora aperti e salva nel kit l'accesso di Antigravity, così non resta nulla in questo PC.",
        )}
      </p>
      <ErrorLine error={loadError} />
      {open === null ? (
        <p className="text-vsc-desc">{t("Caricamento…")}</p>
      ) : (
        <section aria-labelledby="eject-open" className="space-y-2">
          <h2 id="eject-open" className="text-[13px] font-semibold">
            {t("Aperti adesso")}
          </h2>
          <OpenList open={open} />
          {!nothingOpen(open) && (
            <p className="flex items-start gap-2 text-vsc-warning">
              <VscWarning size={16} aria-hidden="true" className="mt-0.5 shrink-0" />
              {t("I file non salvati in questi programmi andranno persi.")}
            </p>
          )}
        </section>
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
    </div>
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
        className="w-[min(32rem,90vw)] space-y-3 rounded border border-vsc-border bg-vsc-sidebar p-5 shadow-lg"
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
            className="rounded-sm bg-vsc-button2 px-3 py-1.5 text-vsc-button2-fg hover:bg-vsc-button2-hover"
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
