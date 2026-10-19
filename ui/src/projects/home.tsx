// The home of Projects, the editor with no project open: what runs, the
// favorite and recent projects with their buttons, what needs a hand, the
// ways to make a new one. Each project starts, stops and opens from here
// without its page.
import type { ReactNode } from "react";
import {
  VscBook,
  VscCode,
  VscDebugStop,
  VscError,
  VscFolderOpened,
  VscGlobe,
  VscNewFolder,
  VscOutput,
  VscRepoClone,
  VscStarEmpty,
  VscStarFull,
  VscTerminal,
  VscWand,
  VscWarning,
} from "react-icons/vsc";
import { urlOpen, type AgentInfo, type EditorStatus, type ProjectAction, type ProjectInfo, type RunInfo } from "../api";
import { AgentButton } from "../agents";
import { basesByCategory, PieceLogo, type Frameworks } from "../frameworks";
import { t } from "../i18n";
import { ActionBar, Button, Dot, EditorBody, EditorSection, IconButton, Link } from "../workbench";
import { KIND_ICON, LanguageIcons, ProjectState } from "./kind";
import { COMPONENT_NAMES } from "./needs";
import { wayLabel, type NewWay } from "./new";
import type { ProjectSection } from "./page";
import { alive } from "./runs";

/** Recent projects shown besides the favorites. */
const RECENT = 6;

/** The action «Avvia» starts: the default one, else the first. */
export function mainAction(p: ProjectInfo): ProjectAction | null {
  return p.actions.find((a) => a.id === p.default) ?? p.actions[0] ?? null;
}

/** When the project was last opened or changed (seconds since 1970). */
function recency(p: ProjectInfo): number {
  return Math.max(p.modified, p.settings.lastOpened ?? 0);
}

export interface HomeHandlers {
  /** Opens the project's page, on a section or an action. */
  onOpen: (id: string, at?: { section?: ProjectSection; action?: string }) => void;
  onStart: (p: ProjectInfo, a: ProjectAction) => void;
  onStop: (key: string) => void;
  onEditor: (p: ProjectInfo) => void;
  onTerminal: (p: ProjectInfo) => void;
  onPin: (p: ProjectInfo, pinned: boolean) => void;
  onNew: (way: NewWay, base?: string) => void;
  onGithub: () => void;
  onFolder: () => void;
  onGuide?: (() => void) | undefined;
}

