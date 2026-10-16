// Projects page, as VSCodium's Explorer: the side bar lists the projects in
// projects\ (detected by tauri/src/detect, corrected by panel.json) with
// their inline actions, and the ways to make a new one; the editor opens
// each project in its own tab (page.tsx). Web actions get their own
// port and open in the browser.
import { useCallback, useState } from "react";
import {
  VscBook,
  VscCode,
  VscFolderOpened,
  VscGlobe,
  VscNewFolder,
  VscRefresh,
  VscRepoClone,
  VscTerminal,
} from "react-icons/vsc";
import {
  editorOpen,
  folderOpen,
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
import { GithubRepositories } from "../github";
import type { GuideTopicId } from "../guide";
import { useAction, useLoad } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { BrandSymbol } from "../brand";
import {
  Button,
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
} from "../workbench";
import { ProjectPage } from "./page";
import { KIND_ICON, ProjectState } from "./kind";
import { CloneProject, NewProject } from "./new";
import { Orphans } from "./orphans";

type PaneId = "projects" | "new" | "github";

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
}) {
  const [panes, setPanes] = useState<Set<PaneId>>(() => new Set(["projects"]));
  const [filter, setFilter] = useState("");
  /** Raised to focus the new project's name (NewProject). */
  const [focusNew, setFocusNew] = useState(0);
  const { busy, error, run } = useAction(onChanged);
  const { keep } = tabs;

  const readProjects = useCallback(
    () =>
      projectsList().then((list) => {
        keep(list.map((p) => p.id));
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
    // A collapsed pane drops its content: reopened from its header, the
    // name must not take the focus again.
    if (id === "new" && open !== true) setFocusNew(0);
  };
  const openNew = () => {
    toggle("new", true);
    setFocusNew((n) => n + 1);
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
                openNew();
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
                  onGuide("feat-projects");
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
                  openNew();
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
        <Orphans projects={projects ?? NO_PROJECTS} onChanged={load} />
        <ViewPane
          title={t("Nuovo progetto")}
          expanded={panes.has("new")}
          onToggle={() => {
            toggle("new");
          }}
        >
          <div className="space-y-4 px-5 pt-1 pb-4">
            <NewProject onDone={created} focus={focusNew} />
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
                      openNew();
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
                {onGuide && (
                  <li className="flex items-center gap-2">
                    <VscBook size={16} aria-hidden="true" className="text-vsc-link" />
                    <Link
                      onClick={() => {
                        onGuide("overview");
                      }}
                    >
                      {t("Guida")}
                    </Link>
                  </li>
                )}
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
