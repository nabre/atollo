// Projects page: what each project in projects\ is, how it starts (detected
// by src-tauri/src/detect, corrected by panel.json) and its running actions
// with their output. Web actions get their own port and open in the browser.
import { useCallback, useEffect, useRef, useState, type ReactNode, type SyntheticEvent } from "react";
import {
  VscAdd,
  VscChevronDown,
  VscChevronRight,
  VscChevronUp,
  VscCloudDownload,
  VscCode,
  VscFolderOpened,
  VscGlobe,
  VscRefresh,
  VscTerminal,
  VscTrash,
  VscWarning,
  VscWindow,
} from "react-icons/vsc";
import type { IconType } from "react-icons";
import {
  editorOpen,
  folderOpen,
  onRun,
  projectClone,
  projectCreate,
  projectSite,
  projectsList,
  runClear,
  runList,
  runLog,
  runPorts,
  runSetPort,
  runStart,
  runStop,
  terminalOpen,
  type AgentInfo,
  type EditorStatus,
  type Ports,
  type ProjectAction,
  type ProjectInfo,
  type RunInfo,
  type RunState,
  type SiteKind,
} from "./api";
import { AgentPicker } from "./agents";
import { ProjectPage } from "./projectpage";
import { GithubRepositories } from "./github";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { t } from "./i18n";
import { LanguageIcon } from "./logos";
import { Mark } from "./tags";

const MAX_LINES = 3000;

export const alive = (s: RunState | undefined) => s === "starting" || s === "running";

/** The actions started in this session, kept current by the "run" events. */
export function useRuns() {
  const [runs, setRuns] = useState<Record<string, RunInfo>>({});
  const [logs, setLogs] = useState<Record<string, string[]>>({});
  const reload = useCallback(() => {
    runList().then(
      (list) => {
        setRuns(Object.fromEntries(list.map((r) => [r.key, r])));
      },
      () => undefined,
    );
  }, []);
  useEffect(() => {
    reload();
    let stop: (() => void) | null = null;
    let gone = false;
    void onRun((e) => {
      if (e.kind === "state") {
        setRuns((r) => ({ ...r, [e.key]: e }));
      } else {
        setLogs((l) => {
          const lines = [...(l[e.key] ?? []), ...e.lines];
          return { ...l, [e.key]: lines.length > MAX_LINES ? lines.slice(-MAX_LINES) : lines };
        });
      }
    }).then((f) => {
      if (gone) f();
      else stop = f;
    });
    return () => {
      gone = true;
      stop?.();
    };
  }, [reload]);
  const loadLog = useCallback((key: string) => {
    runLog(key).then(
      (lines) => {
        setLogs((l) => ({ ...l, [key]: lines }));
      },
      () => undefined,
    );
  }, []);
  const forget = useCallback((key: string) => {
    void runClear(key).then(() => {
      setRuns((r) => Object.fromEntries(Object.entries(r).filter(([k]) => k !== key)));
    });
  }, []);
  return { runs, logs, loadLog, forget };
}

export function stateText(r: RunInfo): string {
  switch (r.state) {
    case "starting":
      return t("in avvio");
    case "running":
      return r.type === "web" ? t("in ascolto") : t("in esecuzione");
    case "finished":
      return t("terminato");
    case "stopped":
      return t("fermato");
    case "error":
      return r.code === null ? t("errore") : t("errore (codice {0})", r.code);
  }
}

export function kindText(k: ProjectAction["type"]): string {
  return { web: t("sito web"), desktop: t("app con finestra"), command: t("comando") }[k];
}

