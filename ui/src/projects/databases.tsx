// The Database section of a project's page: its own database and user on
// the kit's servers, written in .env, and their copies.
import { useRef, useState } from "react";
import type { ProjectInfo } from "../api";
import { useAction, useCopy, useSubscription } from "../shared/hooks";
import { ActionButton, ErrorLine } from "../shared/common";
import { t } from "../i18n";
import {
  onSetup,
  projectDbCreate,
  projectDbLine,
  projectEnvApply,
  projectEnvBlock,
  projectEnvPreview,
  versionsAdd,
  versionsInstall,
  type DbVersions,
  type EnvChange,
  type ProjectDatabases,
  type ProjectDb,
} from "../api";
import { SecretField } from "../databases";
import { DatabaseCopies } from "../databases";
import { codicon, DataTable, EditorSection, Notice, Properties, Select } from "../workbench";
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
  const { copied, copy } = useCopy(1500);
  return (
    <div className="space-y-2">
      {changes === null ? (
        <div className="flex flex-wrap items-center gap-2">
          <ActionButton
            icon={codicon("code")}
            label={t("Scrivi in .env")}
            busy={busy === "preview"}
            onClick={() => {
              run("preview", async () => {
                setChanges(await projectEnvPreview(project, db.engine));
              });
            }}
          />
          <ActionButton
            icon={codicon("copy")}
            label={t("Copia blocco .env")}
            busy={busy === "block"}
            onClick={() => {
              // Read here, not on the first render: the block holds the password.
              run("block", async () => {
                copy(await projectEnvBlock(project, db.engine));
              });
            }}
          />
          {copied && <span className="text-vsc-desc">{t("Copiato")}</span>}
        </div>
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
              icon={codicon("code")}
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
              icon={codicon("arrow-left")}
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

/** The version of the server for the project's database, when more than
 * one is installed (D-102): each runs on its own port. */
function DbVersionSelect({
  project,
  engine,
  versions,
  hasDb,
  onChanged,
}: {
  project: string;
  engine: ProjectDb["engine"];
  versions: DbVersions | undefined;
  hasDb: boolean;
  onChanged: () => void;
}) {
  const { busy, error, run } = useAction(onChanged);
  // After "Install", the section is read again when the preparation ends.
  const installing = useRef<"start" | "run" | null>(null);
  useSubscription(onSetup, (s) => {
    if (installing.current === "start" && s.running) installing.current = "run";
    else if (installing.current === "run" && !s.running) {
      installing.current = null;
      onChanged();
    }
  });
  // The version chosen for the project and then uninstalled: start and
  // creation would fail with "not installed".
  const missing = versions?.chosen != null && !versions.lines.includes(versions.chosen) ? versions.chosen : null;
  if (!versions || (versions.lines.length < 2 && missing === null)) return null;
  const id = `pdb-line-${engine}`;
  const name = COMPONENT_NAMES[engine] ?? engine;
  return (
    <div className="mb-3 space-y-1">
      <label htmlFor={id} className="block font-semibold">
        {t("Versione del server")}
      </label>
      {missing !== null && (
        <Notice tone="warning">
          <div className="space-y-2">
            <p>
              {hasDb
                ? t(
                    "{0} {1} non è installata: il database del progetto è su questa versione, che non si avvia. Installala, o scegli un'altra versione (database e utente si creano anche lì, vuoti).",
                    name,
                    missing,
                  )
                : t(
                    "{0} {1} non è installata: installala, o scegli un'altra versione per il database del progetto.",
                    name,
                    missing,
                  )}
            </p>
            <ActionButton
              icon={codicon("cloud-download")}
              label={t("Installa {0} {1}", name, missing)}
              small
              busy={busy === "install"}
              onClick={() => {
                run("install", async () => {
                  installing.current = "start";
                  await (versions.lines.length === 0 ? versionsInstall : versionsAdd)(engine, missing);
                });
              }}
            />
          </div>
        </Notice>
      )}
      <p className="text-vsc-desc">
        {hasDb
          ? t(
              "Ogni versione ha la sua porta e i suoi dati: cambiandola, database e utente si creano anche sulla nuova, vuoti (i dati si portano con una copia).",
            )
          : t("Ogni versione ha la sua porta e i suoi dati.")}
      </p>
      <Select
        id={id}
        value={versions.chosen ?? ""}
        disabled={busy !== null}
        onChange={(line) => {
          if (line === missing) return;
          run("line", async () => {
            await projectDbLine(project, engine, line === "" ? null : line);
          });
        }}
        options={[
          { value: "", label: t("Quella del server ({0})", versions.server ?? "") },
          ...versions.lines.map((l) => ({ value: l, label: l })),
          ...(missing !== null ? [{ value: missing, label: t("{0} (non installata)", missing) }] : []),
        ]}
        className="w-48"
      />
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
  const engines: ProjectDb["engine"][] = ["mysql", "mongodb"];
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
            <DbVersionSelect
              project={p.id}
              engine={engine}
              versions={dbs.versions[engine]}
              hasDb={db !== undefined}
              onChanged={onChanged}
            />
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
                  icon={codicon("code")}
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
