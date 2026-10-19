// Projects page, as VSCodium's Explorer: the side bar lists the projects in
// projects\ (detected by tauri/src/detect, corrected by panel.json) with
// their inline actions, and the ways to make a new one; the editor opens
// each project in its own tab (page.tsx), and the new-project wizard in one
// of its own (wizard.tsx, D-83). Its first tab, always there, is the home
// (home.tsx): what runs, favorites and recent projects, what needs a hand.
// Ctrl+P finds a project by name (quickopen.tsx). Web actions get their own
// port and open in the browser.
import { useCallback, useEffect, useState } from "react";
import {
  VscBook,
  VscCode,
  VscDebugStart,
  VscDebugStop,
  VscFolderOpened,
  VscGlobe,
  VscHome,
  VscNewFolder,
  VscRefresh,
  VscSearch,
  VscTerminal,
  VscWand,
} from "react-icons/vsc";
import {
  editorOpen,
  folderOpen,
  projectPin,
  projectsList,
  runPorts,
  runStart,
  runStop,
  terminalOpen,
  type AgentInfo,
  type EditorStatus,
  type Ports,
  type ProjectAction,
  type ProjectInfo,
  type RunInfo,
} from "../api";
import { AgentButton } from "../agents";
import { useFrameworks } from "../frameworks";
import { GithubRepositories } from "../github";
import type { GuideTopicId } from "../guide";
import { useAction, useLoad } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import {
  Button,
  EditorTabs,
  IconButton,
  List,
  ListRow,
  PaneStack,
  SearchBox,
  SideBarTitle,
  ViewContainer,
  ViewPane,
  type useTabs,
} from "../workbench";
import { mainAction, ProjectsHome, type HomeHandlers } from "./home";
import { ProjectPage, type ProjectSection } from "./page";
import { QuickOpen } from "./quickopen";
import { alive } from "./runs";
import { KIND_ICON, ProjectState } from "./kind";
import { EmptyProjectEditor, GitProjectEditor, NewProjectTree, wayLabel, type NewWay } from "./new";
import { Orphans } from "./orphans";
import { NewProjectWizard, WIZARD_TAB } from "./wizard";

type PaneId = "projects" | "new" | "github";

/** The home's tab: never in `tabs.open`, shown while no other is active. */
const HOME_TAB = "\u0000home";

/** Stable while the list is loading (Orphans reads again when it changes). */
const NO_PROJECTS: ProjectInfo[] = [];

