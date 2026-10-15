// Versions of the languages kept side by side (DECISIONS.md D-80): the
// default folder, replaced by updates, and one saved folder per minor line
// (an update to a new minor keeps the old one; others can be downloaded).
// The Tools page chooses the kit's line of each language, the project page
// the project's.
import { useCallback, useEffect, useState } from "react";
import { VscCloudDownload, VscTrash } from "react-icons/vsc";
import {
  projectVersions,
  projectVersionsSet,
  versionsAdd,
  versionsAvailable,
  versionsList,
  versionsPrefer,
  versionsRemove,
  type ProjectVersions as ProjectVersionsData,
  type SetupSnapshot,
  type VersionedTool,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { EditorSection, SettingItem, SettingsGroup } from "./workbench";

/** `22.11` with its full version, as the lists show it. */
function lineText(line: string, version: string | null): string {
  return version && version !== line ? `${line} (${version})` : line;
}

/** The lines a tool can use: the default folder, then the saved ones. */
function choices(tool: VersionedTool): { line: string; label: string }[] {
  const out: { line: string; label: string }[] = [];
  if (tool.default) {
    out.push({
      line: tool.default.line,
      label: t("{0}, predefinita", lineText(tool.default.line, tool.default.version)),
    });
  }
  for (const s of tool.saved) out.push({ line: s.line, label: lineText(s.line, s.version) });
  return out;
}

/** Downloading another line: the list comes from the tool's source. */
function Download({ tool, busy, onAdd }: { tool: VersionedTool; busy: boolean; onAdd: (line: string) => void }) {
  const [lines, setLines] = useState<string[] | null>(null);
  const [line, setLine] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const have = new Set([tool.default?.line, ...tool.saved.map((s) => s.line)]);
  if (lines === null) {
    return (
      <div className="mt-2">
        <ActionButton
          icon={VscCloudDownload}
          label={t("Scarica un'altra versione")}
          small
          busy={loading}
          onClick={() => {
            setLoading(true);
            setError(null);
            versionsAvailable(tool.id).then(
              (l) => {
                setLoading(false);
                setLines(l);
                setLine(l.find((x) => !have.has(x)) ?? "");
              },
              (e: unknown) => {
                setLoading(false);
                setError(String(e));
              },
            );
          }}
        />
        <ErrorLine error={error} />
      </div>
    );
  }
  const free = lines.filter((l) => !have.has(l));
  return (
    <div className="mt-2 flex items-center gap-2">
      <select
        aria-label={t("Versione da scaricare")}
        value={line}
        onChange={(e) => {
          setLine(e.target.value);
        }}
        className="input w-40"
        disabled={free.length === 0}
      >
        {free.length === 0 && <option value="">{t("Nessuna altra versione")}</option>}
        {free.map((l) => (
          <option key={l} value={l}>
            {l}
          </option>
        ))}
      </select>
      <ActionButton
        icon={VscCloudDownload}
        label={t("Scarica")}
        small
        primary
        busy={busy}
        disabled={line === ""}
        onClick={() => {
          onAdd(line);
        }}
      />
    </div>
  );
}

/** The "Versioni" group of the Tools page. */
export function VersionsSection({ setup }: { setup: SetupSnapshot | null }) {
  const [tools, setTools] = useState<VersionedTool[] | null>(null);
  const load = useCallback(() => {
    versionsList().then(setTools, () => {
      setTools([]);
    });
  }, []);
  const { busy, error, run } = useAction(load);
  useEffect(load, [load]);
  // A line downloaded, or an update that kept the old one.
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  useEffect(() => {
    if (finished > 0) load();
  }, [finished, load]);
  const running = setup?.running ?? false;
  const installed = (tools ?? []).filter((x) => x.installed);
  return (
    <SettingsGroup
      id="group-versions"
      title={t("Versioni")}
      description={t(
        "Un aggiornamento che cambia la versione minore conserva quella vecchia. La versione scelta qui vale per tutto il kit; ogni progetto può sceglierne un'altra nella sua pagina.",
      )}
    >
      {installed.map((tool) => {
        const options = choices(tool);
        const value = tool.preferred ?? tool.default?.line ?? "";
        return (
          <SettingItem
            key={tool.id}
            title={tool.name}
            keywords={`${tool.id} ${t("versione")}`}
            modified={tool.preferred !== null && tool.preferred !== tool.default?.line}
          >
            <div className="flex items-center gap-2">
              <ComponentLogo id={tool.id} size={16} />
              <select
                aria-label={t("Versione di {0} per il kit", tool.name)}
                value={value}
                disabled={options.length < 2 || busy !== null}
                onChange={(e) => {
                  const line = e.target.value === tool.default?.line ? null : e.target.value;
                  run(`prefer:${tool.id}`, () => versionsPrefer(tool.id, line).then(setTools));
                }}
                className="input min-w-0 flex-1"
              >
                {options.map((o) => (
                  <option key={o.line} value={o.line}>
                    {o.label}
                  </option>
                ))}
              </select>
            </div>
            {tool.saved.length > 0 && (
              <ul className="mt-2 space-y-0.5" aria-label={t("Versioni conservate di {0}", tool.name)}>
                {tool.saved.map((s) => (
                  <li key={s.line} className="flex items-center gap-2">
                    <span className="min-w-0 flex-1 truncate font-mono text-[12px]">{lineText(s.line, s.version)}</span>
                    <ActionButton
                      icon={VscTrash}
                      label={t("Rimuovi {0} {1}", tool.name, s.line)}
                      compact
                      busy={busy === `remove:${tool.id}:${s.line}`}
                      disabled={running}
                      onClick={() => {
                        run(`remove:${tool.id}:${s.line}`, () => versionsRemove(tool.id, s.line).then(setTools));
                      }}
                    />
                  </li>
                ))}
              </ul>
            )}
            {tool.downloadable && (
              <Download
                tool={tool}
                busy={running || busy === `add:${tool.id}`}
                onAdd={(line) => {
                  run(`add:${tool.id}`, () => versionsAdd(tool.id, line));
                }}
              />
            )}
          </SettingItem>
        );
      })}
      {tools !== null && installed.length === 0 && (
        <p data-setting className="max-w-[880px] px-[14px] text-vsc-desc">
          {t("Nessun linguaggio con più versioni installato.")}
        </p>
      )}
      <div className="max-w-[880px] px-[14px] empty:hidden">
        <ErrorLine error={error} />
      </div>
    </SettingsGroup>
  );
}

/** The versions of the project page: for each installed language, the
 * kit's line or another one kept in the kit. */
export function ProjectVersions({ project }: { project: string }) {
  const [data, setData] = useState<ProjectVersionsData | null>(null);
  const load = useCallback(() => {
    projectVersions(project).then(setData, () => {
      setData(null);
    });
  }, [project]);
  const { busy, error, run } = useAction();
  useEffect(load, [load]);
  const tools = (data?.tools ?? []).filter((x) => x.installed && x.saved.length > 0);
  return (
    <EditorSection id="versions" title={t("Versioni")}>
      <p className="text-vsc-desc">
        {t(
          "Valgono per terminale, azioni e agenti del progetto, e per i terminali dell'editor aperti da adesso. Altre versioni si scaricano in Strumenti.",
        )}
      </p>
      {data !== null && tools.length === 0 && (
        <p className="mt-2 text-vsc-desc">{t("Nessun linguaggio ha altre versioni conservate.")}</p>
      )}
      <div className="mt-2 space-y-1.5">
        {tools.map((tool) => {
          const kitLine = tool.preferred ?? tool.default?.line ?? "";
          const chosen = data?.chosen[tool.id] ?? "";
          const missing = chosen !== "" && !choices(tool).some((c) => c.line === chosen);
          return (
            <div key={tool.id} className="flex items-center gap-2">
              <ComponentLogo id={tool.id} size={16} />
              <label htmlFor={`ver-${tool.id}`} className="w-28 shrink-0 truncate">
                {tool.name}
              </label>
              <select
                id={`ver-${tool.id}`}
                value={chosen}
                disabled={busy !== null}
                onChange={(e) => {
                  const line = e.target.value === "" ? null : e.target.value;
                  run(tool.id, () => projectVersionsSet(project, tool.id, line).then(setData));
                }}
                className="input min-w-0 flex-1"
              >
                <option value="">{t("Come il kit ({0})", kitLine)}</option>
                {choices(tool).map((o) => (
                  <option key={o.line} value={o.line}>
                    {o.label}
                  </option>
                ))}
                {missing && (
                  <option value={chosen}>{t("{0} (non più presente: si usa la predefinita)", chosen)}</option>
                )}
              </select>
            </div>
          );
        })}
      </div>
      <ErrorLine error={error} />
    </EditorSection>
  );
}
