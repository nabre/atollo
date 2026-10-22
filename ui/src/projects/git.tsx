// The Git section of a project's page: its state, read from .git.
import { ActionButton } from "../shared/common";
import { t } from "../i18n";
import { urlOpen, type ProjectStatus } from "../api";
import { codicon, EditorSection, Properties } from "../workbench";

export function Git({ status }: { status: ProjectStatus | null }) {
  const git = status?.git;
  if (!status) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  if (!git) return <p className="text-vsc-desc">{t("La cartella non è un repository Git.")}</p>;
  const web = git.remote?.replace(/\.git$/, "").replace(/^git@github\.com:/, "https://github.com/") ?? null;
  return (
    <EditorSection
      id="repo"
      title={t("Repository")}
      actions={
        web?.startsWith("https://") && (
          <ActionButton
            icon={codicon("github")}
            label={t("Apri sul web")}
            onClick={() => {
              void urlOpen(web);
            }}
          />
        )
      }
    >
      <Properties
        rows={[
          [t("Remoto"), <span className="truncate font-mono select-text">{git.remote ?? t("nessuno")}</span>],
          [t("Ramo"), <span className="font-mono">{git.branch ?? t("nessun ramo")}</span>],
          [
            t("Remoto del ramo"),
            <span className="font-mono">{git.upstream ?? t("ramo non collegato al remoto")}</span>,
          ],
          [
            t("Sincronizzato"),
            git.synced === null
              ? t("non si sa")
              : git.synced
                ? t("sì, allo stesso commit (all'ultimo aggiornamento dal remoto)")
                : t("no: ci sono commit da inviare o da ricevere"),
          ],
        ]}
      />
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Commit, push e l'elenco delle modifiche sono nell'editor o in un agente. Atollo legge lo stato dai file di .git, senza eseguire Git nel progetto.",
        )}
      </p>
    </EditorSection>
  );
}