export function ProjectsPage({
  status,
  agents,
  onChanged,
  runs,
  logs,
  loadLog,
  forget,
  tabs,
  onGuide,
  onSettings,
  quickOpen = false,
  onQuickOpen,
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
  onGuide?: (topic: GuideTopicId) => void;
  /** Opens Settings (the wizard's instructions). */
  onSettings?: (() => void) | undefined;
  /** The Quick Open box is open (Ctrl+P, kept by the app). */
  quickOpen?: boolean;
  onQuickOpen?: ((open: boolean) => void) | undefined;
}) {
  const [panes, setPanes] = useState<Set<PaneId>>(() => new Set(["projects"]));
  const [filter, setFilter] = useState("");
  /** Raised to focus the new project's name (NewProject). */
  const [focusNew, setFocusNew] = useState(0);
  /** The procedure in the «Nuovo progetto» tab; the wizard's base. */
  const [way, setWay] = useState<NewWay>("framework");
  const [base, setBase] = useState("");
  const [preset, setPreset] = useState<{ base: string; at: number } | null>(null);
  /** The wizard runs its commands: its tab stays open. */
  const [wizardRunning, setWizardRunning] = useState(false);
  const fw = useFrameworks();
  const { load: loadFrameworks } = fw;
  useEffect(loadFrameworks, [loadFrameworks]);
  const { busy, error, run } = useAction(onChanged);
  const { keep } = tabs;
  /** Where the next project page opens (the home's links). */
  const [jump, setJump] = useState<{ id: string; section?: ProjectSection; action?: string } | null>(null);

  const readProjects = useCallback(
    () =>
      projectsList().then((list) => {
        keep([...list.map((p) => p.id), WIZARD_TAB]);
        return list;
      }),
    [keep],
  );
  const {
    data: projects,
    error: loadError,
    reload: reloadProjects,
  } = useLoad<ProjectInfo[] | null>(readProjects, null, { onError: "show" });
  const {
    data: ports,
    setData: setPorts,
    reload: reloadPorts,
  } = useLoad<Ports>(runPorts, { ports: {}, locked: [] }, { onError: "keep" });
  const load = useCallback(() => {
    reloadProjects();
    reloadPorts();
  }, [reloadProjects, reloadPorts]);

  const toggle = (id: PaneId, open?: boolean) => {
    setPanes((current) => {
      const next = new Set(current);
      if (open ?? !next.has(id)) next.add(id);
      else next.delete(id);
      return next;
    });
  };
  /** Opens a procedure in the «Nuovo progetto» tab, the tree in view. */
  const openNew = (next: NewWay, pick?: string) => {
    toggle("new", true);
    setWay(next);
    if (next === "empty") setFocusNew((n) => n + 1);
    if (pick) setPreset({ base: pick, at: Date.now() });
    tabs.show(WIZARD_TAB);
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
      reloadPorts();
    });
  };
  const stop = (key: string) => {
    run(`stop:${key}`, () => runStop(key));
  };
  const openProject = (id: string, at?: { section?: ProjectSection; action?: string }) => {
    setJump(at ? { id, ...at } : null);
    if (at?.action) loadLog(`${id}|${at.action}`);
    tabs.show(id);
  };
  const selectTab = (id: string) => {
    setJump(null);
    tabs.setActive(id === HOME_TAB ? null : id);
  };
  const home: HomeHandlers = {
    onOpen: openProject,
    onStart: start,
    onStop: stop,
    onEditor: (p) => {
      run(`editor:${p.id}`, () => editorOpen(p.id));
    },
    onTerminal: (p) => {
      run(`terminal:${p.id}`, () => terminalOpen(p.id));
    },
    onPin: (p, pinned) => {
      run(`pin:${p.id}`, async () => {
        await projectPin(p.id, pinned);
        reloadProjects();
      });
    },
    onNew: openNew,
    onGithub: () => {
      toggle("github", true);
    },
    onFolder: () => {
      run("folder", () => folderOpen("projects"));
    },
    onGuide: onGuide
      ? () => {
          onGuide("project-page");
        }
      : undefined,
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
                openNew("empty");
              }}
            />
            <IconButton
              icon={VscWand}
              label={t("Nuovo progetto da framework o CMS")}
              onClick={() => {
                openNew("framework");
              }}
            />
            <IconButton
              icon={VscSearch}
              label={t("Apri un progetto (Ctrl+P)")}
              onClick={() => {
                onQuickOpen?.(true);
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
            {onGuide && (
              <IconButton
                icon={VscBook}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("project-page");
                }}
              />
            )}
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
                  openNew("empty");
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
                      openProject(p.id);
                    }}
                    actions={
                      <>
                        <StartStop p={p} runs={runs} busy={busy} onStart={start} onStop={stop} />
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
        <Orphans projects={projects ?? NO_PROJECTS} onChanged={load} />
        <ViewPane
          title={t("Nuovo progetto")}
          expanded={panes.has("new")}
          onToggle={() => {
            toggle("new");
          }}
        >
          <NewProjectTree fw={fw} way={tabs.active === WIZARD_TAB ? way : null} base={base} onSelect={openNew} />
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
      <EditorTabs
        tabs={[
          { id: HOME_TAB, label: t("Pannello rapido"), icon: <VscHome size={16} aria-hidden="true" />, pinned: true },
          ...tabs.open.map((id) => {
            if (id === WIZARD_TAB) {
              return {
                id,
                label: t("Nuovo progetto"),
                description: wayLabel(way),
                icon: <VscWand size={16} aria-hidden="true" />,
                pinned: wizardRunning,
              };
            }
            const p = byId(id);
            const Icon = p ? KIND_ICON[p.type] : VscGlobe;
            return { id, label: p?.name ?? id, icon: <Icon size={16} aria-hidden="true" /> };
          }),
        ]}
        active={tabs.active ?? HOME_TAB}
        onSelect={selectTab}
        onClose={tabs.close}
      />
      {/* Kept while another tab is shown: its choices and the commands'
          output stay. */}
      {tabs.open.includes(WIZARD_TAB) && (
        <div className={tabs.active === WIZARD_TAB && way === "framework" ? "flex min-h-0 flex-1 flex-col" : "hidden"}>
          <NewProjectWizard
            fw={fw}
            preset={preset}
            onBase={setBase}
            onOpen={tabs.show}
            onCreated={load}
            onRunning={setWizardRunning}
            onSettings={onSettings}
            onGuide={onGuide}
          />
        </div>
      )}
      {tabs.active === WIZARD_TAB ? (
        way === "empty" ? (
          <EmptyProjectEditor onDone={created} focus={focusNew} />
        ) : way === "git" ? (
          <GitProjectEditor onDone={created} />
        ) : null
      ) : page ? (
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
          onStop={stop}
          onPorts={setPorts}
          onShowLog={loadLog}
          onForget={forget}
          onChanged={load}
          initial={jump?.id === page.id ? jump : undefined}
        />
      ) : projects === null ? (
        <p className="p-5 text-vsc-desc">{t("Caricamento…")}</p>
      ) : (
        <ProjectsHome
          projects={projects}
          runs={runs}
          busy={busy}
          status={status}
          agents={agents}
          fw={fw}
          handlers={home}
        />
      )}
      {quickOpen && (
        <QuickOpen
          projects={projects ?? NO_PROJECTS}
          runs={runs}
          onOpen={openProject}
          onStart={(p) => {
            const a = mainAction(p);
            if (a) start(p, a);
          }}
          onNew={openNew}
          onClose={() => {
            onQuickOpen?.(false);
          }}
        />
      )}
    </ViewContainer>
  );
}

/** The row's ▶ (the action «Avvia» starts), or ■ while it runs. */
function StartStop({
  p,
  runs,
  busy,
  onStart,
  onStop,
}: {
  p: ProjectInfo;
  runs: Record<string, RunInfo>;
  busy: string | null;
  onStart: (p: ProjectInfo, a: ProjectAction) => void;
  onStop: (key: string) => void;
}) {
  const main = mainAction(p);
  if (!main) return null;
  const key = `${p.id}|${main.id}`;
  return alive(runs[key]?.state) ? (
    <IconButton
      icon={VscDebugStop}
      label={t("Ferma")}
      title={t("Ferma {0}", main.label)}
      busy={busy === `stop:${key}`}
      onClick={() => {
        onStop(key);
      }}
    />
  ) : (
    <IconButton
      icon={VscDebugStart}
      label={t("Avvia")}
      title={t("Avvia {0}", main.label)}
      busy={busy === `run:${key}`}
      onClick={() => {
        onStart(p, main);
      }}
    />
  );
}