export function ProjectsPage({
  status,
  agents,
  onChanged,
  runs,
  logs,
  loadLog,
  forget,
}: {
  status: EditorStatus | null;
  agents: AgentInfo[] | null;
  onChanged: () => void;
  /** The actions started in this session (useRuns, kept by the app). */
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  loadLog: (key: string) => void;
  forget: (key: string) => void;
}) {
  const [projects, setProjects] = useState<ProjectInfo[] | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [open, setOpen] = useState<string | null>(null);
  /** The open panes of the side bar. */
  const [creating, setCreating] = useState<Set<NewWay>>(() => new Set(["empty"]));
  const [ports, setPorts] = useState<Ports>({ ports: {}, locked: [] });
  const { busy, error, run } = useAction(onChanged);

  const load = useCallback(() => {
    projectsList().then(
      (list) => {
        setLoadError(null);
        setProjects(list);
      },
      (e: unknown) => {
        setLoadError(String(e));
      },
    );
    runPorts().then(setPorts, () => undefined);
  }, []);
  useEffect(load, [load]);

  const created = (name: string) => {
    setOpen(name);
    load();
  };
  const start = (p: ProjectInfo, a: ProjectAction) => {
    const key = `${p.id}|${a.id}`;
    run(`run:${key}`, async () => {
      await runStart(p.id, a.id);
      loadLog(key);
      runPorts().then(setPorts, () => undefined);
    });
  };
  const page = open ? (projects ?? []).find((x) => x.id === open) : undefined;
  if (page) {
    return (
      <ProjectPage
        p={page}
        agents={agents}
        editor={status}
        runs={runs}
        logs={logs}
        ports={ports}
        busy={busy}
        onStart={(a) => {
          start(page, a);
        }}
        onStop={(key) => {
          run(`stop:${key}`, () => runStop(key));
        }}
        onPorts={setPorts}
        onShowLog={loadLog}
        onForget={forget}
        onBack={() => {
          setOpen(null);
        }}
        onChanged={load}
      />
    );
  }

  return (
    // The whole content area, as the editor's side bar: never taller than the window.
    <div className="absolute inset-0 flex">
      <aside className="flex w-[300px] shrink-0 flex-col overflow-hidden border-r border-vsc-border bg-vsc-sidebar">
        <div className="flex h-8 shrink-0 items-center gap-0.5 pr-2 pl-3">
          <h2 className="min-w-0 flex-1 truncate text-[12px] font-semibold text-vsc-sidebar-title">
            {t("Nuovo progetto")}
          </h2>
          <ToolbarIcon icon={VscRefresh} label={t("Aggiorna")} onClick={load} />
          <ToolbarIcon
            icon={VscFolderOpened}
            label={t("Apri la cartella")}
            onClick={() => {
              run("folder", () => folderOpen("projects"));
            }}
          />
        </div>
        {newWays().map((w) => (
          <SidePane
            key={w.id}
            title={w.label}
            fill={w.id === "github"}
            expanded={creating.has(w.id)}
            onToggle={() => {
              const next = new Set(creating);
              if (!next.delete(w.id)) next.add(w.id);
              setCreating(next);
            }}
          >
            {w.id === "empty" && <NewProject onDone={created} />}
            {w.id === "url" && <CloneProject onDone={created} />}
            {w.id === "github" && <GithubRepositories editor={status} onChanged={onChanged} />}
          </SidePane>
        ))}
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <ErrorLine error={loadError ?? error} />
        {projects === null ? (
          <p className="p-3 text-vsc-desc">{t("Caricamento…")}</p>
        ) : projects.length === 0 ? (
          <p className="p-3 text-vsc-desc">{t("Nessun progetto. Creane uno nuovo o clonane uno.")}</p>
        ) : (
          <ProjectsTable
            projects={projects}
            runs={runs}
            agents={agents}
            editor={status}
            busy={busy}
            onOpen={setOpen}
            onEditor={(p) => {
              run(`editor:${p.id}`, () => editorOpen(p.id));
            }}
            onTerminal={(p) => {
              run(`terminal:${p.id}`, () => terminalOpen(p.id));
            }}
          />
        )}
        <p className="shrink-0 border-t border-vsc-border px-3 py-1 text-[12px] text-vsc-desc">
          {t(
            "Le cartelle in projects sono attendibili per Claude e Codex e i loro comandi partono dai pulsanti qui sotto: apri qui solo codice di cui ti fidi.",
          )}
        </p>
      </div>
    </div>
  );
}

export const KIND_ICON: Record<ProjectInfo["type"], IconType> = {
  web: VscGlobe,
  desktop: VscWindow,
  command: VscTerminal,
};

/** How the project starts (site, app with a window, command) and what it
 * was recognised as (Laravel, FastAPI, static site...). */
export function ProjectKind({ p }: { p: ProjectInfo }) {
  const Icon = KIND_ICON[p.type];
  return (
    <span className="flex min-w-0 items-center gap-1.5 text-vsc-desc" title={`${kindText(p.type)} · ${p.reason}`}>
      <Mark label={kindText(p.type)}>
        <Icon size={14} aria-hidden="true" />
      </Mark>
      <span>{p.platform}</span>
    </span>
  );
}

/** Every language as its logo, in alphabetical order. */
export function LanguageIcons({ names }: { names: string[] }) {
  return (
    <span className="flex items-center gap-1.5">
      {[...names]
        .sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }))
        .map((l) => (
          <LanguageIcon key={l} name={l} />
        ))}
    </span>
  );
}

/** All the projects as a table, as the editor's lists: a filter, headers
 * that stay put and sort, one 26px row per project. */
