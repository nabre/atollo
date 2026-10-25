// The Git section of a project's page: its state, read from .git, and
// whether its VM may use the kit's GitHub access (D-172).
import { useCallback, useState } from "react";
import { ActionButton } from "../shared/common";
import { useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { projectGithubVm, projectGithubVmSet, urlOpen, type ProjectStatus } from "../ipc";
import { CheckboxLabel, codicon, EditorSection, Properties } from "../workbench";

/** The kit's GitHub access in the project's VM: off unless turned on here,
 * since the project's code could then act as the user on GitHub. */
export function GithubInVm({ project }: { project: string }) {
  const load = useCallback(() => projectGithubVm(project), [project]);
  const { data, setData } = useLoad<boolean>(load, false, { onError: "keep" });
  const [error, setError] = useState<string | null>(null);
  return (
    <EditorSection id="github-vm" title={t("GitHub nella VM")}>
      <CheckboxLabel
        checked={data}
        onChange={(on) => {
          setError(null);
          projectGithubVmSet(project, on).then(
            () => {
              setData(on);
            },
            (e: unknown) => {
              setError(String(e));
            },
          );
        }}
      >
        {t("Il progetto nella VM usa il tuo accesso a GitHub del kit (push, gh)")}
      </CheckboxLabel>
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Il token non entra nella VM: lo aggiunge il proxy di Docker Sandboxes alle richieste verso GitHub. Ma il codice del progetto potrebbe agire su GitHub a tuo nome: accendilo solo per i progetti di cui ti fidi.",
        )}
      </p>
      {error && <p className="text-vsc-error">{error}</p>}
    </EditorSection>
  );
}

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
