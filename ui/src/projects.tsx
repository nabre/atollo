// Projects page, as VSCodium's Explorer: the side bar lists the projects in
// projects\ (detected by tauri/src/detect, corrected by panel.json) with
// their inline actions, and the ways to make a new one; the editor opens
// each project in its own tab (projectpage.tsx). Web actions get their own
// port and open in the browser.
import { useCallback, useEffect, useRef, useState, type SyntheticEvent } from "react";
import {
  VscCloudDownload,
  VscCode,
  VscFolderOpened,
  VscGlobe,
  VscNewFolder,
  VscRefresh,
  VscRepoClone,
  VscTerminal,
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
import { AgentButton } from "./agents";
import { ProjectPage } from "./projectpage";
import { GithubRepositories } from "./github";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { t } from "./i18n";
import { LanguageIcon } from "./logos";
import { Mark } from "./tags";
import { BrandSymbol } from "./brand";
import { useSubscription } from "./hooks";
import {
  Button,
  Dot,
  EditorTabs,
  IconButton,
  Link,
  List,
  ListRow,
  PaneStack,
  SearchBox,
  SideBarTitle,
  ViewContainer,
  ViewPane,
  Watermark,
  type useTabs,
} from "./workbench";

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
  useEffect(reload, [reload]);
  useSubscription(onRun, (e) => {
    if (e.kind === "state") {
      setRuns((r) => ({ ...r, [e.key]: e }));
    } else {
      setLogs((l) => {
        const lines = [...(l[e.key] ?? []), ...e.lines];
        return { ...l, [e.key]: lines.length > MAX_LINES ? lines.slice(-MAX_LINES) : lines };
      });
    }
  });
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
    <span className="flex min-w-0 items-center gap-1.5" title={`${kindText(p.type)} · ${p.reason}`}>
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

/** The project's state as the decoration of its row: running, or missing
 * components. */
export function ProjectState({ p, runs }: { p: ProjectInfo; runs: Record<string, RunInfo> }) {
  const live = Object.values(runs).filter((r) => r.project === p.id && alive(r.state));
  if (live.length > 0) {
    const text = live.some((r) => r.type === "web") ? t("in ascolto") : t("in esecuzione");
    return <Dot tone="ok" label={`${text}: ${live.map((r) => r.label).join(", ")}`} />;
  }
  if (p.missing.length > 0) {
    return (
      <Mark label={t("Mancano componenti: {0}", p.missing.join(", "))} className="text-vsc-warning">
        <VscWarning size={14} aria-hidden="true" />
      </Mark>
    );
  }
  return null;
}

type PaneId = "projects" | "new" | "github";

export function ProjectsPage({
  status,
  agents,
  onChanged,
  runs,
  logs,
  loadLog,
  forget,
  tabs,
}: {
  status: EditorStatus | null;
  agents: AgentInfo[] | null;
  onChanged: () => void;
  /** The actions started in this session (useRuns, kept by the app). */
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  loadLog: (key: string) => void;
  forget: (key: string) => void;
  /** The projects open in the editor (kept by the app across pages). */
  tabs: ReturnType<typeof useTabs>;
}) {
  const [projects, setProjects] = useState<ProjectInfo[] | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [panes, setPanes] = useState<Set<PaneId>>(() => new Set(["projects"]));
  const [filter, setFilter] = useState("");
  const [ports, setPorts] = useState<Ports>({ ports: {}, locked: [] });
  const newName = useRef<HTMLInputElement>(null);
  const { busy, error, run } = useAction(onChanged);
  const { keep } = tabs;

  const load = useCallback(() => {
    projectsList().then(
      (list) => {
        setLoadError(null);
        setProjects(list);
        keep(list.map((p) => p.id));
      },
      (e: unknown) => {
        setLoadError(String(e));
      },
    );
    runPorts().then(setPorts, () => undefined);
  }, [keep]);
  useEffect(load, [load]);

  const toggle = (id: PaneId, open?: boolean) => {
    setPanes((current) => {
      const next = new Set(current);
      if (open ?? !next.has(id)) next.add(id);
      else next.delete(id);
      return next;
    });
  };
  const created = (name: string) => {
    tabs.show(name);
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

  const words = filter.toLowerCase().split(/\s+/).filter(Boolean);
  const shown = (projects ?? [])
    .filter((p) => {
      const text = `${p.name} ${p.platform} ${p.languages.join(" ")}`.toLowerCase();
      return words.every((w) => text.includes(w));
    })
    .sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" }));
  const byId = (id: string) => (projects ?? []).find((p) => p.id === id);
  const page = tabs.active ? byId(tabs.active) : undefined;

  const side = (
    <>
      <SideBarTitle
        title={t("Progetti")}
        actions={
          <>
            <IconButton
              icon={VscNewFolder}
              label={t("Nuovo progetto")}
              onClick={() => {
                toggle("new", true);
                window.setTimeout(() => newName.current?.focus(), 0);
              }}
            />
            <IconButton icon={VscRefresh} label={t("Aggiorna")} onClick={load} />
            <IconButton
              icon={VscFolderOpened}
              label={t("Apri la cartella")}
              onClick={() => {
                run("folder", () => folderOpen("projects"));
              }}
            />
          </>
        }
      />
      <PaneStack>
        <ViewPane
          first
          fill
          title={t("Progetti")}
          badge={projects && projects.length > 0 ? projects.length : undefined}
          expanded={panes.has("projects")}
          onToggle={() => {
            toggle("projects");
          }}
        >
          {projects && projects.length > 6 && (
            <div className="px-3 pt-1 pb-1.5">
              <SearchBox value={filter} onChange={setFilter} placeholder={t("Cerca nei progetti")} />
            </div>
          )}
          <div className="px-3 empty:hidden">
            <ErrorLine error={loadError ?? error} />
          </div>
          {projects === null ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
          ) : projects.length === 0 ? (
            <div className="space-y-3 px-5 py-2">
              <p>{t("Nessun progetto. Creane uno nuovo o clonane uno.")}</p>
              <Button
                variant="primary"
                wide
                onClick={() => {
                  toggle("new", true);
                  window.setTimeout(() => newName.current?.focus(), 0);
                }}
              >
                {t("Nuovo progetto")}
              </Button>
            </div>
          ) : shown.length === 0 ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Nessun progetto corrisponde alla ricerca.")}</p>
          ) : (
            <List label={t("Progetti")}>
              {shown.map((p) => {
                const Icon = KIND_ICON[p.type];
                return (
                  <ListRow
                    key={p.id}
                    icon={<Icon size={16} aria-hidden="true" />}
                    label={p.name}
                    title={t("Apre la pagina di {0}", p.name)}
                    selected={tabs.active === p.id}
                    onSelect={() => {
                      tabs.show(p.id);
                    }}
                    actions={
                      <>
                        <IconButton
                          icon={VscCode}
                          label={t("Editor")}
                          busy={busy === `editor:${p.id}`}
                          disabled={status?.installed === false}
                          title={t("Apre {0} nell'editor", p.name)}
                          onClick={() => {
                            run(`editor:${p.id}`, () => editorOpen(p.id));
                          }}
                        />
                        <IconButton
                          icon={VscTerminal}
                          label={t("Terminale")}
                          busy={busy === `terminal:${p.id}`}
                          title={t("Console del kit in {0}", p.name)}
                          onClick={() => {
                            run(`terminal:${p.id}`, () => terminalOpen(p.id));
                          }}
                        />
                        <AgentButton agents={agents} project={p.id} preferred={p.settings.agent} />
                      </>
                    }
                    decorations={<ProjectState p={p} runs={runs} />}
                  />
                );
              })}
            </List>
          )}
        </ViewPane>
        <ViewPane
          title={t("Nuovo progetto")}
          expanded={panes.has("new")}
          onToggle={() => {
            toggle("new");
          }}
        >
          <div className="space-y-4 px-5 pt-1 pb-4">
            <NewProject onDone={created} inputRef={newName} />
            <CloneProject onDone={created} />
          </div>
        </ViewPane>
        <ViewPane
          fill
          title={t("Dai tuoi repository GitHub")}
          expanded={panes.has("github")}
          onToggle={() => {
            toggle("github");
          }}
        >
          <div className="px-3 pt-1 pb-3">
            <GithubRepositories editor={status} onChanged={onChanged} />
          </div>
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="projects" side={side}>
      {tabs.open.length > 0 && (
        <EditorTabs
          tabs={tabs.open.map((id) => {
            const p = byId(id);
            const Icon = p ? KIND_ICON[p.type] : VscGlobe;
            return { id, label: p?.name ?? id, icon: <Icon size={16} aria-hidden="true" /> };
          })}
          active={tabs.active}
          onSelect={tabs.setActive}
          onClose={tabs.close}
        />
      )}
      {page ? (
        <ProjectPage
          key={page.id}
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
          onChanged={load}
        />
      ) : (
        <Watermark>
          <BrandSymbol size={96} />
          <div className="w-full max-w-[420px] space-y-4">
            <section aria-labelledby="start-title" className="space-y-1.5">
              <h2 id="start-title" className="text-[13px] font-semibold text-vsc-settings-header">
                {t("Inizia")}
              </h2>
              <ul className="m-0 list-none space-y-1 p-0">
                <li className="flex items-center gap-2">
                  <VscNewFolder size={16} aria-hidden="true" className="text-vsc-link" />
                  <Link
                    onClick={() => {
                      toggle("new", true);
                      window.setTimeout(() => newName.current?.focus(), 0);
                    }}
                  >
                    {t("Nuovo progetto…")}
                  </Link>
                </li>
                <li className="flex items-center gap-2">
                  <VscRepoClone size={16} aria-hidden="true" className="text-vsc-link" />
                  <Link
                    onClick={() => {
                      toggle("github", true);
                    }}
                  >
                    {t("Clona dai tuoi repository GitHub…")}
                  </Link>
                </li>
                <li className="flex items-center gap-2">
                  <VscFolderOpened size={16} aria-hidden="true" className="text-vsc-link" />
                  <Link
                    onClick={() => {
                      run("folder", () => folderOpen("projects"));
                    }}
                  >
                    {t("Apri la cartella dei progetti")}
                  </Link>
                </li>
              </ul>
            </section>
            <p className="text-[12px] text-vsc-desc">
              {t(
                "Le cartelle in projects sono attendibili per Claude e Codex e i loro comandi partono dai pulsanti dei progetti: apri qui solo codice di cui ti fidi.",
              )}
            </p>
          </div>
        </Watermark>
      )}
    </ViewContainer>
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

/** The output of an action, as the editor's Output view. */
export function LogView({ title, lines }: { title: string; lines: string[] }) {
  const ref = useRef<HTMLPreElement>(null);
  const [follow, setFollow] = useState(true);
  useEffect(() => {
    const el = ref.current;
    if (el && follow) el.scrollTop = el.scrollHeight;
  }, [lines, follow]);
  return (
    <div className="overflow-hidden rounded border border-vsc-border">
      <div className="flex h-[26px] items-center gap-2 border-b border-vsc-border px-3 text-[11px] text-vsc-desc uppercase">
        <span className="flex-1 truncate">{t("Output di {0}", title)}</span>
        <label className="flex items-center gap-1 normal-case">
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
        className="m-0 h-64 overflow-auto px-3 py-2 font-mono text-[12px] leading-[18px] whitespace-pre-wrap select-text"
      >
        {lines.join("\n")}
      </pre>
    </div>
  );
}

function NewProject({
  onDone,
  inputRef,
}: {
  onDone: (name: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
}) {
  const [name, setName] = useState("");
  const { busy, error, run } = useAction();
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("create", async () => {
      onDone(await projectCreate(name));
    });
  };
  return (
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Vuoto")}>
      <label htmlFor="new-project-name" className="block font-semibold">
        {t("Vuoto")}
      </label>
      <p className="text-[12px] text-vsc-desc">{t("Nome della cartella")}</p>
      <input
        id="new-project-name"
        ref={inputRef}
        required
        value={name}
        onChange={(e) => {
          setName(e.target.value);
        }}
        className="input w-full"
      />
      <Button variant="primary" wide submit busy={busy === "create"}>
        {t("Crea")}
      </Button>
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
    <form onSubmit={submit} className="space-y-1.5" aria-label={t("Da un indirizzo Git")}>
      <p className="font-semibold">{t("Da un indirizzo Git")}</p>
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
      <Button wide submit busy={busy === "clone"}>
        <VscCloudDownload size={14} aria-hidden="true" />
        {t("Clona")}
      </Button>
      <ErrorLine error={error} />
    </form>
  );
}