function ProjectsTable({
  projects,
  runs,
  agents,
  editor,
  busy,
  onOpen,
  onEditor,
  onTerminal,
}: {
  projects: ProjectInfo[];
  runs: Record<string, RunInfo>;
  agents: AgentInfo[] | null;
  editor: EditorStatus | null;
  busy: string | null;
  onOpen: (id: string) => void;
  onEditor: (p: ProjectInfo) => void;
  onTerminal: (p: ProjectInfo) => void;
}) {
  const [filter, setFilter] = useState("");
  const [desc, setDesc] = useState(false);
  const words = filter.toLowerCase().split(/\s+/).filter(Boolean);
  const shown = projects
    .filter((p) => {
      const text = `${p.name} ${p.platform} ${p.languages.join(" ")}`.toLowerCase();
      return words.every((w) => text.includes(w));
    })
    .sort((a, b) => {
      const d = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
      return desc ? -d : d;
    });
  return (
    <>
      <div className="flex shrink-0 items-center gap-2 px-3 py-2">
        <input
          type="search"
          aria-label={t("Cerca nei progetti")}
          placeholder={t("Cerca nei progetti")}
          value={filter}
          onChange={(e) => {
            setFilter(e.target.value);
          }}
          className="input w-64"
        />
        <span className="text-[12px] text-vsc-desc">
          {shown.length === projects.length
            ? t("{0} progetti", projects.length)
            : t("{0} di {1} progetti", shown.length, projects.length)}
        </span>
      </div>
      <div className="min-h-0 flex-1 overflow-auto">
        {shown.length === 0 ? (
          <p className="px-3 text-vsc-desc">{t("Nessun progetto corrisponde alla ricerca.")}</p>
        ) : (
          <table aria-label={t("Progetti")} className="w-full border-collapse text-left">
            <thead className="sticky top-0 z-10 bg-vsc-bg text-[12px] text-vsc-desc shadow-[inset_0_-1px_0_var(--vsc-border)]">
              <tr>
                <th scope="col" className="w-6 p-0" />
                <th scope="col" aria-sort={desc ? "descending" : "ascending"} className="p-0 font-normal">
                  <button
                    type="button"
                    onClick={() => {
                      setDesc(!desc);
                    }}
                    className="flex w-full items-center gap-1 px-2 py-1 text-left hover:text-vsc-fg"
                  >
                    {t("Nome")}
                    {desc ? (
                      <VscChevronUp size={12} aria-hidden="true" />
                    ) : (
                      <VscChevronDown size={12} aria-hidden="true" />
                    )}
                  </button>
                </th>
                <th scope="col" className="w-px px-2 py-1 font-normal whitespace-nowrap">
                  {t("Tipo")}
                </th>
                <th scope="col" className="w-px px-2 py-1 font-normal whitespace-nowrap">
                  {t("Linguaggi")}
                </th>
                <th scope="col" className="w-px px-2 py-1 font-normal whitespace-nowrap">
                  {t("Stato")}
                </th>
                <th scope="col" className="w-px px-2 py-1 font-normal">
                  <span className="sr-only">{t("Comandi")}</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {shown.map((p) => {
                const live = Object.values(runs).filter((r) => r.project === p.id && alive(r.state));
                return (
                  <tr
                    key={p.id}
                    onClick={(e) => {
                      // The row opens the project; its buttons do their own thing.
                      if (!(e.target as HTMLElement).closest("button, input, a")) onOpen(p.id);
                    }}
                    className="group h-[26px] cursor-pointer hover:bg-vsc-list-hover"
                  >
                    <td className="p-0 pl-2">
                      <VscChevronRight size={14} aria-hidden="true" className="text-vsc-desc" />
                    </td>
                    <td className="max-w-0 px-2">
                      <span className="flex min-w-0 items-baseline gap-2">
                        <button
                          type="button"
                          title={t("Apre la pagina di {0}", p.name)}
                          onClick={() => {
                            onOpen(p.id);
                          }}
                          className="min-w-0 shrink-0 truncate text-left font-semibold"
                        >
                          {p.name}
                        </button>
                      </span>
                    </td>
                    <td className="px-2 whitespace-nowrap">
                      <ProjectKind p={p} />
                    </td>
                    <td className="px-2 whitespace-nowrap">
                      <LanguageIcons names={p.languages} />
                    </td>
                    <td className="px-2 text-[12px] whitespace-nowrap">
                      {live.length > 0 ? (
                        <span className="text-vsc-success" title={live.map((r) => r.label).join(", ")}>
                          ● {live.some((r) => r.type === "web") ? t("in ascolto") : t("in esecuzione")}
                          {live.length > 1 && ` (${String(live.length)})`}
                        </span>
                      ) : p.missing.length > 0 ? (
                        <span
                          className="flex items-center gap-1 text-vsc-warning"
                          title={t("Mancano componenti: {0}", p.missing.join(", "))}
                        >
                          <VscWarning size={12} aria-hidden="true" />
                          {t("Mancano componenti: {0}", p.missing.join(", "))}
                        </span>
                      ) : null}
                    </td>
                    <td className="px-2 whitespace-nowrap">
                      <span className="flex items-center justify-end gap-1 opacity-70 group-focus-within:opacity-100 group-hover:opacity-100">
                        <ActionButton
                          icon={VscCode}
                          label={t("Editor")}
                          compact
                          busy={busy === `editor:${p.id}`}
                          disabled={editor?.installed === false}
                          title={t("Apre {0} nell'editor", p.name)}
                          onClick={() => {
                            onEditor(p);
                          }}
                        />
                        <ActionButton
                          icon={VscTerminal}
                          label={t("Terminale")}
                          compact
                          busy={busy === `terminal:${p.id}`}
                          title={t("Console del kit in {0}", p.name)}
                          onClick={() => {
                            onTerminal(p);
                          }}
                        />
                        <AgentPicker agents={agents} project={p.id} preferred={p.settings.agent} compact />
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </>
  );
}

type NewWay = "empty" | "github" | "url";

/** The ways to make a new project, each a pane of the side bar. */
function newWays(): { id: NewWay; label: string }[] {
  return [
    { id: "empty", label: t("Vuoto") },
    { id: "github", label: t("Dai tuoi repository GitHub") },
    { id: "url", label: t("Da un indirizzo Git") },
  ];
}

/** An icon of a side bar's title, as in the editor. */
export function ToolbarIcon({ icon: Icon, label, onClick }: { icon: IconType; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className="flex size-[22px] items-center justify-center rounded p-[3px] text-vsc-icon hover:bg-vsc-toolbar-hover"
    >
      <Icon size={16} aria-hidden="true" />
    </button>
  );
}

/** A collapsible pane of the side bar, as OPEN EDITORS or OUTLINE in the
 * editor. It opens with a slide and scrolls inside; a `fill` pane takes the
 * height left by the others. */
export function SidePane({
  title,
  expanded,
  fill = false,
  onToggle,
  children,
}: {
  title: string;
  expanded: boolean;
  fill?: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  return (
    <section
      // Only the `fill` pane gives up height: the small forms stay whole.
      className={`flex flex-col border-t border-vsc-border transition-[flex-grow] duration-200 ease-out ${
        fill ? "min-h-[22px] shrink" : "shrink-0"
      }`}
      style={{ flexGrow: fill && expanded ? 1 : 0, flexBasis: fill && expanded ? 0 : "auto" }}
    >
      <h3 className="shrink-0">
        <button
          type="button"
          aria-expanded={expanded}
          onClick={onToggle}
          className="flex h-[22px] w-full items-center gap-0.5 rounded px-0.5 text-left text-[11px] font-semibold text-vsc-section-fg hover:bg-vsc-list-hover"
        >
          <VscChevronRight
            size={16}
            aria-hidden="true"
            className={`shrink-0 transition-transform duration-200 ${expanded ? "rotate-90" : ""}`}
          />
          <span className="truncate">{title}</span>
        </button>
      </h3>
      <div
        inert={!expanded}
        className="grid min-h-0 flex-1 transition-[grid-template-rows] duration-200 ease-out"
        style={{ gridTemplateRows: expanded ? "1fr" : "0fr" }}
      >
        <div className="min-h-0 overflow-y-auto">
          <div className="px-3 pt-1 pb-3">{children}</div>
        </div>
      </div>
    </section>
  );
}

/** The port of a web action: automatic (kept between starts) or chosen. */
export function PortField({
  actionKey,
  ports,
  onPorts,
}: {
  actionKey: string;
  ports: Ports;
  onPorts: (p: Ports) => void;
}) {
  const current = ports.ports[actionKey];
  const locked = ports.locked.includes(actionKey);
  const [value, setValue] = useState(current === undefined ? "" : String(current));
  const [error, setError] = useState<string | null>(null);
  const save = () => {
    const text = value.trim();
    const port = text === "" ? null : Number(text);
    if (port === (locked ? (current ?? null) : null)) return;
    if (port !== null && (!Number.isInteger(port) || port < 1024 || port > 65535)) {
      setError(t("Porta tra {0} e {1}", 1024, 65535));
      return;
    }
    runSetPort(actionKey, port).then(
      (p) => {
        setError(null);
        onPorts(p);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  };
  return (
    <div>
      <input
        type="text"
        inputMode="numeric"
        aria-label={t("Porta")}
        placeholder={t("automatica")}
        value={value}
        onChange={(e) => {
          setValue(e.target.value);
        }}
        onBlur={save}
        onKeyDown={(e) => {
          if (e.key === "Enter") save();
        }}
        title={
          locked
            ? t("Porta scelta da te")
            : t("Porta automatica, la stessa a ogni avvio. Scrivi un numero per sceglierla.")
        }
        className="w-24 input"
      />
      {error && (
        <p role="alert" className="text-[12px] text-vsc-error">
          {error}
        </p>
      )}
    </div>
  );
}

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
        <span className="text-[12px] text-vsc-desc">{t("Servi come sito web la cartella")}</span>
        <input
          value={folder}
          onChange={(e) => {
            setFolder(e.target.value);
          }}
          className="w-full input"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{t("con")}</span>
        <select
          value={kind}
          onChange={(e) => {
            setKind(e.target.value as SiteKind);
          }}
          className="input"
        >
          <option value="static">{t("file statici")}</option>
          <option value="php">{t("PHP (server integrato)")}</option>
          <option value="apache">{t("Apache dedicato")}</option>
        </select>
      </label>
      <ActionButton icon={VscGlobe} label={t("Salva")} busy={busy === "site"} onClick={() => undefined} submit />
      {site && (
        <ActionButton
          icon={VscTrash}
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
      <ErrorLine error={error} />
    </form>
  );
}

export function LogView({ title, lines }: { title: string; lines: string[] }) {
  const ref = useRef<HTMLPreElement>(null);
  const [follow, setFollow] = useState(true);
  useEffect(() => {
    const el = ref.current;
    if (el && follow) el.scrollTop = el.scrollHeight;
  }, [lines, follow]);
  return (
    <div className="border-t border-vsc-border">
      <div className="flex items-center gap-2 px-3 py-1 text-[12px] text-vsc-desc">
        <span className="flex-1 truncate">{t("Output di {0}", title)}</span>
        <label className="flex items-center gap-1">
          <input
            type="checkbox"
            checked={follow}
            onChange={(e) => {
              setFollow(e.target.checked);
            }}
          />
          {t("Segui")}
        </label>
      </div>
      <pre
        ref={ref}
        tabIndex={0}
        aria-label={t("Output di {0}", title)}
        className="m-0 h-64 overflow-auto bg-vsc-bg px-3 py-2 font-mono text-[12px] leading-snug whitespace-pre-wrap select-text"
      >
        {lines.join("\n")}
      </pre>
    </div>
  );
}

function NewProject({ onDone }: { onDone: (name: string) => void }) {
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("create", async () => {
      onDone(await projectCreate(name));
    });
  };
  return (
    <form onSubmit={submit} className="space-y-1">
      <label htmlFor="new-project-name" className="block text-[12px] text-vsc-desc">
        {t("Nome della cartella")}
      </label>
      <div className="flex items-center gap-1">
        <input
          id="new-project-name"
          required
          value={name}
          onChange={(e) => {
            setName(e.target.value);
          }}
          className="input min-w-0 flex-1"
        />
        <ActionButton
          icon={VscAdd}
          label={t("Crea")}
          primary
          compact
          submit
          busy={busy === "create"}
          onClick={() => undefined}
        />
      </div>
      <ErrorLine error={error} />
    </form>
  );
}

function CloneProject({ onDone }: { onDone: (name: string) => void }) {
  const [source, setSource] = useState("");
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("clone", async () => {
      onDone(await projectClone(source, name.trim() === "" ? null : name));
    });
  };
  return (
    <form onSubmit={submit} className="space-y-2">
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{t("owner/repository di GitHub o indirizzo https://")}</span>
        <input
          required
          value={source}
          onChange={(e) => {
            setSource(e.target.value);
          }}
          className="w-full input"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-[12px] text-vsc-desc">{t("Nome della cartella (facoltativo)")}</span>
        <input
          value={name}
          onChange={(e) => {
            setName(e.target.value);
          }}
          className="w-full input"
        />
      </label>
      <ActionButton
        icon={VscCloudDownload}
        label={t("Clona")}
        primary
        submit
        busy={busy === "clone"}
        onClick={() => undefined}
      />
      <ErrorLine error={error} />
    </form>
  );
}