export function ProjectsHome({
  projects,
  runs,
  busy,
  status,
  agents,
  fw,
  handlers,
}: {
  projects: ProjectInfo[];
  runs: Record<string, RunInfo>;
  busy: string | null;
  status: EditorStatus | null;
  agents: AgentInfo[] | null;
  fw: Frameworks;
  handlers: HomeHandlers;
}) {
  const byId = new Map(projects.map((p) => [p.id, p]));
  const live = Object.values(runs)
    .filter((r) => alive(r.state) && byId.has(r.project))
    .sort((a, b) => a.start - b.start);
  const failed = Object.values(runs).filter((r) => r.state === "error" && byId.has(r.project));
  const missing = projects.filter((p) => p.missing.length > 0);
  const pinned = projects.filter((p) => p.settings.pinned).sort((a, b) => a.name.localeCompare(b.name));
  const recent = projects
    .filter((p) => !p.settings.pinned)
    .sort((a, b) => recency(b) - recency(a))
    .slice(0, RECENT);
  const groups = basesByCategory(fw.pieces);
  const card = (p: ProjectInfo) => (
    <ProjectCard key={p.id} p={p} runs={runs} busy={busy} status={status} agents={agents} handlers={handlers} />
  );

  return (
    <EditorBody>
      <div className="max-w-5xl space-y-6">
        {live.length > 0 && (
          <EditorSection id="home-running" title={t("In esecuzione")}>
            <ul aria-labelledby="home-running" className="m-0 list-none space-y-1 p-0">
              {live.map((r) => (
                <RunningRow
                  key={r.key}
                  run={r}
                  project={byId.get(r.project)?.name ?? r.project}
                  busy={busy}
                  handlers={handlers}
                />
              ))}
            </ul>
          </EditorSection>
        )}

        {pinned.length > 0 && (
          <EditorSection id="home-pinned" title={t("Preferiti")}>
            <Cards>{pinned.map(card)}</Cards>
          </EditorSection>
        )}

        {recent.length > 0 && (
          <EditorSection id="home-recent" title={t("Recenti")}>
            <Cards>{recent.map(card)}</Cards>
          </EditorSection>
        )}

        {(failed.length > 0 || missing.length > 0) && (
          <EditorSection id="home-fix" title={t("Da sistemare")}>
            <ul className="m-0 list-none space-y-1 p-0">
              {failed.map((r) => (
                <li key={r.key} className="flex flex-wrap items-center gap-2">
                  <VscError size={14} aria-hidden="true" className="shrink-0 text-vsc-error" />
                  <span>
                    {t("{0}: «{1}» si è fermato con un errore", byId.get(r.project)?.name ?? r.project, r.label)}
                    {r.code !== null && ` (${t("codice {0}", r.code)})`}
                  </span>
                  <Link
                    onClick={() => {
                      handlers.onOpen(r.project, { action: r.action });
                    }}
                  >
                    {t("Vedi l'output")}
                  </Link>
                </li>
              ))}
              {missing.map((p) => (
                <li key={p.id} className="flex flex-wrap items-center gap-2">
                  <VscWarning size={14} aria-hidden="true" className="shrink-0 text-vsc-warning" />
                  <span>{t("{0}: mancano {1}", p.name, p.missing.map((m) => COMPONENT_NAMES[m] ?? m).join(", "))}</span>
                  <Link
                    onClick={() => {
                      handlers.onOpen(p.id, { section: "components" });
                    }}
                  >
                    {t("Installa")}
                  </Link>
                </li>
              ))}
            </ul>
          </EditorSection>
        )}

        <EditorSection id="home-new" title={t("Nuovo progetto")}>
          <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2">
            <WayTile
              icon={<VscNewFolder size={20} aria-hidden="true" />}
              title={wayLabel("empty")}
              text={t("Una cartella nuova e vuota in projects.")}
              onClick={() => {
                handlers.onNew("empty");
              }}
            />
            <WayTile
              icon={<VscRepoClone size={20} aria-hidden="true" />}
              title={wayLabel("git")}
              text={t("Il repository si clona in una cartella di projects.")}
              onClick={() => {
                handlers.onNew("git");
              }}
            />
            <WayTile
              icon={<VscWand size={20} aria-hidden="true" />}
              title={wayLabel("framework")}
              text={t("Una base e le sue aggiunte, con i comandi ufficiali.")}
              onClick={() => {
                handlers.onNew("framework");
              }}
            >
              {groups.length > 0 && (
                <span className="flex flex-wrap gap-1 pt-1">
                  {/* A group opens the wizard on its first base. */}
                  {groups.map((g) => (
                    <button
                      key={g.id}
                      type="button"
                      title={g.bases.map((b) => b.name || b.id).join(", ")}
                      onClick={() => {
                        handlers.onNew("framework", g.bases[0]?.id);
                      }}
                      className="flex items-center gap-1 rounded-sm border border-vsc-border px-1.5 text-[12px] leading-5 hover:bg-vsc-list-hover"
                    >
                      {g.bases[0] && <PieceLogo piece={g.bases[0]} size={12} />}
                      {g.label}
                    </button>
                  ))}
                </span>
              )}
            </WayTile>
          </div>
          <ul className="m-0 flex list-none flex-wrap gap-x-5 gap-y-1 p-0 pt-1">
            <li className="flex items-center gap-1.5">
              <VscRepoClone size={14} aria-hidden="true" className="text-vsc-link" />
              <Link onClick={handlers.onGithub}>{t("Clona dai tuoi repository GitHub…")}</Link>
            </li>
            <li className="flex items-center gap-1.5">
              <VscFolderOpened size={14} aria-hidden="true" className="text-vsc-link" />
              <Link onClick={handlers.onFolder}>{t("Apri la cartella dei progetti")}</Link>
            </li>
            {handlers.onGuide && (
              <li className="flex items-center gap-1.5">
                <VscBook size={14} aria-hidden="true" className="text-vsc-link" />
                <Link onClick={handlers.onGuide}>{t("Guida")}</Link>
              </li>
            )}
          </ul>
        </EditorSection>

        <p className="text-[12px] text-vsc-desc">
          {t(
            "Le cartelle in projects sono attendibili per Claude e Codex e i loro comandi partono dai pulsanti dei progetti: apri qui solo codice di cui ti fidi.",
          )}{" "}
          {t("Ctrl+P apre un progetto per nome.")}
        </p>
      </div>
    </EditorBody>
  );
}

function Cards({ children }: { children: ReactNode }) {
  return <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-2 p-0">{children}</ul>;
}

