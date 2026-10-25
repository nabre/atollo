// "Rimuovi l'unità": lists what is still
// open from the kit, stops it, puts Antigravity's login back into the kit,
// then closes the app so the drive can be removed safely. The same list is
// shown when the window is closed with something open (CloseDialog).
import { useState } from "react";

import { appQuit, ejectList, ejectRun, type OpenNow } from "../ipc";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { Codicon, codicon, ConfirmDialog, EditorBody, EditorSection, Notice, SingleEditor } from "../workbench";
import { t } from "../i18n";

function nothingOpen(o: OpenNow): boolean {
  return (
    o.processes.length === 0 &&
    !o.editor &&
    o.terminals === 0 &&
    o.actions === 0 &&
    !o.setup &&
    o.databases === 0 &&
    o.llm === 0 &&
    o.downloads === 0
  );
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
      {open.llm > 0 && <li>{t("Il server dei modelli locali")}</li>}
      {open.downloads > 0 && <li>{t("Modelli in scaricamento: {0}", open.downloads)}</li>}
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

export function EjectPage({ onGuide }: { onGuide?: (topic: "eject") => void } = {}) {
  // What is open: a read that works clears the error of the one before.
  const {
    data: open,
    error: loadError,
    reload: load,
    setData: setOpen,
  } = useLoad<OpenNow | null>(ejectList, null, { onError: "show" });
  const [done, setDone] = useState(false);
  const { busy, error, run } = useAction();

  const title = t("Rimuovi l'unità");
  const icon = <Codicon name="debug-disconnect" size={16} />;
  if (done) {
    return (
      <SingleEditor title={title} icon={icon}>
        <EditorBody>
          <Notice tone="success">{t("Programmi del kit chiusi e accesso di Antigravity salvato nel kit.")}</Notice>
          {open && !nothingOpen(open) && (
            <>
              <p className="text-vsc-warning">{t("Ancora aperti (chiudili a mano):")}</p>
              <OpenList open={open} />
            </>
          )}
          <p>{t("Ora chiudi Atollo, poi rimuovi l'unità con «Rimozione sicura dell'hardware» di Windows.")}</p>
          <ActionButton
            icon={codicon("close")}
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
      <EditorBody>
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
          <ActionButton icon={codicon("refresh")} label={t("Aggiorna")} onClick={load} />
          {onGuide && (
            <ActionButton
              icon={codicon("book")}
              label={t("Guida")}
              onClick={() => {
                onGuide("eject");
              }}
            />
          )}
          <ActionButton
            icon={codicon("debug-disconnect")}
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
  return (
    <ConfirmDialog
      title={t("Chiudere Atollo?")}
      cancelLabel={t("Annulla")}
      onCancel={onCancel}
      actions={
        <ActionButton
          icon={codicon("close")}
          label={t("Chiudi tutto")}
          primary
          busy={busy === "quit"}
          onClick={() => {
            run("quit", appQuit);
          }}
        />
      }
      footer={<ErrorLine error={error} />}
    >
      <p>{t("Chiudendo si chiudono anche:")}</p>
      <OpenList open={open} />
    </ConfirmDialog>
  );
}
