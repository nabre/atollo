// The move of MariaDB's data to MySQL on the Database page (DECISIONS.md
// D-117): what waits and where it goes, the error with the dump kept and
// "Riprova", then the MariaDB data kept after the move with "Elimina".
import { useState } from "react";
import { VscClose, VscRefresh, VscTrash } from "react-icons/vsc";
import { dbMigration, dbMigrationDropKept, dbMigrationRetry, type DbMigration } from "../api";
import { locale, t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { useAction, useLoad } from "../hooks";
import { InlineConfirm, Notice } from "../workbench";

function megabytes(bytes: number): string {
  return new Intl.NumberFormat(locale(), { maximumFractionDigits: 0 }).format(Math.max(1, bytes / 1024 / 1024));
}

/** The text of the move for the Activity panel; null: nothing to tell. */
export function migrationText(m: DbMigration | null | undefined): string | null {
  switch (m?.state) {
    case "running":
      return t("I dati di MariaDB passano a MySQL…");
    case "failed":
      return t("I dati di MariaDB non sono passati a MySQL: MariaDB resta attivo.");
    case "pending":
      return t("I dati di MariaDB passano a MySQL al prossimo avvio.");
    default:
      return null;
  }
}

export function MigrationPanel() {
  const { data: m, setData } = useLoad<DbMigration | null>(dbMigration, null, { onError: "keep", every: 3000 });
  const { busy, error, run } = useAction();
  const [confirm, setConfirm] = useState(false);
  if (!m || m.state === "none") return null;
  const waiting = m.state !== "done";
  return (
    <section aria-label={t("Dati di MariaDB")} className="space-y-2 border-b border-vsc-border px-5 py-3">
      {waiting ? (
        <>
          <Notice tone={m.state === "failed" ? "error" : "info"}>
            <span>
              {migrationText(m)}{" "}
              {t(
                "MySQL è il server SQL del kit: ogni versione di MariaDB passa a una di MySQL, sulla stessa porta, con gli stessi utenti e password.",
              )}
            </span>
          </Notice>
          {m.lines.length > 0 && (
            <ul className="ml-6 list-disc text-vsc-desc">
              {m.lines.map((l) => (
                <li key={l.mariadb}>
                  {t("MariaDB {0} → MySQL {1}, porta {2}", l.mariadb, l.mysql, String(l.port))}
                  {l.done ? ` · ${t("copiato")}` : ""}
                </li>
              ))}
            </ul>
          )}
          {m.error && <pre className="m-0 font-mono text-[12px] whitespace-pre-wrap text-vsc-error">{m.error}</pre>}
          {m.dump && (
            <p className="text-[12px] text-vsc-desc">
              {t("Copia dei dati salvata in")} <code className="font-mono select-text">{m.dump}</code>
            </p>
          )}
          {m.state !== "running" && (
            <ActionButton
              icon={VscRefresh}
              label={m.state === "failed" ? t("Riprova") : t("Sposta ora")}
              busy={busy === "retry"}
              onClick={() => {
                run("retry", () => dbMigrationRetry().then(setData));
              }}
            />
          )}
        </>
      ) : confirm ? (
        <InlineConfirm
          row
          question={t("Eliminare i dati MariaDB conservati?")}
          textClassName="text-vsc-warning"
          onCancel={() => {
            setConfirm(false);
          }}
          actions={
            <>
              <ActionButton
                icon={VscTrash}
                label={t("Elimina")}
                primary
                busy={busy === "drop"}
                onClick={() => {
                  setConfirm(false);
                  run("drop", () => dbMigrationDropKept().then(setData));
                }}
              />
              <ActionButton
                icon={VscClose}
                label={t("Annulla")}
                onClick={() => {
                  setConfirm(false);
                }}
              />
            </>
          }
        >
          {t("I database sono già in MySQL: si cancellano la cartella e le copie dei file di prima.")}
        </InlineConfirm>
      ) : (
        <div className="flex flex-wrap items-center gap-2">
          <span className="flex-1">
            {t("Dati MariaDB conservati ({0} MB)", megabytes(m.keptBytes))}
            {m.kept && <code className="ml-2 font-mono text-[12px] text-vsc-desc select-text">{m.kept}</code>}
          </span>
          <ActionButton
            icon={VscTrash}
            label={t("Elimina")}
            small
            onClick={() => {
              setConfirm(true);
            }}
          />
        </div>
      )}
      <ErrorLine error={error} />
    </section>
  );
}
