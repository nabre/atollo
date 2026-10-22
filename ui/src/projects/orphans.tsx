// Data of the kit without a project (D-89): a closed section of
// the Projects side bar, shown only when there is something. Each entry
// can be given to an existing project ("Collega a…", which may have its
// settings but no other data of the kit yet) or deleted with the confirmation of a project's deletion
// (./delete.tsx). Nothing is deleted by itself.
import { useCallback, useState } from "react";
import { projectOrphanAdopt, projectsOrphans, type Holdings, type ProjectInfo } from "../api";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { engineName } from "../databases";
import { codicon, ConfirmDialog, Select, ViewPane } from "../workbench";
import { DeleteProjectDialog } from "./delete";

/** What an orphan holds, as one line. */
function holds(h: Holdings): string {
  const parts: string[] = [];
  if (h.settings) parts.push(t("impostazioni"));
  if (h.ports > 0) parts.push(t("porte: {0}", h.ports));
  if (h.databases.length > 0) parts.push(t("database: {0}", h.databases.map(engineName).join(", ")));
  if (h.backups > 0) parts.push(t("copie: {0}", h.backups));
  return parts.length > 0 ? parts.join(", ") : t("file del kit");
}

export function Orphans({ projects, onChanged }: { projects: ProjectInfo[]; onChanged: () => void }) {
  const [expanded, setExpanded] = useState(false);
  const [adopting, setAdopting] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);
  // Read again when the projects change (a rename outside the app, a refresh).
  const { data: orphans, reload } = useLoad<Holdings[]>(projectsOrphans, [], { onError: "keep", key: projects });
  const done = useCallback(() => {
    setAdopting(null);
    setDeleting(null);
    reload();
    onChanged();
  }, [reload, onChanged]);
  if (orphans.length === 0) return null;
  return (
    <ViewPane
      title={t("Dati senza progetto ({0})", orphans.length)}
      expanded={expanded}
      onToggle={() => {
        setExpanded(!expanded);
      }}
    >
      <p className="px-5 pb-1 text-[12px] text-vsc-desc">
        {t("Il kit ha ancora i dati di progetti che non sono più in projects. Non vengono mai eliminati da soli.")}
      </p>
      <ul className="m-0 list-none space-y-2 p-0 px-5 pb-3">
        {orphans.map((h) => (
          <li key={h.project} className="space-y-1">
            <div className="font-semibold break-all">{h.project}</div>
            <div className="text-[12px] text-vsc-desc">{holds(h)}</div>
            <div className="flex flex-wrap gap-2">
              <ActionButton
                icon={codicon("link")}
                label={t("Collega a…")}
                small
                title={`${t("Collega a…")}: ${h.project}`}
                onClick={() => {
                  setAdopting(h.project);
                }}
              />
              <ActionButton
                icon={codicon("trash")}
                label={t("Elimina")}
                small
                title={`${t("Elimina")}: ${h.project}`}
                onClick={() => {
                  setDeleting(h.project);
                }}
              />
            </div>
          </li>
        ))}
      </ul>
      {adopting !== null && (
        <AdoptDialog
          orphan={adopting}
          projects={projects}
          onCancel={() => {
            setAdopting(null);
          }}
          onDone={done}
        />
      )}
      {deleting !== null && (
        <DeleteProjectDialog
          project={deleting}
          onCancel={() => {
            setDeleting(null);
          }}
          onDeleted={done}
        />
      )}
    </ViewPane>
  );
}

/** Picks the project that receives the data of `orphan`. */
function AdoptDialog({
  orphan,
  projects,
  onCancel,
  onDone,
}: {
  orphan: string;
  projects: ProjectInfo[];
  onCancel: () => void;
  onDone: () => void;
}) {
  const [target, setTarget] = useState(projects[0]?.id ?? "");
  const { busy, error, run } = useAction(onDone);
  return (
    <ConfirmDialog
      title={t("Collegare {0} a un progetto?", orphan)}
      cancelLabel={t("Annulla")}
      onCancel={onCancel}
      actions={
        <ActionButton
          icon={codicon("link")}
          label={t("Collega")}
          primary
          busy={busy === "adopt"}
          disabled={target === "" || busy !== null}
          onClick={() => {
            run("adopt", () => projectOrphanAdopt(orphan, target));
          }}
        />
      }
      footer={<ErrorLine error={error} />}
    >
      <p>
        {t(
          "Impostazioni, porte, database e copie passano al progetto scelto, le cui impostazioni lasciano il posto a queste; non deve avere porte, server, database o copie suoi.",
        )}
      </p>
      {projects.length === 0 ? (
        <p className="text-vsc-desc">{t("Nessun progetto a cui collegare i dati.")}</p>
      ) : (
        <Select
          label={t("Progetto")}
          value={target}
          onChange={setTarget}
          options={projects.map((p) => ({ value: p.id, label: p.name }))}
          className="w-full"
        />
      )}
    </ConfirmDialog>
  );
}
