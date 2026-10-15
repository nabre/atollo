// Deleting a project (PLAN-DATA.md WP-5, user choice Q-2): the confirmation
// lists what goes with it, from project_delete_plan: its empty folder, its
// databases and users on the kit's servers (dropped first, a stopped server
// is started for it), the copies of them. A failed drop deletes nothing;
// then "Elimina senza i database" deletes the folders and leaves the
// databases on the servers (named in the log).
import { useCallback, useState } from "react";
import { VscFolderOpened, VscTrash } from "react-icons/vsc";
import { folderOpen, projectDelete, projectDeletePlan, type DeletePlan, type PlannedDb } from "../api";
import { ActionButton, ErrorLine } from "../common";
import { useAction, useLoad } from "../hooks";
import { locale, t } from "../i18n";
import { ConfirmDialog } from "../workbench";

const ENGINE_NAMES: Record<PlannedDb["engine"], string> = { mariadb: "MariaDB", mongodb: "MongoDB" };

function size(bytes: number): string {
  const kb = bytes / 1024;
  return kb < 1024
    ? `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 0 }).format(Math.max(1, kb))} KB`
    : `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(kb / 1024)} MB`;
}

export function DeleteProjectDialog({
  project,
  onCancel,
  onDeleted,
}: {
  project: string;
  onCancel: () => void;
  onDeleted: () => void;
}) {
  const read = useCallback(() => projectDeletePlan(project), [project]);
  const { data: plan, error: planError } = useLoad<DeletePlan | null>(read, null, { onError: "show" });
  const { busy, error, run } = useAction(onDeleted);
  // Offered once something went wrong: a drop, or reading the plan.
  const [failed, setFailed] = useState(false);
  const remove = (force: boolean) => {
    run(force ? "force" : "delete", async () => {
      try {
        await projectDelete(project, force);
      } catch (e) {
        setFailed(true);
        throw e;
      }
    });
  };
  const offerForce = failed || planError !== null;
  return (
    <ConfirmDialog
      title={t("Eliminare {0}?", project)}
      cancelLabel={t("Annulla")}
      onCancel={onCancel}
      actions={
        <>
          {offerForce && (
            <ActionButton
              icon={VscTrash}
              label={t("Elimina senza i database")}
              busy={busy === "force"}
              disabled={busy !== null}
              onClick={() => {
                remove(true);
              }}
            />
          )}
          <ActionButton
            icon={VscTrash}
            label={t("Elimina")}
            primary
            busy={busy === "delete"}
            disabled={plan === null || busy !== null}
            onClick={() => {
              remove(false);
            }}
          />
        </>
      }
      footer={
        <>
          <ErrorLine error={error ?? planError} />
          {offerForce && (
            <p className="text-[12px] text-vsc-desc">
              {t("«Elimina senza i database» lascia database e utenti sui server.")}
            </p>
          )}
        </>
      }
    >
      <p>{t("Vengono eliminati:")}</p>
      <ul className="list-disc space-y-1 pl-5">
        <li>
          {plan?.folderExists === false
            ? t("Le impostazioni e i dati del kit di questo progetto")
            : t("La cartella vuota del progetto e le impostazioni del kit")}
        </li>
        {plan?.databases.map((db) =>
          db.installed ? (
            <li key={db.engine}>{t("Il database {0} e il suo utente su {1}", db.database, ENGINE_NAMES[db.engine])}</li>
          ) : (
            <li key={db.engine} className="text-vsc-desc">
              {t("{0} non è installato: il database {1} resta nei suoi dati", ENGINE_NAMES[db.engine], db.database)}
            </li>
          ),
        )}
        {plan !== null && plan.backups > 0 && (
          <li>
            <span className="flex flex-wrap items-center gap-2">
              {t("Copie dei database: {0} ({1})", plan.backups, size(plan.backupsSize))}
              <ActionButton
                icon={VscFolderOpened}
                label={t("Apri la cartella delle copie")}
                small
                onClick={() => {
                  void folderOpen("backup", project);
                }}
              />
            </span>
          </li>
        )}
      </ul>
      {plan?.databases.some((db) => db.installed) && (
        <p className="text-[12px] text-vsc-desc">{t("Un server fermo viene avviato per eliminare il database.")}</p>
      )}
    </ConfirmDialog>
  );
}
