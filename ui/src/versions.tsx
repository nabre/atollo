// Versions of the languages kept side by side (DECISIONS.md D-80): the
// default folder, replaced by updates, and one saved folder per minor line
// (an update to a new minor keeps the old one; others can be downloaded).
// The Tools page chooses the kit's line of each language in its row, the
// project page the project's. The lists propose also the lines the source offers and
// are not on disk yet: choosing one downloads it.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { VscAdd, VscCloudDownload, VscLock, VscRefresh, VscTrash } from "react-icons/vsc";
import {
  projectVersions,
  projectVersionsSet,
  versionsAdd,
  versionsAvailable,
  versionsList,
  versionsRemove,
  type ProjectVersions as ProjectVersionsData,
  type SetupSnapshot,
  type VersionedTool,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { ItemBar, ItemStatus, itemState, useSetup } from "./setup";
import { ByHandBox } from "./byhand";
import { Tag } from "./tags";
import { EditorSection, Link, Notice } from "./workbench";

/** A list older than this (seconds) is asked again. */
const STALE = 24 * 60 * 60;

/** A full version and the parts that come with it, as the installers
 * write it: `8.4.13 (Composer 2.8.12)` → `8.4.13`, `["Composer 2.8.12"]`. */
export function splitVersion(version: string | null): { main: string | null; parts: string[] } {
  const m = version?.match(/^(.*?) \((.*)\)$/);
  return m ? { main: m[1] ?? null, parts: (m[2] ?? "").split(", ") } : { main: version, parts: [] };
}

/** The Composer that came with a version of PHP: `2.8.12`. */
export function composerOf(version: string | null): string | null {
  const part = splitVersion(version).parts.find((p) => p.startsWith("Composer "));
  return part ? part.slice("Composer ".length) : null;
}

/** `22.11` with its full version, as the lists show it. */
function lineText(line: string, version: string | null): string {
  const main = splitVersion(version).main;
  return main && main !== line ? `${line} (${main})` : line;
}

/** The lines on disk: the default folder, then the saved ones. */
function present(tool: VersionedTool): { line: string; label: string }[] {
  const out: { line: string; label: string }[] = [];
  if (tool.default) {
    out.push({
      line: tool.default.line,
      label: lineText(tool.default.line, tool.default.version),
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
  inList = false,
}: {
  tool: VersionedTool;
  line: string | null;
  fallback: string;
  setup: SetupSnapshot | null;
  error: string | undefined;
  onDownload: (line: string) => void;
  /** The list of the lines shows its download (Tools page). */
  inList?: boolean;
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
  const waiting = setup?.running && setup.queue.some((q) => q.id === tool.id && q.name === `${tool.name} ${line}`);
  if (inList && (waiting || (step && step.outcome === null && setup?.running))) return null;
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

/** The lines on disk of a tool, with who uses them. Each can go while
 * another stays (the newest left becomes the default); the last goes with
 * the tool, by its checkbox. */
function DiskLines({
  tool,
  setup,
  busy,
  running,
  outside,
  onRemove,
}: {
  tool: VersionedTool;
  setup: SetupSnapshot | null;
  outside: Record<string, string[]>;
  busy: string | null;
  running: boolean;
  onRemove: (line: string) => void;
}) {
  const rows = [...(tool.default ? [tool.default] : []), ...tool.saved];
  // Lines the preparation is downloading or will download, not on disk yet.
  const prefix = `${tool.name} `;
  const coming = [...(setup?.running ? setup.queue : []), ...(setup?.steps ?? [])]
    .filter((s) => s.id === tool.id && s.name.startsWith(prefix))
    .map((s) => s.name.slice(prefix.length))
    .filter((line, i, all) => all.indexOf(line) === i && !isPresent(tool, line))
    .filter((line) => {
      const state = itemState(setup, tool.id, prefix + line);
      // A failed download stays, to be done in the browser.
      return state !== null && (state.kind !== "problem" || Boolean(state.byHand));
    });
  if (rows.length === 0 && coming.length === 0) return null;
  return (
    <ul className="mt-1 space-y-0.5" aria-label={t("Versioni di {0}", tool.name)}>
      {rows.map((r) => {
        const users = tool.projects[r.line] ?? [];
        const state = itemState(setup, tool.id, prefix + r.line);
        return (
          <li key={r.line}>
            <div className="flex min-h-[22px] items-center gap-2">
              <span className="min-w-0 truncate font-mono text-[12px]">{lineText(r.line, r.version)}</span>
              {/* What comes with this version (Maven, Gradle with Java); Composer has its row. */}
              {splitVersion(r.version)
                .parts.filter((p) => !p.startsWith("Composer "))
                .map((p) => (
                  <Tag key={p}>{p}</Tag>
                ))}
              {/* The newest line its source offers. */}
              {tool.available[0] === r.line && <Tag tone="focus">latest</Tag>}
              {(outside[r.line] ?? []).length > 0 && (
                <Tag tone="warning">{t("non per {0}", (outside[r.line] ?? []).join(", "))}</Tag>
              )}
              <span className="flex-1" />
              {state && (
                <span className="text-[12px]">
                  <ItemStatus state={state} />
                </span>
              )}
              {/* Locked by the projects that use it; otherwise it can go. */}
              {users.length > 0 ? (
                <Locked projects={users} label={t("{0} {1}: progetti che la usano", tool.name, r.line)} />
              ) : (
                rows.length > 1 && (
                  <ActionButton
                    icon={VscTrash}
                    label={t("Rimuovi {0} {1}", tool.name, r.line)}
                    compact
                    busy={busy === `remove:${tool.id}:${r.line}`}
                    disabled={running || busy !== null}
                    onClick={() => {
                      onRemove(r.line);
                    }}
                  />
                )
              )}
            </div>
            {state && <ItemBar state={state} label={prefix + r.line} />}
            {state?.kind === "problem" && <p className="text-[12px] text-vsc-error select-text">{state.text}</p>}
            {state?.kind === "problem" && state.byHand && (
              <ByHandBox
                id={tool.id}
                name={prefix + r.line}
                byHand={state.byHand}
                disabled={running}
                onRetry={() => versionsAdd(tool.id, r.line)}
              />
            )}
          </li>
        );
      })}
      {coming.map((line) => {
        const state = itemState(setup, tool.id, prefix + line);
        return (
          <li key={`coming-${line}`}>
            <div className="flex min-h-[22px] items-center gap-2">
              <span className="min-w-0 truncate font-mono text-[12px] text-vsc-desc">{line}</span>
              <span className="flex-1" />
              {state && (
                <span className="text-[12px]">
                  <ItemStatus state={state} />
                </span>
              )}
            </div>
            {state && <ItemBar state={state} label={prefix + line} />}
            {state?.kind === "problem" && state.byHand && (
              <>
                <p className="text-[12px] text-vsc-error select-text">{state.text}</p>
                <ByHandBox
                  id={tool.id}
                  name={prefix + line}
                  byHand={state.byHand}
                  disabled={running}
                  onRetry={() => versionsAdd(tool.id, line)}
                />
              </>
            )}
          </li>
        );
      })}
    </ul>
  );
}

/** "locked": a version some projects use, so it cannot go; its small menu
 * lists them. The counterpart of the bin. */
function Locked({ projects, label }: { projects: string[]; label: string }) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !box.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  return (
    <div ref={box} className="relative">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={label}
        onClick={() => {
          setOpen((o) => !o);
        }}
        className="inline-flex items-center gap-1 rounded-sm px-1 text-[11px] text-vsc-desc hover:bg-vsc-list-hover hover:text-vsc-fg"
      >
        <VscLock size={12} aria-hidden="true" />
        locked
      </button>
      {open && (
        <ul
          role="menu"
          aria-label={label}
          className="absolute top-full right-0 z-20 mt-1 min-w-36 rounded border border-vsc-menu-border bg-vsc-menu py-1 text-[12px] text-vsc-menu-fg shadow-[0_2px_8px_var(--vsc-shadow)]"
        >
          <li role="none" className="px-3 pb-0.5 text-[11px] text-vsc-desc">
            {t("Usata da")}
          </li>
          {projects.map((p) => (
            <li key={p} role="menuitem" className="px-3 py-0.5">
              {p}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** "+": the lines its source offers that are not on disk (nor being
 * downloaded), by major version; one chosen is downloaded without becoming
 * the kit's (for the projects). */
function AddLine({
  tool,
  setup,
  busy,
  onAdd,
}: {
  tool: VersionedTool;
  setup: SetupSnapshot | null;
  busy: boolean;
  onAdd: (line: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const groups = offered(tool)
    .map((g) => ({
      ...g,
      lines: g.lines.filter((l) => {
        const state = itemState(setup, tool.id, `${tool.name} ${l}`);
        return state === null || state.kind === "problem";
      }),
    }))
    .filter((g) => g.lines.length > 0);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !box.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  if (groups.length === 0) return null;
  return (
    <div ref={box} className="relative">
      <ActionButton
        icon={VscAdd}
        label={t("Aggiungi una versione di {0}", tool.name)}
        compact
        busy={busy}
        onClick={() => {
          setOpen((o) => !o);
        }}
      />
      {open && (
        <ul
          role="menu"
          aria-label={t("Versioni da scaricare")}
          className="absolute top-full right-0 z-20 mt-1 max-h-72 min-w-44 overflow-y-auto rounded border border-vsc-menu-border bg-vsc-menu py-1 text-vsc-menu-fg shadow-[0_2px_8px_var(--vsc-shadow)]"
        >
          {groups.map((g) => (
            <li key={g.major} role="none">
              <div className="px-3 pt-1 pb-0.5 text-[11px] text-vsc-desc">{`${g.major}.x`}</div>
              <ul role="none">
                {g.lines.map((l) => (
                  <li key={l} role="none">
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setOpen(false);
                        onAdd(l);
                      }}
                      className="flex w-full items-center gap-2 px-3 py-0.5 text-left font-mono text-[12px] hover:bg-vsc-list-hover focus:bg-vsc-list-hover focus:outline-none"
                    >
                      <VscCloudDownload size={12} aria-hidden="true" />
                      {tool.name} {l}
                      {tool.available[0] === l && (
                        <Tag tone="focus" className="ml-auto">
                          latest
                        </Tag>
                      )}
                    </button>
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
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

/** The kit's versions for the Tools page, where each versioned tool shows
 * them in its own row (ToolVersions). `enabled` false (first start) asks
 * nothing. */
export function useKitVersions(setup: SetupSnapshot | null, enabled: boolean) {
  const [tools, setTools] = useState<VersionedTool[] | null>(null);
  const load = useCallback(() => {
    if (!enabled) return;
    versionsList().then(setTools, () => {
      setTools([]);
    });
  }, [enabled]);
  const { busy, error, run: runAction } = useAction(load);
  // The tool of the last action, the one that shows its error.
  const [last, setLast] = useState<string | null>(null);
  const offers = useOffers(tools, load);
  useEffect(load, [load]);
  useReloadOnSteps(setup, load);
  const run = useCallback(
    (id: string, key: string, task: () => Promise<void>) => {
      setLast(id);
      runAction(key, task);
    },
    [runAction],
  );
  const byId = useMemo(() => new Map((tools ?? []).map((x) => [x.id, x])), [tools]);
  return { byId, setTools, busy, error, last, run, offers };
}

export type KitVersions = ReturnType<typeof useKitVersions>;

/** The versions of an installed tool in its row of the Tools page: the
 * kit's line, the lines on disk with who uses them, one to download. */
export function ToolVersions({
  tool,
  kit,
  setup,
  outside = {},
}: {
  tool: VersionedTool;
  kit: KitVersions;
  setup: SetupSnapshot | null;
  /** Line → the components that cannot run on it (relations matrix). */
  outside?: Record<string, string[]>;
}) {
  const { busy, run, offers, setTools } = kit;
  const running = setup?.running ?? false;
  const add = (line: string) => {
    run(tool.id, `add:${tool.id}`, () => versionsAdd(tool.id, line));
  };
  return (
    <div className="mt-1.5 max-w-[620px]">
      <div className="flex items-center gap-1">
        <span className="flex-1 text-[12px] text-vsc-desc">
          {t("Versioni: il kit usa la più recente; un progetto nuovo tiene quella che c'è alla sua creazione")}
        </span>
        <div className="flex items-center gap-1">
          <RefreshLines
            tool={tool}
            checking={offers.checking.has(tool.id)}
            disabled={busy !== null}
            onRefresh={() => {
              offers.refresh([tool.id]);
            }}
          />
          {tool.downloadable && (
            <AddLine tool={tool} setup={setup} busy={running || busy === `add:${tool.id}`} onAdd={add} />
          )}
        </div>
      </div>
      <LineState
        tool={tool}
        line={tool.preferred}
        fallback={tool.default?.line ?? ""}
        setup={setup}
        error={offers.failed[tool.id]}
        onDownload={add}
        inList
      />
      <DiskLines
        tool={tool}
        setup={setup}
        outside={outside}
        busy={busy}
        running={running}
        onRemove={(line) => {
          run(tool.id, `remove:${tool.id}:${line}`, () => versionsRemove(tool.id, line).then(setTools));
        }}
      />
      {kit.last === tool.id && <ErrorLine error={kit.error} />}
    </div>
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
