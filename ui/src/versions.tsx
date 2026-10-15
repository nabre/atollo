// Versions of the languages kept side by side (DECISIONS.md D-80): the
// default folder, replaced by updates, and one saved folder per minor line
// (an update to a new minor keeps the old one; others can be downloaded).
// The Tools page chooses the kit's line of each language, the project page
// the project's. The lists propose also the lines the source offers and
// are not on disk yet: choosing one downloads it.
import { useCallback, useEffect, useRef, useState } from "react";
import { VscCloudDownload, VscRefresh, VscTrash } from "react-icons/vsc";
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
import { useSetup } from "./setup";
import { Badge, Tag } from "./tags";
import { EditorSection, Link, Notice, SettingItem, SettingsGroup } from "./workbench";

/** A list older than this (seconds) is asked again. */
const STALE = 24 * 60 * 60;

/** `22.11` with its full version, as the lists show it. */
function lineText(line: string, version: string | null): string {
  return version && version !== line ? `${line} (${version})` : line;
}

/** The lines on disk: the default folder, then the saved ones. */
function present(tool: VersionedTool): { line: string; label: string }[] {
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

function isPresent(tool: VersionedTool, line: string): boolean {
  return tool.default?.line === line || tool.saved.some((s) => s.line === line);
}

/** The offered lines not on disk, by major version, newest first. */
function offered(tool: VersionedTool): { major: string; lines: string[] }[] {
  if (!tool.downloadable) return [];
  const groups: { major: string; lines: string[] }[] = [];
  for (const line of tool.available) {
    if (isPresent(tool, line)) continue;
    const major = line.split(".")[0] ?? line;
    const last = groups.at(-1);
    if (last?.major === major) last.lines.push(line);
    else groups.push({ major, lines: [line] });
  }
  return groups;
}

/** The download of `line` of `tool` in the preparation task. */
function stepOf(setup: SetupSnapshot | null, tool: VersionedTool, line: string) {
  return [...(setup?.steps ?? [])].reverse().find((s) => s.id === tool.id && s.name.endsWith(` ${line}`)) ?? null;
}

/** Asks the sources of the installed tools whose list is missing or
 * older than a day, once per view; `reload` after. */
function useOffers(tools: VersionedTool[] | null, reload: () => void) {
  const [checking, setChecking] = useState<ReadonlySet<string>>(new Set());
  const [failed, setFailed] = useState<Record<string, string>>({});
  const asked = useRef(new Set<string>());
  const refresh = useCallback(
    (ids: string[]) => {
      for (const id of ids) asked.current.add(id);
      setChecking((c) => new Set([...c, ...ids]));
      void Promise.allSettled(
        ids.map((id) =>
          versionsAvailable(id).then(
            () => {
              setFailed((f) => Object.fromEntries(Object.entries(f).filter(([k]) => k !== id)));
            },
            (e: unknown) => {
              setFailed((f) => ({ ...f, [id]: String(e) }));
            },
          ),
        ),
      ).then(() => {
        setChecking((c) => new Set([...c].filter((id) => !ids.includes(id))));
        reload();
      });
    },
    [reload],
  );
  useEffect(() => {
    if (tools === null) return;
    const now = Date.now() / 1000;
    const stale = tools
      .filter((x) => x.installed && x.downloadable && !asked.current.has(x.id))
      .filter((x) => x.checked === null || now - x.checked > STALE)
      .map((x) => x.id);
    if (stale.length > 0) refresh(stale);
  }, [tools, refresh]);
  return { checking, failed, refresh };
}

/** The list of a tool's lines: those on disk, then those to download by
 * major version. `first` is an option before them (the project's "as the
 * kit"); a `value` in neither list is shown as it is. */
function LineSelect({
  id,
  tool,
  value,
  first,
  label,
  disabled,
  onChange,
}: {
  id?: string;
  tool: VersionedTool;
  value: string;
  first?: { value: string; label: string };
  label?: string;
  disabled: boolean;
  onChange: (line: string) => void;
}) {
  const disk = present(tool);
  const groups = offered(tool);
  const known = value === first?.value || disk.some((o) => o.line === value) || tool.available.includes(value);
  return (
    <select
      id={id}
      aria-label={label}
      value={value}
      disabled={disabled}
      onChange={(e) => {
        onChange(e.target.value);
      }}
      className="input min-w-0 flex-1"
    >
      {first && <option value={first.value}>{first.label}</option>}
      <optgroup label={t("Scaricate")}>
        {disk.map((o) => (
          <option key={o.line} value={o.line}>
            {o.label}
          </option>
        ))}
      </optgroup>
      {groups.map((g) => (
        <optgroup key={g.major} label={t("Da scaricare: {0}.x", g.major)}>
          {g.lines.map((l) => (
            <option key={l} value={l}>
              {t("{0} (da scaricare)", l)}
            </option>
          ))}
        </optgroup>
      ))}
      {!known && <option value={value}>{t("{0} (non scaricata)", value)}</option>}
    </select>
  );
}

/** The button that asks the source for the lines it offers now. */
function RefreshLines({
  tool,
  checking,
  disabled,
  onRefresh,
}: {
  tool: VersionedTool;
  checking: boolean;
  disabled: boolean;
  onRefresh: () => void;
}) {
  if (!tool.downloadable) return null;
  const when =
    tool.checked === null
      ? t("Elenco delle versioni mai scaricato.")
      : t("Elenco delle versioni del {0}.", new Date(tool.checked * 1000).toLocaleString());
  return (
    <ActionButton
      icon={VscRefresh}
      label={t("Cerca altre versioni di {0}", tool.name)}
      title={`${t("Cerca altre versioni di {0}", tool.name)}. ${when}`}
      compact
      busy={checking}
      disabled={disabled}
      onClick={onRefresh}
    />
  );
}

/** What happens to a chosen line not on disk: its download, its failure, or
 * a link to download it; until then the default folder (`fallback`) is used,
 * as versions.rs does. */
function LineState({
  tool,
  line,
  fallback,
  setup,
  error,
  onDownload,
}: {
  tool: VersionedTool;
  line: string | null;
  fallback: string;
  setup: SetupSnapshot | null;
  error: string | undefined;
  onDownload: (line: string) => void;
}) {
  if (error) {
    return (
      <Notice tone="warning" className="mt-1 text-[12px]">
        {t("Impossibile leggere le versioni disponibili: {0}", error)}
      </Notice>
    );
  }
  if (line === null || isPresent(tool, line)) return null;
  const step = stepOf(setup, tool, line);
  if (step && step.outcome === null && setup?.running) {
    const percent = step.size && step.done !== null ? ` ${String(Math.floor((step.done * 100) / step.size))}%` : "";
    return (
      <Notice tone="info" className="mt-1 text-[12px]">
        {t("Download di {0} {1} in corso…", tool.name, line)}
        {percent}
      </Notice>
    );
  }
  const failure = step?.outcome?.kind === "failed" ? step.outcome.message : null;
  return (
    <Notice tone={failure ? "error" : "warning"} className="mt-1 text-[12px]">
      {failure
        ? t("Download di {0} {1} non riuscito: {2}", tool.name, line, failure)
        : t("{0} {1} non è scaricata: per ora si usa la {2}.", tool.name, line, fallback)}{" "}
      {tool.downloadable && !setup?.running && (
        <Link
          onClick={() => {
            onDownload(line);
          }}
        >
          {failure ? t("Riprova") : t("Scarica")}
        </Link>
      )}
    </Notice>
  );
}

/** The lines on disk of a tool, with who uses them; saved ones can go. */
function DiskLines({
  tool,
  busy,
  running,
  onRemove,
}: {
  tool: VersionedTool;
  busy: string | null;
  running: boolean;
  onRemove: (line: string) => void;
}) {
  const kitLine = tool.preferred ?? tool.default?.line;
  const rows = [
    ...(tool.default ? [{ ...tool.default, isDefault: true }] : []),
    ...tool.saved.map((s) => ({ ...s, isDefault: false })),
  ];
  if (rows.length < 2 && Object.keys(tool.projects).length === 0) return null;
  return (
    <ul className="mt-2 space-y-0.5" aria-label={t("Versioni scaricate di {0}", tool.name)}>
      {rows.map((r) => {
        const users = tool.projects[r.line] ?? [];
        return (
          <li key={r.line} className="flex min-h-[22px] items-center gap-2">
            <span className="min-w-0 truncate font-mono text-[12px]">{lineText(r.line, r.version)}</span>
            {r.isDefault && <Tag>{t("predefinita")}</Tag>}
            {r.line === kitLine && <Badge>{t("kit")}</Badge>}
            {users.length > 0 && (
              <Tag title={users.join(", ")}>{users.length === 1 ? users[0] : t("{0} progetti", users.length)}</Tag>
            )}
            <span className="flex-1" />
            {!r.isDefault && (
              <ActionButton
                icon={VscTrash}
                label={t("Rimuovi {0} {1}", tool.name, r.line)}
                title={
                  users.length > 0
                    ? t("La usano {0}: dopo la rimozione useranno la predefinita.", users.join(", "))
                    : t("Rimuovi {0} {1}", tool.name, r.line)
                }
                compact
                busy={busy === `remove:${tool.id}:${r.line}`}
                disabled={running || busy !== null}
                onClick={() => {
                  onRemove(r.line);
                }}
              />
            )}
          </li>
        );
      })}
    </ul>
  );
}

/** Downloading a line without choosing it (for the projects). */
function Download({ tool, busy, onAdd }: { tool: VersionedTool; busy: boolean; onAdd: (line: string) => void }) {
  const free = offered(tool).flatMap((g) => g.lines);
  const [line, setLine] = useState("");
  const value = free.includes(line) ? line : (free[0] ?? "");
  if (free.length === 0) return null;
  return (
    <div className="mt-2 flex items-center gap-2">
      <select
        aria-label={t("Versione da scaricare")}
        value={value}
        onChange={(e) => {
          setLine(e.target.value);
        }}
        className="input w-40"
      >
        {offered(tool).map((g) => (
          <optgroup key={g.major} label={`${g.major}.x`}>
            {g.lines.map((l) => (
              <option key={l} value={l}>
                {l}
              </option>
            ))}
          </optgroup>
        ))}
      </select>
      <ActionButton
        icon={VscCloudDownload}
        label={t("Scarica")}
        small
        busy={busy}
        disabled={value === ""}
        onClick={() => {
          onAdd(value);
        }}
      />
    </div>
  );
}

/** Reloads after every step the preparation task finishes. */
function useReloadOnSteps(setup: SetupSnapshot | null, load: () => void) {
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  useEffect(() => {
    if (finished > 0) load();
  }, [finished, load]);
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
  const offers = useOffers(tools, load);
  useEffect(load, [load]);
  useReloadOnSteps(setup, load);
  const running = setup?.running ?? false;
  const installed = (tools ?? []).filter((x) => x.installed);
  return (
    <SettingsGroup
      id="group-versions"
      title={t("Versioni")}
      description={t(
        "La versione scelta qui vale per tutto il kit; ogni progetto può sceglierne un'altra nella sua pagina. Le versioni da scaricare si scaricano appena scelte; un aggiornamento che cambia la versione minore conserva quella vecchia.",
      )}
    >
      {installed.map((tool) => {
        const value = tool.preferred ?? tool.default?.line ?? "";
        const add = (line: string) => {
          run(`add:${tool.id}`, () => versionsAdd(tool.id, line));
        };
        return (
          <SettingItem
            key={tool.id}
            title={tool.name}
            keywords={`${tool.id} ${t("versione")}`}
            modified={tool.preferred !== null && tool.preferred !== tool.default?.line}
          >
            <div className="flex items-center gap-2">
              <ComponentLogo id={tool.id} size={16} />
              <LineSelect
                tool={tool}
                value={value}
                label={t("Versione di {0} per il kit", tool.name)}
                disabled={busy !== null || (running && !isPresent(tool, value))}
                onChange={(l) => {
                  const line = l === tool.default?.line ? null : l;
                  run(`prefer:${tool.id}`, () => versionsPrefer(tool.id, line).then(setTools));
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
            <LineState
              tool={tool}
              line={tool.preferred}
              fallback={tool.default?.line ?? ""}
              setup={setup}
              error={offers.failed[tool.id]}
              onDownload={add}
            />
            <DiskLines
              tool={tool}
              busy={busy}
              running={running}
              onRemove={(line) => {
                run(`remove:${tool.id}:${line}`, () => versionsRemove(tool.id, line).then(setTools));
              }}
            />
            {tool.downloadable && <Download tool={tool} busy={running || busy === `add:${tool.id}`} onAdd={add} />}
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
                  first={{ value: "", label: t("Come il kit ({0})", kitLine) }}
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
