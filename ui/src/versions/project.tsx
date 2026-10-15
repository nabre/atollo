// The versions of the project page: the kit's line of each language, or
// another one.
import { useCallback, useEffect, useState } from "react";
import { projectVersions, projectVersionsSet, versionsAdd, type ProjectVersions as ProjectVersionsData } from "../api";
import { t } from "../i18n";
import { ComponentLogo } from "../logos";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { useSetup } from "../setup";
import { EditorSection } from "../workbench";
import { isPresent, useOffers, useReloadOnSteps } from "./model";
import { LineSelect, LineState, RefreshLines } from "./lines";

/** The versions of the project page: for each installed language, the
 * kit's line, another one on disk or one to download. */
export function ProjectVersions({ project }: { project: string }) {
  const [data, setData] = useState<ProjectVersionsData | null>(null);
  const setup = useSetup();
  const load = useCallback(() => {
    projectVersions(project).then(setData, () => {
      setData(null);
    });
  }, [project]);
  const { busy, error, run } = useAction();
  const offers = useOffers(data?.tools ?? null, load);
  useEffect(load, [load]);
  useReloadOnSteps(setup, load);
  const running = setup?.running ?? false;
  const tools = (data?.tools ?? []).filter((x) => x.installed && (x.downloadable || x.saved.length > 0));
  return (
    <EditorSection id="versions" title={t("Versioni")}>
      <p className="text-vsc-desc">
        {t(
          "Valgono per terminale, azioni e agenti del progetto, e per i terminali dell'editor aperti da adesso. Una versione da scaricare si scarica appena scelta.",
        )}
      </p>
      {data !== null && tools.length === 0 && (
        <p className="mt-2 text-vsc-desc">{t("Nessun linguaggio installato ha più versioni.")}</p>
      )}
      <div className="mt-2 space-y-2">
        {tools.map((tool) => {
          const kitLine = tool.preferred ?? tool.default?.line ?? "";
          const chosen = data?.chosen[tool.id] ?? "";
          const set = (line: string | null) => {
            run(tool.id, () => projectVersionsSet(project, tool.id, line).then(setData));
          };
          return (
            <div key={tool.id}>
              <div className="flex items-center gap-2">
                <ComponentLogo id={tool.id} size={16} />
                <label htmlFor={`ver-${tool.id}`} className="w-28 shrink-0 truncate">
                  {tool.name}
                </label>
                <LineSelect
                  id={`ver-${tool.id}`}
                  tool={tool}
                  value={chosen}
                  first={{ value: "", label: t("La più recente ({0})", kitLine) }}
                  disabled={busy !== null || (running && chosen !== "" && !isPresent(tool, chosen))}
                  onChange={(l) => {
                    set(l === "" ? null : l);
                  }}
                />
                <RefreshLines
                  tool={tool}
                  checking={offers.checking.has(tool.id)}
                  disabled={busy !== null}
                  onRefresh={() => {
                    offers.refresh([tool.id]);
                  }}
                />
              </div>
              <div className="pl-[136px]">
                <LineState
                  tool={tool}
                  line={chosen === "" ? null : chosen}
                  fallback={tool.default?.line ?? ""}
                  setup={setup}
                  error={offers.failed[tool.id]}
                  onDownload={(line) => {
                    run(tool.id, () => versionsAdd(tool.id, line));
                  }}
                />
              </div>
            </div>
          );
        })}
      </div>
      <ErrorLine error={error} />
    </EditorSection>
  );
}
