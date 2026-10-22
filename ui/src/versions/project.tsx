// The versions of the project page: the kit's line of each language, or
// another one, split into two zones (languages active in the project vs
// languages not present in the project).
import { useCallback, useState } from "react";
import {
  projectVersions,
  projectVersionsSet,
  relationsMatrix,
  versionsAdd,
  type ProjectInfo,
  type ProjectVersions as ProjectVersionsData,
  type RelationsMatrix,
  type VersionedTool,
} from "../api";
import { t } from "../i18n";
import { ComponentLogo } from "../shared/logos";
import { useAction, useLoad } from "../shared/hooks";
import { ErrorLine, LoadError } from "../shared/common";
import { useSetup } from "../components/setup";
import { Codicon, EditorSection, Link, Notice } from "../workbench";
import { isPresent, useOffers, useReloadOnSteps } from "./model";
import { LineSelect, LineState, RefreshLines } from "./lines";

/** The versions of the project page: for each installed language, the
 * kit's line, another one on disk or one to download, divided into
 * active project languages and other available languages. */
export function ProjectVersions({ p }: { p: ProjectInfo }) {
  const project = p.id;
  const setup = useSetup();
  const read = useCallback(() => projectVersions(project), [project]);
  // A read that fails says so, with «Riprova»: an empty section would read
  // as nothing to choose.
  const {
    data,
    error: loadError,
    setData,
    reload: load,
    retry,
  } = useLoad<ProjectVersionsData | null>(read, null, { onError: "show" });
  const readMatrix = useCallback(() => relationsMatrix(false), []);
  const { data: matrix } = useLoad<RelationsMatrix | null>(readMatrix, null, { onError: { fallback: null } });
  const { busy, error, run } = useAction();
  const offers = useOffers(data?.tools ?? null, load);
  useReloadOnSteps(setup, load);
  const [activeOpen, setActiveOpen] = useState(true);
  const [inactiveOpen, setInactiveOpen] = useState(false);

  const running = setup?.running ?? false;
  const allTools = data?.tools ?? [];
  // A language the project needs and the kit lacks (p.missing) stays in the
  // first zone, with what to do instead of a line to choose.
  const activeTools = allTools.filter(
    (x) =>
      (p.requires.includes(x.id) || Boolean(data?.chosen[x.id])) &&
      (p.missing.includes(x.id) || x.installed || x.downloadable || x.saved.length > 0),
  );
  const inactiveTools = allTools.filter(
    (x) => !p.requires.includes(x.id) && !data?.chosen[x.id] && x.installed && (x.downloadable || x.saved.length > 0),
  );
  const projectNeeds = matrix?.projects.find((x) => x.project === project)?.needs ?? [];

  const renderTool = (tool: VersionedTool) => {
    const kitLine = tool.preferred ?? tool.default?.line ?? "";
    const chosen = data?.chosen[tool.id] ?? "";
    const need = projectNeeds.find((n) => n.component === tool.id);
    const set = (line: string | null) => {
      run(tool.id, () => projectVersionsSet(project, tool.id, line).then(setData));
    };
    // The line the project's containers name (Compose, Dockerfile): only
    // proposed, applied by the user, when it is not the one in use.
    const fromContainers = p.lines?.[tool.id];
    const suggested = fromContainers && fromContainers !== (chosen || kitLine) ? fromContainers : null;
    const applicable =
      suggested !== null &&
      (isPresent(tool, suggested) || (tool.downloadable && tool.available.includes(suggested))) &&
      busy === null &&
      !(running && !isPresent(tool, suggested));
    if (p.missing.includes(tool.id)) {
      return (
        <div key={tool.id}>
          <div className="flex items-center gap-2">
            <ComponentLogo id={tool.id} size={16} />
            <span className="w-28 shrink-0 truncate">{tool.name}</span>
            <Notice tone="warning" className="min-w-0 flex-1 text-[12px]">
              {t(
                "Non installato: «Installa ciò che manca» in Componenti lo installa, poi qui se ne sceglie la versione.",
              )}
            </Notice>
          </div>
        </div>
      );
    }
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
            first={{
              value: "",
              label: kitLine ? t("La più recente ({0})", kitLine) : t("Predefinita del kit"),
            }}
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
          {need && (
            <p className="mt-0.5 text-[12px] text-vsc-desc">{t("Richiesto da {0}: {1}", need.source, need.declared)}</p>
          )}
          {suggested !== null && (
            <p className="mt-0.5 text-[12px] text-vsc-desc">
              {t("Dai container del progetto (Compose o Dockerfile): {0} {1}.", tool.name, suggested)}{" "}
              {applicable ? (
                <Link
                  onClick={() => {
                    set(suggested);
                  }}
                >
                  {t("Usa {0}", suggested)}
                </Link>
              ) : (
                !isPresent(tool, suggested) &&
                !tool.available.includes(suggested) &&
                t("Non è tra le versioni offerte.")
              )}
            </p>
          )}
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
  };

  return (
    <EditorSection id="versions" title={t("Versioni")}>
      <p className="text-vsc-desc">
        {t(
          "Valgono per terminale, azioni e agenti del progetto, per il server web che lo serve (Caddy o Apache, con il suo PHP) e per i terminali dell'editor aperti da adesso. Una versione da scaricare si scarica appena scelta.",
        )}
      </p>
      <LoadError error={loadError} onRetry={retry} className="mt-2" />
      {data !== null && activeTools.length === 0 && inactiveTools.length === 0 && (
        <p className="mt-2 text-vsc-desc">{t("Nessun linguaggio installato ha più versioni.")}</p>
      )}
      {data !== null && (activeTools.length > 0 || inactiveTools.length > 0) && (
        <div className="mt-3 space-y-3">
          <div className="rounded border border-vsc-border bg-vsc-widget/40 p-2.5">
            <button
              type="button"
              aria-expanded={activeOpen}
              onClick={() => {
                setActiveOpen(!activeOpen);
              }}
              className="flex w-full items-center gap-1.5 text-left text-[12px] font-semibold text-vsc-settings-header outline-none focus-visible:outline-1 focus-visible:outline-vsc-focus"
            >
              <Codicon
                name="chevron-right"
                size={15}
                className={`shrink-0 transition-transform duration-100 ${activeOpen ? "rotate-90" : ""}`}
              />
              <span>{t("Linguaggi attivi nel progetto ({0})", activeTools.length)}</span>
            </button>
            {activeOpen && (
              <div className="mt-2.5 space-y-2 pl-1">
                {activeTools.length === 0 ? (
                  <p className="text-[12px] text-vsc-desc">
                    {t("Nessun linguaggio versionabile rilevato nel progetto.")}
                  </p>
                ) : (
                  activeTools.map(renderTool)
                )}
              </div>
            )}
          </div>

          {inactiveTools.length > 0 && (
            <div className="rounded border border-vsc-border p-2.5">
              <button
                type="button"
                aria-expanded={inactiveOpen}
                onClick={() => {
                  setInactiveOpen(!inactiveOpen);
                }}
                className="flex w-full items-center gap-1.5 text-left text-[12px] font-semibold text-vsc-desc hover:text-vsc-fg outline-none focus-visible:outline-1 focus-visible:outline-vsc-focus"
              >
                <Codicon
                  name="chevron-right"
                  size={15}
                  className={`shrink-0 transition-transform duration-100 ${inactiveOpen ? "rotate-90" : ""}`}
                />
                <span>{t("Linguaggi non presenti nel progetto ({0})", inactiveTools.length)}</span>
              </button>
              {!inactiveOpen ? (
                <p className="mt-1 pl-5 text-[12px] text-vsc-desc">
                  {t("Clicca per espandere se vuoi impostare altri linguaggi per il terminale o gli script.")}
                </p>
              ) : (
                <div className="mt-2.5 space-y-2 pl-1">{inactiveTools.map(renderTool)}</div>
              )}
            </div>
          )}
        </div>
      )}
      <ErrorLine error={error} />
    </EditorSection>
  );
}
