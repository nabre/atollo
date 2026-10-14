// Projects page: what each project in projects\ is, how it starts (detected
// by src-tauri/src/detect, corrected by panel.json) and its running actions
// with their output. Web actions get their own port and open in the browser.
import { useCallback, useEffect, useRef, useState, type ReactNode, type SyntheticEvent } from "react";
import {
  VscAdd,
  VscChevronRight,
  VscCloudDownload,
  VscCode,
  VscDebugStart,
  VscDebugStop,
  VscDiscard,
  VscEdit,
  VscFolderOpened,
  VscGlobe,
  VscOutput,
  VscRefresh,
  VscStarEmpty,
  VscTerminal,
  VscTrash,
  VscWarning,
} from "react-icons/vsc";
import type { IconType } from "react-icons";
import {
  editorOpen,
  folderOpen,
  onRun,
  projectClone,
  projectCreate,
  projectDelete,
  projectActionDefault,
  projectActionRemove,
  projectInstallMissing,
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
  urlOpen,
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
import { ProjectExtensions } from "./extensions";
import { ProjectPage } from "./projectpage";
import { ActionForm } from "./actionform";
import { GithubRepositories } from "./github";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { t } from "./i18n";
import { LanguageIcon } from "./logos";

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
  const [shownLog, setShownLog] = useState<string | null>(null);
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
  const showLog = (key: string) => {
    setShownLog(key);
    loadLog(key);
  };
  const start = (p: ProjectInfo, a: ProjectAction) => {
    const key = `${p.id}|${a.id}`;
    run(`run:${key}`, async () => {
      setShownLog(key);
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
        onShowLog={showLog}
        shownLog={shownLog}
        onForget={(key) => {
          if (shownLog === key) setShownLog(null);
          forget(key);
        }}
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
      <aside className="flex w-72 shrink-0 flex-col overflow-hidden border-r border-vsc-border bg-vsc-sidebar">
        <div className="flex h-[35px] shrink-0 items-center gap-0.5 pr-2 pl-5">
          <h2 className="min-w-0 flex-1 truncate text-[11px] font-normal tracking-wide uppercase">
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
      <div className="min-w-0 flex-1 space-y-3 overflow-y-auto p-3">
        <p className="text-vsc-desc">
          {t(
            "Le cartelle in projects sono attendibili per Claude e Codex e i loro comandi partono dai pulsanti qui sotto: apri qui solo codice di cui ti fidi.",
          )}
        </p>
        <ErrorLine error={loadError ?? error} />
        {projects === null ? (
          <p className="text-vsc-desc">{t("Caricamento…")}</p>
        ) : projects.length === 0 ? (
          <p className="text-vsc-desc">{t("Nessun progetto. Creane uno nuovo o clonane uno.")}</p>
        ) : (
          <ul aria-label={t("Progetti")} className="-mx-3">
            {projects.map((p) => {
              return (
                <li key={p.id}>
                  <div className="flex items-center gap-2 px-3 py-0.5 hover:bg-vsc-list-hover">
                    <button
                      type="button"
                      title={t("Apre la pagina di {0}", p.name)}
                      onClick={() => {
                        setOpen(p.id);
                      }}
                      className="flex min-w-0 flex-1 items-center gap-2 text-left"
                    >
                      <VscChevronRight size={16} aria-hidden="true" className="shrink-0" />
                      <span className="truncate font-semibold">{p.name}</span>
                      <span className="truncate text-[12px] text-vsc-desc">{p.platform}</span>
                      {p.languages.length > 0 && (
                        <span className="flex min-w-0 flex-wrap items-center gap-1.5">
                          {p.languages.slice(0, 6).map((l) => (
                            <LanguageIcon key={l} name={l} />
                          ))}
                          {p.languages.length > 6 && (
                            <span className="text-[11px] text-vsc-desc">+{String(p.languages.length - 6)}</span>
                          )}
                        </span>
                      )}
                      {p.missing.length > 0 && (
                        <VscWarning
                          size={14}
                          className="shrink-0 text-vsc-warning"
                          aria-label={t("Mancano componenti: {0}", p.missing.join(", "))}
                        />
                      )}
                    </button>
                    <ActionButton
                      icon={VscCode}
                      label={t("Editor")}
                      compact
                      busy={busy === `editor:${p.id}`}
                      disabled={status?.installed === false}
                      title={t("Apre {0} nell'editor", p.name)}
                      onClick={() => {
                        run(`editor:${p.id}`, () => editorOpen(p.id));
                      }}
                    />
                    <ActionButton
                      icon={VscTerminal}
                      label={t("Terminale")}
                      compact
                      busy={busy === `terminal:${p.id}`}
                      title={t("Console del kit in {0}", p.name)}
                      onClick={() => {
                        run(`terminal:${p.id}`, () => terminalOpen(p.id));
                      }}
                    />
                    <AgentPicker agents={agents} project={p.id} preferred={p.settings.agent} compact />
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
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
function ToolbarIcon({ icon: Icon, label, onClick }: { icon: IconType; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className="rounded-sm p-1 text-vsc-fg hover:bg-vsc-list-hover"
    >
      <Icon size={16} aria-hidden="true" />
    </button>
  );
}

/** A collapsible pane of the side bar, as OPEN EDITORS or OUTLINE in the
 * editor. It opens with a slide and scrolls inside; a `fill` pane takes the
 * height left by the others. */
function SidePane({
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
          className="flex h-[22px] w-full items-center gap-0.5 px-0.5 text-left text-[11px] font-bold uppercase hover:bg-vsc-list-hover"
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

export function ProjectDetails({
  project: p,
  runs,
  ports,
  busy,
  onStart,
  onStop,
  onPorts,
  onChanged,
  onlyRun = false,
}: {
  /** Only what starts the project (its page shows the rest in other tabs). */
  onlyRun?: boolean;
  project: ProjectInfo;
  runs: Record<string, RunInfo>;
  ports: Ports;
  busy: string | null;
  onStart: (a: ProjectAction) => void;
  onStop: (key: string) => void;
  onPorts: (p: Ports) => void;
  onChanged: () => void;
}) {
  const { busy: working, error, run } = useAction(onChanged);
  /** The action being edited ("new": a new one). */
  const [editing, setEditing] = useState<string | null>(null);
  return (
    <div id={`project-${p.id}`} className="space-y-2 border-t border-vsc-border bg-vsc-sidebar px-3 py-2">
      {!onlyRun && <p className="text-[12px] text-vsc-desc">{p.reason}</p>}
      {!onlyRun && p.missing.length > 0 && (
        <div className="flex flex-wrap items-center gap-2">
          <p role="status" className="flex items-start gap-2 text-vsc-warning">
            <VscWarning size={16} aria-hidden="true" className="mt-0.5 shrink-0" />
            {t("Mancano componenti del kit: {0}.", p.missing.join(", "))}
          </p>
          <ActionButton
            icon={VscCloudDownload}
            label={t("Installa")}
            primary
            busy={working === "install"}
            title={t("Installa {0} e li tiene in Strumenti", p.missing.join(", "))}
            onClick={() => {
              run("install", () => projectInstallMissing(p.id));
            }}
          />
        </div>
      )}
      {!onlyRun && p.warnings.length > 0 && (
        <ul className="list-disc space-y-1 pl-5 text-vsc-desc">
          {p.warnings.map((w) => (
            <li key={w}>{w}</li>
          ))}
        </ul>
      )}
      {p.actions.length === 0 ? (
        <p className="text-vsc-desc">
          {t("Nessuna azione riconosciuta: aggiungine in panel.json o servi una cartella come sito web.")}
        </p>
      ) : (
        <table className="w-full text-left text-[13px]">
          <caption className="sr-only">{t("Azioni di {0}", p.name)}</caption>
          <thead className="text-[12px] text-vsc-desc">
            <tr>
              <th scope="col" className="py-1 font-normal">
                {t("Azione")}
              </th>
              <th scope="col" className="py-1 font-normal">
                {t("Tipo")}
              </th>
              <th scope="col" className="py-1 font-normal">
                {t("Porta")}
              </th>
              <th scope="col" className="py-1 font-normal">
                <span className="sr-only">{t("Comandi")}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {p.actions.map((a) => {
              const key = `${p.id}|${a.id}`;
              const r = runs[key];
              return (
                <tr key={a.id} className="border-t border-vsc-border align-middle">
                  <td className="py-1 pr-3">
                    <div>
                      {a.label}
                      {p.default === a.id && <span className="ml-2 text-[11px] text-vsc-desc">{t("predefinita")}</span>}
                      {a.custom && <span className="ml-2 text-[11px] text-vsc-desc">panel.json</span>}
                    </div>
                    <div className="font-mono text-[11px] break-all text-vsc-desc select-text">
                      {a.static !== null ? t("file server del kit: {0}", a.static) : a.command}
                    </div>
                  </td>
                  <td className="py-1 pr-3 text-vsc-desc">
                    {kindText(a.type)}
                    {r && ` · ${stateText(r)}`}
                  </td>
                  <td className="py-1 pr-3">
                    {a.type === "web" && (
                      // Remounted when the saved port changes: the field shows it.
                      <PortField
                        key={`${key}:${String(ports.ports[key] ?? "")}`}
                        actionKey={key}
                        ports={ports}
                        onPorts={onPorts}
                      />
                    )}
                  </td>
                  <td className="py-1">
                    <div className="flex flex-wrap justify-end gap-2">
                      {onlyRun && a.static === null && (
                        <ActionButton
                          icon={VscEdit}
                          label={t("Modifica")}
                          onClick={() => {
                            setEditing(editing === a.id ? null : a.id);
                          }}
                        />
                      )}
                      {onlyRun && p.default !== a.id && (
                        <ActionButton
                          icon={VscStarEmpty}
                          label={t("Predefinita")}
                          title={t("La avvia il pulsante Avvia del progetto")}
                          busy={working === `default:${a.id}`}
                          onClick={() => {
                            run(`default:${a.id}`, () => projectActionDefault(p.id, a.id));
                          }}
                        />
                      )}
                      {onlyRun && a.custom && a.static === null && (
                        <ActionButton
                          icon={VscDiscard}
                          label={t("Togli")}
                          title={t("Toglie l'azione da panel.json; se era rilevata, torna quella rilevata")}
                          busy={working === `remove:${a.id}`}
                          onClick={() => {
                            run(`remove:${a.id}`, () => projectActionRemove(p.id, a.id));
                          }}
                        />
                      )}
                      {alive(r?.state) ? (
                        <ActionButton
                          icon={VscDebugStop}
                          label={t("Ferma")}
                          busy={busy === `stop:${key}`}
                          onClick={() => {
                            onStop(key);
                          }}
                        />
                      ) : (
                        <ActionButton
                          icon={VscDebugStart}
                          label={t("Avvia")}
                          busy={busy === `run:${key}`}
                          onClick={() => {
                            onStart(a);
                          }}
                        />
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
      {onlyRun && editing !== null && (
        <ActionForm
          project={p.id}
          action={editing === "new" ? null : (p.actions.find((a) => a.id === editing) ?? null)}
          onDone={() => {
            setEditing(null);
            onChanged();
          }}
          onCancel={() => {
            setEditing(null);
          }}
        />
      )}
      {onlyRun && editing === null && (
        <ActionButton
          icon={VscAdd}
          label={t("Nuova azione")}
          onClick={() => {
            setEditing("new");
          }}
        />
      )}
      <SiteForm project={p} onChanged={onChanged} />
      {!onlyRun && <ProjectExtensions project={p} onChanged={onChanged} />}
      {p.empty && (
        <ActionButton
          icon={VscTrash}
          label={t("Elimina la cartella vuota")}
          busy={working === "delete"}
          onClick={() => {
            run("delete", () => projectDelete(p.id));
          }}
        />
      )}
      <ErrorLine error={error} />
    </div>
  );
}

/** The port of a web action: automatic (kept between starts) or chosen. */
function PortField({ actionKey, ports, onPorts }: { actionKey: string; ports: Ports; onPorts: (p: Ports) => void }) {
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
function SiteForm({ project, onChanged }: { project: ProjectInfo; onChanged: () => void }) {
  const site = project.actions.find((a) => a.id === "site");
  const [folder, setFolder] = useState(site?.static ?? ".");
  const [kind, setKind] = useState<SiteKind>(site ? (site.static !== null ? "static" : "php") : "static");
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
          <option value="php">PHP</option>
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
      <ErrorLine error={error} />
    </form>
  );
}

export function RunsPanel({
  runs,
  logs,
  shown,
  onShow,
  onStop,
  onForget,
}: {
  runs: RunInfo[];
  logs: Record<string, string[]>;
  shown: string | null;
  onShow: (key: string) => void;
  onStop: (key: string) => void;
  onForget: (key: string) => void;
}) {
  const { error, run } = useAction();
  const current = runs.find((r) => r.key === shown) ?? null;
  return (
    <section aria-labelledby="runs" className="border-t border-vsc-border">
      <h2 id="runs" className="px-3 py-2 text-[13px] font-semibold">
        {t("Avviati")}
      </h2>
      <ul className="divide-y divide-vsc-border">
        {runs.map((r) => (
          <li key={r.key} className="flex flex-wrap items-center gap-2 px-2 py-1">
            <span className="min-w-0 flex-1 truncate">
              {r.project}: {r.label}
            </span>
            <span
              className={r.state === "error" ? "text-vsc-error" : alive(r.state) ? "text-vsc-success" : "text-vsc-desc"}
            >
              {stateText(r)}
            </span>
            {r.url && r.state === "running" && (
              <ActionButton
                icon={VscGlobe}
                label={r.url.replace(/^http:\/\//, "").replace(/\/$/, "")}
                title={t("Apre nel browser")}
                onClick={() => {
                  run("open", () => urlOpen(r.url ?? ""));
                }}
              />
            )}
            <ActionButton
              icon={VscOutput}
              label={t("Output")}
              onClick={() => {
                onShow(r.key);
              }}
            />
            {alive(r.state) ? (
              <ActionButton
                icon={VscDebugStop}
                label={t("Ferma")}
                onClick={() => {
                  onStop(r.key);
                }}
              />
            ) : (
              <ActionButton
                icon={VscTrash}
                label={t("Togli")}
                onClick={() => {
                  onForget(r.key);
                }}
              />
            )}
          </li>
        ))}
      </ul>
      <ErrorLine error={error} />
      {current && <LogView title={`${current.project}: ${current.label}`} lines={logs[current.key] ?? []} />}
    </section>
  );
}

function LogView({ title, lines }: { title: string; lines: string[] }) {
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
