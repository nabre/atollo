// The Database section of a project's page: its own database and user on
// the kit's servers, written in .env, and their copies.
import { useState } from "react";
import { VscCode } from "react-icons/vsc";
import type { ProjectInfo } from "../api";
import { useAction } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { t } from "../i18n";
import { VscArrowLeft } from "react-icons/vsc";
import {
  projectDbCreate,
  projectEnvApply,
  projectEnvPreview,
  type EnvChange,
  type ProjectDatabases,
  type ProjectDb,
} from "../api";
import { SecretField } from "../databases";
import { DatabaseCopies } from "../backups";
import { DataTable, EditorSection, Notice, Properties } from "../workbench";
import { COMPONENT_NAMES } from "./needs";

function EnvPreview({
  project,
  db,
  envIgnored,
  onDone,
}: {
  project: string;
  db: ProjectDb;
  envIgnored: boolean;
  onDone: () => void;
}) {
  const [changes, setChanges] = useState<EnvChange[] | null>(null);
  const { busy, error, run } = useAction(onDone);
  return (
    <div className="space-y-2">
      {changes === null ? (
        <ActionButton
          icon={VscCode}
          label={t("Scrivi in .env")}
          busy={busy === "preview"}
          onClick={() => {
            run("preview", async () => {
              setChanges(await projectEnvPreview(project, db.engine));
            });
          }}
        />
      ) : changes.length === 0 ? (
        <Notice tone="success">{t("Il file .env ha già questa connessione.")}</Notice>
      ) : (
        <>
          <p className="text-vsc-desc">{t("Righe di .env che cambiano:")}</p>
          <div className="overflow-x-auto rounded border border-vsc-border">
            <DataTable className="w-full text-[12px]">
              {changes.map((c) => (
                <tr key={c.key} className="border-t border-vsc-border first:border-t-0">
                  <td className="px-2 py-1 font-mono">{c.key}</td>
                  <td className="px-2 py-1 font-mono text-vsc-desc line-through">{c.old ?? ""}</td>
                  <td className="px-2 py-1 font-mono">
                    {c.key.includes("PASSWORD") || c.key.includes("URI") || c.key.includes("URL") ? "••••" : c.new}
                  </td>
                </tr>
              ))}
            </DataTable>
          </div>
          {!envIgnored && (
            <Notice tone="warning">
              {t(".env non è in .gitignore: con la password dentro finirebbe nel repository. Aggiungilo a .gitignore.")}
            </Notice>
          )}
          <div className="flex gap-2">
            <ActionButton
              icon={VscCode}
              label={t("Scrivi")}
              primary
              busy={busy === "apply"}
              onClick={() => {
                run("apply", async () => {
                  await projectEnvApply(project, db.engine);
                  setChanges([]);
                });
              }}
            />
            <ActionButton
              icon={VscArrowLeft}
              label={t("Annulla")}
              onClick={() => {
                setChanges(null);
              }}
            />
          </div>
        </>
      )}
      <ErrorLine error={error} />
    </div>
  );
}

export function Databases({
  p,
  dbs,
  onChanged,
}: {
  p: ProjectInfo;
  dbs: ProjectDatabases | null;
  onChanged: () => void;
}) {
  const { busy, error, run } = useAction(onChanged);
  if (dbs === null) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  const engines: ProjectDb["engine"][] = ["mariadb", "mongodb"];
  return (
    <>
      {engines.map((engine) => {
        const db = dbs.dbs.find((d) => d.engine === engine);
        const used = dbs.detected.includes(engine);
        return (
          <EditorSection
            key={engine}
            id={`pdb-${engine}`}
            title={COMPONENT_NAMES[engine] ?? engine}
            actions={used ? <span className="text-[12px] text-vsc-desc">{t("usato dal progetto")}</span> : undefined}
          >
            {db ? (
              <div className="space-y-3">
                <Properties
                  rows={[
                    [t("Database"), <span className="font-mono">{db.database}</span>],
                    [t("Utente"), <span className="font-mono">{db.user}</span>],
                    [t("Password"), <SecretField value={db.password} label={t("la password di {0}", db.user)} />],
                    [
                      t("Indirizzo"),
                      <span className="font-mono">
                        {db.host}:{String(db.port)}
                      </span>,
                    ],
                    [t("Stringa"), <SecretField value={db.url} label={t("la stringa di connessione")} />],
                  ]}
                />
                <EnvPreview project={p.id} db={db} envIgnored={dbs.envIgnored} onDone={onChanged} />
                <DatabaseCopies project={p.id} engine={db.engine} />
              </div>
            ) : (
              <div className="flex flex-wrap items-center gap-3">
                <p className="min-w-0 flex-1 text-vsc-desc">
                  {t("Un database e un utente solo per questo progetto, con una password generata dal kit.")}
                </p>
                <ActionButton
                  icon={VscCode}
                  label={t("Crea database e utente")}
                  primary={used}
                  busy={busy === engine}
                  onClick={() => {
                    run(engine, async () => {
                      await projectDbCreate(p.id, engine);
                    });
                  }}
                />
              </div>
            )}
          </EditorSection>
        );
      })}
      {dbs.detected.includes("sqlite") && (
        <EditorSection id="pdb-sqlite" title="SQLite">
          <p className="text-vsc-desc">
            {t("Il progetto usa SQLite: il database è un file del progetto, nessun server da avviare.")}
          </p>
        </EditorSection>
      )}
      <ErrorLine error={error} />
    </>
  );
}
