// The Sito web section of a project's page: a folder of it served on its
// own port.
import { useState, type SyntheticEvent } from "react";
import { VscGlobe } from "react-icons/vsc";
import { projectSite, type ProjectInfo, type SiteKind } from "../api";
import { useAction } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { t } from "../i18n";
import { Select } from "../workbench";

/** "Servi come sito web": a folder of the project served on its own port. */
export function SiteForm({ project, onChanged }: { project: ProjectInfo; onChanged: () => void }) {
  const site = project.actions.find((a) => a.id === "site");
  const [folder, setFolder] = useState(site?.static ?? site?.apache ?? ".");
  const [kind, setKind] = useState<SiteKind>(
    site ? (site.static !== null ? "static" : site.apache !== null ? "apache" : "php") : "static",
  );
  const { busy, error, run } = useAction(onChanged);
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("site", () => projectSite(project.id, folder, kind));
  };
  return (
    <form onSubmit={submit} className="flex flex-wrap items-end gap-2" aria-label={t("Servi come sito web")}>
      <label className="flex flex-col gap-1">
        <span className="text-vsc-desc">{t("Servi come sito web la cartella")}</span>
        <input
          value={folder}
          onChange={(e) => {
            setFolder(e.target.value);
          }}
          className="w-64 input font-mono"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-vsc-desc">{t("con")}</span>
        <Select<SiteKind>
          value={kind}
          onChange={setKind}
          options={[
            { value: "static", label: t("file statici") },
            { value: "php", label: t("PHP (server integrato)") },
            { value: "apache", label: t("Apache dedicato") },
          ]}
        />
      </label>
      <ActionButton icon={VscGlobe} label={t("Salva")} busy={busy === "site"} onClick={() => undefined} submit />
      {site && (
        <ActionButton
          icon={VscGlobe}
          label={t("Togli il sito")}
          busy={busy === "none"}
          onClick={() => {
            run("none", () => projectSite(project.id, ".", "none"));
          }}
        />
      )}
      {kind === "apache" && (
        <p className="w-full text-[12px] text-vsc-desc">
          {t(
            "Un Apache solo per questo progetto, sulla sua porta, con .htaccess e il PHP del kit. La configurazione è in data\\app\\projects\\{0}\\apache; le tue direttive vanno in conf.d.",
            project.id,
          )}
        </p>
      )}
      <div className="w-full empty:hidden">
        <ErrorLine error={error} />
      </div>
    </form>
  );
}