/** A project as a card: what it is, its state, its buttons. */
function ProjectCard({
  p,
  runs,
  busy,
  status,
  agents,
  handlers,
}: {
  p: ProjectInfo;
  runs: Record<string, RunInfo>;
  busy: string | null;
  status: EditorStatus | null;
  agents: AgentInfo[] | null;
  handlers: HomeHandlers;
}) {
  const Icon = KIND_ICON[p.type];
  const main = mainAction(p);
  const key = main ? `${p.id}|${main.id}` : "";
  const running = main !== null && alive(runs[key]?.state);
  const pinned = p.settings.pinned === true;
  return (
    <li className="flex flex-col gap-2 rounded-md border border-vsc-border p-3">
      <div className="flex min-w-0 items-center gap-2">
        <Icon size={18} aria-hidden="true" className="shrink-0 text-vsc-icon" />
        <button
          type="button"
          title={t("Apre la pagina di {0}", p.name)}
          onClick={() => {
            handlers.onOpen(p.id);
          }}
          className="min-w-0 flex-1 truncate text-left font-semibold hover:underline"
        >
          {p.name}
        </button>
        <ProjectState p={p} runs={runs} />
        <IconButton
          icon={pinned ? VscStarFull : VscStarEmpty}
          label={pinned ? t("Togli {0} dai preferiti", p.name) : t("Aggiungi {0} ai preferiti", p.name)}
          checked={pinned}
          onClick={() => {
            handlers.onPin(p, !pinned);
          }}
        />
      </div>
      <div className="flex min-w-0 items-center gap-2 text-[12px] text-vsc-desc">
        <span className="truncate">{p.platform}</span>
        {p.languages.length > 0 && <LanguageIcons names={p.languages} />}
      </div>
      <div className="flex flex-wrap items-center gap-1.5">
        {main &&
          (running ? (
            <Button
              small
              busy={busy === `stop:${key}`}
              onClick={() => {
                handlers.onStop(key);
              }}
            >
              {t("Ferma")}
            </Button>
          ) : (
            <Button
              small
              variant="primary"
              title={main.label}
              busy={busy === `run:${key}`}
              onClick={() => {
                handlers.onStart(p, main);
              }}
            >
              {t("Avvia")}
            </Button>
          ))}
        <ActionBar>
          <IconButton
            icon={VscCode}
            label={t("Editor")}
            title={t("Apre {0} nell'editor", p.name)}
            busy={busy === `editor:${p.id}`}
            disabled={status?.installed === false}
            onClick={() => {
              handlers.onEditor(p);
            }}
          />
          <IconButton
            icon={VscTerminal}
            label={t("Terminale")}
            title={t("Console del kit in {0}", p.name)}
            busy={busy === `terminal:${p.id}`}
            onClick={() => {
              handlers.onTerminal(p);
            }}
          />
          <AgentButton agents={agents} project={p.id} preferred={p.settings.agent} />
        </ActionBar>
      </div>
    </li>
  );
}

/** An action that runs: where it listens, its output, the stop. */
function RunningRow({
  run,
  project,
  busy,
  handlers,
}: {
  run: RunInfo;
  project: string;
  busy: string | null;
  handlers: HomeHandlers;
}) {
  return (
    <li className="flex min-h-7 items-center gap-2 rounded px-2 hover:bg-vsc-list-hover">
      <Dot tone="ok" label={run.state === "running" ? t("avviato") : t("in avvio")} />
      <span className="min-w-0 truncate">
        <span className="font-semibold">{project}</span>
        <span className="text-vsc-desc"> · {run.label}</span>
      </span>
      {run.url && run.state === "running" && (
        <span className="truncate font-mono text-[12px] text-vsc-desc">{run.url}</span>
      )}
      <span className="flex-1" />
      <ActionBar>
        {run.url && run.state === "running" && (
          <IconButton
            icon={VscGlobe}
            label={t("Apri nel browser")}
            onClick={() => {
              void urlOpen(run.url ?? "");
            }}
          />
        )}
        <IconButton
          icon={VscOutput}
          label={t("Vedi l'output")}
          onClick={() => {
            handlers.onOpen(run.project, { action: run.action });
          }}
        />
        <IconButton
          icon={VscDebugStop}
          label={t("Ferma")}
          busy={busy === `stop:${run.key}`}
          onClick={() => {
            handlers.onStop(run.key);
          }}
        />
      </ActionBar>
    </li>
  );
}

/** A way of making a project, as a card that starts it. */
function WayTile({
  icon,
  title,
  text,
  onClick,
  children,
}: {
  icon: ReactNode;
  title: string;
  text: string;
  onClick: () => void;
  children?: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1 rounded-md border border-vsc-border p-3">
      <button
        type="button"
        onClick={onClick}
        className="flex items-center gap-2 text-left font-semibold text-vsc-link hover:underline"
      >
        <span className="text-vsc-icon">{icon}</span>
        {title}
      </button>
      <span className="text-[12px] text-vsc-desc">{text}</span>
      {children}
    </div>
  );
}
