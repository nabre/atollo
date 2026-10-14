// The page of one project (user request, 14/10/2026, D-75): every setting
// and choice of the project in tabs. Starting it, its databases, its editor
// extensions, its agent and its Git state; the kit's settings of the
// project live in data\app\projects\<id>\dev-settings.json, never in it.
import { useCallback, useEffect, useState } from "react";
import type { IconType } from "react-icons";
import {
  VscArrowLeft,
  VscCloudDownload,
  VscCode,
  VscDebugStart,
  VscDebugStop,
  VscExtensions,
  VscGlobe,
  VscDatabase,
  VscFolderOpened,
  VscGithub,
  VscHubot,
  VscPackage,
  VscRefresh,
  VscSettingsGear,
  VscSourceControl,
  VscTerminal,
  VscWarning,
} from "react-icons/vsc";
import {
  editorOpen,
  folderOpen,
  projectDatabases,
  projectDbCreate,
  projectEnvApply,
  projectEnvPreview,
  projectInstallMissing,
  projectStatus,
  terminalOpen,
  urlOpen,
  type AgentInfo,
  type EditorStatus,
  type EnvChange,
  type Ports,
  type ProjectAction,
  type ProjectDatabases,
  type ProjectDb,
  type ProjectInfo,
  type ProjectStatus,
  type RunInfo,
} from "./api";
import { AgentPicker } from "./agents";
import { SecretField } from "./databases";
import { ProjectExtensions } from "./extensions";
import { EditorSettingsBox } from "./editorsettings";
import { DatabaseCopies } from "./backups";
import { ProjectMcpBox } from "./projectmcp";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";
import {
  alive,
  LanguageIcons,
  ProjectDetails,
  ProjectKind,
  RunsPanel,
  SidePane,
  stateText,
  ToolbarIcon,
} from "./projects";

type Section = "run" | "components" | "databases" | "extensions" | "editor" | "git" | "agents";

/** The sections of the page, listed in the side bar as the editor's views. */
function sections(): { id: Section; label: string; icon: IconType }[] {
  return [
    { id: "run", label: t("Avvio"), icon: VscDebugStart },
    { id: "components", label: t("Componenti"), icon: VscPackage },
    { id: "databases", label: t("Database"), icon: VscDatabase },
    { id: "extensions", label: t("Estensioni"), icon: VscExtensions },
    { id: "editor", label: t("Editor"), icon: VscSettingsGear },
    { id: "git", label: t("Git"), icon: VscSourceControl },
    { id: "agents", label: t("Agenti"), icon: VscHubot },
  ];
}

function Box({
  id,
  title,
  children,
  right,
}: {
  id: string;
  title: string;
  children: React.ReactNode;
  right?: React.ReactNode;
}) {
  return (
    <section className="min-w-0 border-t border-vsc-border" aria-labelledby={id}>
      <h2
        id={id}
        className="flex items-center gap-2 px-3 py-1.5 text-[12px] font-semibold tracking-wide text-vsc-desc uppercase"
      >
        {title}
        {right && <span className="ml-auto text-[12px] font-normal tracking-normal normal-case">{right}</span>}
      </h2>
      {children}
    </section>
  );
}

/** Names of the kit's components (src-tauri/catalog/components.json). */
const COMPONENT_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex",
  antigravity: "Antigravity",
  git: "Git",
  python: "Python",
  node: "Node.js",
  uv: "uv",
  gh: "GitHub CLI",
  php: "PHP",
  apache: "Apache",
  rust: "Rust",
  go: "Go",
  java: "Java",
  kotlin: "Kotlin",
  dotnet: ".NET",
  dart: "Dart",
  zig: "Zig",
  julia: "Julia",
  bun: "Bun",
  deno: "Deno",
  typst: "Typst",
  tinytex: "TinyTeX",
  pnpm: "pnpm, yarn",
  cmake: "CMake, Ninja",
  cc: "C, C++",
  ruby: "Ruby",
  r: "R",
  mariadb: "MariaDB",
  mongodb: "MongoDB",
  sqlite: "SQLite",
  pwsh: "PowerShell 7",
  terminal: "Windows Terminal",
  "vscodium-server": "Editor",
};

/** The components the project needs, with "Installa ciò che manca". */
function Needs({ p, onChanged }: { p: ProjectInfo; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  return (
    <Box
      id="need"
      title={t("Componenti")}
      right={
        p.missing.length > 0 ? (
          <span className="text-vsc-warning">{t("Mancano: {0}", String(p.missing.length))}</span>
        ) : undefined
      }
    >
      <ul className="divide-y divide-vsc-border">
        {p.requires.length === 0 && <li className="px-3 py-1.5 text-vsc-desc">{t("Nessun componente del kit.")}</li>}
        {p.requires.map((id) => {
          const missing = p.missing.includes(id);
          return (
            <li key={id} className="flex items-center gap-2 px-3 py-1.5">
              <ComponentLogo id={id} />
              <span className="min-w-0 flex-1">{COMPONENT_NAMES[id] ?? id}</span>
              <span className={missing ? "text-vsc-warning" : "text-vsc-success"}>
                {missing ? t("da installare") : t("installato")}
              </span>
            </li>
          );
        })}
      </ul>
      {p.missing.length > 0 && (
        <div className="px-4 pb-3">
          <ActionButton
            icon={VscCloudDownload}
            label={t("Installa ciò che manca")}
            primary
            busy={busy === "install"}
            onClick={() => {
              run("install", () => projectInstallMissing(p.id));
            }}
          />
        </div>
      )}
      <p className="px-4 pb-3 text-[12px] text-vsc-desc">{p.reason}</p>
      {p.warnings.length > 0 && (
        <ul className="list-disc space-y-1 pr-4 pb-3 pl-9 text-[12px] text-vsc-desc">
          {p.warnings.map((w) => (
            <li key={w}>{w}</li>
          ))}
        </ul>
      )}
      <ErrorLine error={error} />
    </Box>
  );
}

/** The default action with its output: what is done most in a project. */
function MainAction({
  p,
  runs,
  logs,
  busy,
  onStart,
  onStop,
}: {
  p: ProjectInfo;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  busy: string | null;
  onStart: (a: ProjectAction) => void;
  onStop: (key: string) => void;
}) {
  const main = p.actions.find((a) => a.id === p.default) ?? p.actions[0] ?? null;
  const key = main ? `${p.id}|${main.id}` : "";
  const r = main ? runs[key] : undefined;
  const lines = r ? (logs[key] ?? []) : [];
  return (
    <section aria-label={t("Avvio")}>
      <div className="flex flex-wrap items-center gap-2 px-3 py-2">
        {main ? (
          <>
            <span className="min-w-0 flex-1">
              <span className="font-semibold">{main.label}</span>
              <span className="ml-2 text-vsc-desc">
                {r ? stateText(r) : t("fermo")}
                {r?.url && r.state === "running" && ` · ${r.url}`}
              </span>
            </span>
            {r?.url && r.state === "running" && (
              <ActionButton
                icon={VscGlobe}
                label={t("Apri nel browser")}
                onClick={() => {
                  void urlOpen(r.url ?? "");
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
                primary
                busy={busy === `run:${key}`}
                onClick={() => {
                  onStart(main);
                }}
              />
            )}
          </>
        ) : (
          <span className="text-vsc-desc">
            {t("Nessuna azione riconosciuta: aggiungine in panel.json o servi una cartella come sito web.")}
          </span>
        )}
      </div>
      {lines.length > 0 && (
        <pre
          tabIndex={0}
          aria-label={t("Output di {0}", main?.label ?? "")}
          className="m-0 max-h-40 overflow-auto border-t border-vsc-border bg-vsc-bg px-3 py-1.5 font-mono text-[12px] leading-snug whitespace-pre-wrap select-text"
        >
          {lines.slice(-200).join("\n")}
        </pre>
      )}
    </section>
  );
}

/** The project at a glance, in the side bar: as the columns of the list,
 * then agent, Git and databases. A value that needs work opens its section. */
function Overview({
  p,
  status,
  dbs,
  live,
  onSection,
}: {
  p: ProjectInfo;
  status: ProjectStatus | null;
  dbs: ProjectDatabases | null;
  live: RunInfo[];
  onSection: (s: Section) => void;
}) {
  const row = "contents";
  return (
    <dl className="grid grid-cols-[max-content_minmax(0,1fr)] items-center gap-x-3 gap-y-1">
      <div className={row}>
        <dt className="text-vsc-desc">{t("Tipo")}</dt>
        <dd>
          <ProjectKind p={p} />
        </dd>
      </div>
      <div className={row}>
        <dt className="text-vsc-desc">{t("Linguaggi")}</dt>
        <dd>
          {p.languages.length === 0 ? <span className="text-vsc-desc">—</span> : <LanguageIcons names={p.languages} />}
        </dd>
      </div>
      <div className={row}>
        <dt className="text-vsc-desc">{t("Stato")}</dt>
        <dd className="truncate">
          {live.length > 0 ? (
            <span className="text-vsc-success" title={live.map((r) => r.label).join(", ")}>
              ● {live.some((r) => r.type === "web") ? t("in ascolto") : t("in esecuzione")}
            </span>
          ) : p.missing.length > 0 ? (
            <button
              type="button"
              className="flex max-w-full items-center gap-1 text-vsc-warning hover:underline"
              onClick={() => {
                onSection("components");
              }}
            >
              <VscWarning size={12} aria-hidden="true" className="shrink-0" />
              <span className="truncate">
                {t("Mancano: {0}", p.missing.map((m) => COMPONENT_NAMES[m] ?? m).join(", "))}
              </span>
            </button>
          ) : (
            <span className="text-vsc-desc">{t("fermo")}</span>
          )}
        </dd>
      </div>
      <div className={row}>
        <dt className="text-vsc-desc">{t("Agente")}</dt>
        <dd className="truncate">
          {p.settings.agent ? (AGENT_NAMES[p.settings.agent] ?? p.settings.agent) : t("da scegliere")}
        </dd>
      </div>
      <div className={row}>
        <dt className="text-vsc-desc">{t("Git")}</dt>
        <dd className="truncate">
          {status?.git ? (
            <>
              <span className="font-mono">{status.git.branch ?? t("nessun ramo")}</span>
              {status.git.synced === false && <span className="ml-2 text-vsc-warning">{t("non sincronizzato")}</span>}
            </>
          ) : (
            <span className="text-vsc-desc">{status ? t("non è un repository Git") : "…"}</span>
          )}
        </dd>
      </div>
      <div className={row}>
        <dt className="text-vsc-desc">{t("Database")}</dt>
        <dd className="truncate">
          {dbs === null || dbs.dbs.length === 0 ? (
            <button
              type="button"
              className="text-vsc-link hover:underline"
              onClick={() => {
                onSection("databases");
              }}
            >
              {dbs?.detected.some((d) => d !== "sqlite") ? t("da creare") : t("nessuno")}
            </button>
          ) : (
            dbs.dbs.map((d) => COMPONENT_NAMES[d.engine] ?? d.engine).join(", ")
          )}
        </dd>
      </div>
    </dl>
  );
}

const AGENT_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex",
  antigravity: "Antigravity",
};

function EnvPreview({
  project,
  db,
  envIgnored,
  onDone,
}: {
  project: string;
  db: ProjectDb;
  envIgnored: boolean;
  onDone: () => void;
}) {
  const [changes, setChanges] = useState<EnvChange[] | null>(null);
  const { busy, error, run } = useAction(onDone);
  return (
    <div className="space-y-2 px-4 pb-3">
      {changes === null ? (
        <ActionButton
          icon={VscCode}
          label={t("Scrivi in .env")}
          busy={busy === "preview"}
          onClick={() => {
            run("preview", async () => {
              setChanges(await projectEnvPreview(project, db.engine));
            });
          }}
        />
      ) : changes.length === 0 ? (
        <p className="text-vsc-success">{t("Il file .env ha già questa connessione.")}</p>
      ) : (
        <>
          <p className="text-vsc-desc">{t("Righe di .env che cambiano:")}</p>
          <div className="overflow-x-auto">
            <table className="text-[12px]">
              <tbody>
                {changes.map((c) => (
                  <tr key={c.key} className="border-t border-vsc-border">
                    <td className="py-1 pr-4 font-mono">{c.key}</td>
                    <td className="py-1 pr-4 font-mono text-vsc-desc line-through">{c.old ?? ""}</td>
                    <td className="py-1 font-mono">
                      {c.key.includes("PASSWORD") || c.key.includes("URI") || c.key.includes("URL") ? "••••" : c.new}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {!envIgnored && (
            <p role="alert" className="flex items-start gap-2 text-vsc-warning">
              <VscWarning size={16} aria-hidden="true" className="mt-0.5 shrink-0" />
              {t(".env non è in .gitignore: con la password dentro finirebbe nel repository. Aggiungilo a .gitignore.")}
            </p>
          )}
          <div className="flex gap-2">
            <ActionButton
              icon={VscCode}
              label={t("Scrivi")}
              primary
              busy={busy === "apply"}
              onClick={() => {
                run("apply", async () => {
                  await projectEnvApply(project, db.engine);
                  setChanges([]);
                });
              }}
            />
            <ActionButton
              icon={VscArrowLeft}
              label={t("Annulla")}
              onClick={() => {
                setChanges(null);
              }}
            />
          </div>
        </>
      )}
      <ErrorLine error={error} />
    </div>
  );
}

function Databases({ p, dbs, onChanged }: { p: ProjectInfo; dbs: ProjectDatabases | null; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  if (dbs === null) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  const engines: ProjectDb["engine"][] = ["mariadb", "mongodb"];
  return (
    <div className="space-y-3">
      {engines.map((engine) => {
        const db = dbs.dbs.find((d) => d.engine === engine);
        const used = dbs.detected.includes(engine);
        return (
          <Box
            key={engine}
            id={`pdb-${engine}`}
            title={COMPONENT_NAMES[engine] ?? engine}
            right={used ? <span>{t("usato dal progetto")}</span> : undefined}
          >
            {db ? (
              <>
                <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-5 gap-y-2 px-3 py-2">
                  <dt className="text-vsc-desc">{t("Database")}</dt>
                  <dd className="font-mono">{db.database}</dd>
                  <dt className="text-vsc-desc">{t("Utente")}</dt>
                  <dd className="font-mono">{db.user}</dd>
                  <dt className="text-vsc-desc">{t("Password")}</dt>
                  <dd>
                    <SecretField value={db.password} label={t("la password di {0}", db.user)} />
                  </dd>
                  <dt className="text-vsc-desc">{t("Indirizzo")}</dt>
                  <dd className="font-mono">
                    {db.host}:{String(db.port)}
                  </dd>
                  <dt className="text-vsc-desc">{t("Stringa")}</dt>
                  <dd>
                    <SecretField value={db.url} label={t("la stringa di connessione")} />
                  </dd>
                </dl>
                <EnvPreview project={p.id} db={db} envIgnored={dbs.envIgnored} onDone={onChanged} />
                <DatabaseCopies project={p.id} engine={db.engine} />
              </>
            ) : (
              <div className="flex flex-wrap items-center gap-2 px-3 py-2">
                <p className="min-w-0 flex-1 text-vsc-desc">
                  {t("Un database e un utente solo per questo progetto, con una password generata dal kit.")}
                </p>
                <ActionButton
                  icon={VscDatabase}
                  label={t("Crea database e utente")}
                  primary={used}
                  busy={busy === engine}
                  onClick={() => {
                    run(engine, async () => {
                      await projectDbCreate(p.id, engine);
                    });
                  }}
                />
              </div>
            )}
          </Box>
        );
      })}
      {dbs.detected.includes("sqlite") && (
        <Box id="pdb-sqlite" title="SQLite">
          <p className="px-3 py-2 text-vsc-desc">
            {t("Il progetto usa SQLite: il database è un file del progetto, nessun server da avviare.")}
          </p>
        </Box>
      )}
      <ErrorLine error={error} />
    </div>
  );
}

function Agents({ status }: { status: ProjectStatus | null }) {
  return (
    <Box id="instructions" title={t("Istruzioni per gli agenti")}>
      <ul className="divide-y divide-vsc-border">
        {(status?.instructions ?? []).map((f) => (
          <li key={f} className="px-3 py-1.5 font-mono">
            {f}
          </li>
        ))}
        <li className="px-3 py-1.5 text-vsc-desc">
          {t("Istruzioni, skill e MCP del kit valgono anche qui (data\\agents).")}
        </li>
      </ul>
    </Box>
  );
}

function Git({ status }: { status: ProjectStatus | null }) {
  const git = status?.git;
  if (!status) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  if (!git) return <p className="text-vsc-desc">{t("La cartella non è un repository Git.")}</p>;
  const web = git.remote?.replace(/\.git$/, "").replace(/^git@github\.com:/, "https://github.com/") ?? null;
  return (
    <Box id="repo" title={t("Repository")}>
      <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-5 gap-y-2 px-3 py-2">
        <dt className="text-vsc-desc">{t("Remoto")}</dt>
        <dd className="truncate font-mono">{git.remote ?? t("nessuno")}</dd>
        <dt className="text-vsc-desc">{t("Ramo")}</dt>
        <dd className="font-mono">{git.branch ?? t("nessun ramo")}</dd>
        <dt className="text-vsc-desc">{t("Remoto del ramo")}</dt>
        <dd className="font-mono">{git.upstream ?? t("ramo non collegato al remoto")}</dd>
        <dt className="text-vsc-desc">{t("Sincronizzato")}</dt>
        <dd>
          {git.synced === null
            ? t("non si sa")
            : git.synced
              ? t("sì, allo stesso commit (all'ultimo aggiornamento dal remoto)")
              : t("no: ci sono commit da inviare o da ricevere")}
        </dd>
      </dl>
      <p className="px-4 pb-2 text-[12px] text-vsc-desc">
        {t(
          "Commit, push e l'elenco delle modifiche sono nell'editor o in un agente. Dev-Kit legge lo stato dai file di .git, senza eseguire Git nel progetto.",
        )}
      </p>
      {web?.startsWith("https://") && (
        <div className="px-4 pb-3">
          <ActionButton
            icon={VscGithub}
            label={t("Apri sul web")}
            onClick={() => {
              void urlOpen(web);
            }}
          />
        </div>
      )}
    </Box>
  );
}

export function ProjectPage({
  p,
  agents,
  editor,
  runs,
  logs,
  ports,
  busy,
  onStart,
  onStop,
  onPorts,
  onShowLog,
  shownLog,
  onForget,
  onBack,
  onChanged,
}: {
  p: ProjectInfo;
  agents: AgentInfo[] | null;
  editor: EditorStatus | null;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  ports: Ports;
  busy: string | null;
  onStart: (a: ProjectAction) => void;
  onStop: (key: string) => void;
  onPorts: (p: Ports) => void;
  onShowLog: (key: string) => void;
  shownLog: string | null;
  onForget: (key: string) => void;
  onBack: () => void;
  onChanged: () => void;
}) {
  const [section, setSection] = useState<Section>("run");
  /** The open panes of the side bar. */
  const [panes, setPanes] = useState<Set<string>>(() => new Set(["overview", "sections"]));
  const [status, setStatus] = useState<ProjectStatus | null>(null);
  const [dbs, setDbs] = useState<ProjectDatabases | null>(null);
  const { busy: opening, error, run } = useAction();
  const reload = useCallback(() => {
    projectStatus(p.id).then(setStatus, () => undefined);
    projectDatabases(p.id).then(setDbs, () => undefined);
  }, [p.id]);
  useEffect(reload, [reload]);
  const mine = Object.values(runs)
    .filter((r) => r.project === p.id)
    .sort((a, b) => b.start - a.start);
  const live = mine.filter((r) => alive(r.state));
  const list = sections();
  const current = list.find((x) => x.id === section) ?? list[0];
  const toggle = (id: string) => {
    const next = new Set(panes);
    if (!next.delete(id)) next.add(id);
    setPanes(next);
  };
  const changed = () => {
    onChanged();
    reload();
  };
  return (
    // As the Projects page: a side bar with the project and its sections,
    // the section on the right with its own scroll.
    <div className="absolute inset-0 flex">
      <aside className="flex w-72 shrink-0 flex-col overflow-hidden border-r border-vsc-border bg-vsc-sidebar">
        <div className="flex h-[35px] shrink-0 items-center gap-0.5 pr-2 pl-1">
          <ToolbarIcon icon={VscArrowLeft} label={t("Progetti")} onClick={onBack} />
          <h2 className="min-w-0 flex-1 truncate pl-1 text-[11px] font-normal tracking-wide uppercase" title={p.name}>
            {p.name}
          </h2>
          <ToolbarIcon icon={VscRefresh} label={t("Aggiorna")} onClick={changed} />
          <ToolbarIcon
            icon={VscFolderOpened}
            label={t("Apri la cartella")}
            onClick={() => {
              run("folder", () => folderOpen("projects", p.id));
            }}
          />
        </div>
        <SidePane
          title={t("Progetto")}
          expanded={panes.has("overview")}
          onToggle={() => {
            toggle("overview");
          }}
        >
          <Overview p={p} status={status} dbs={dbs} live={live} onSection={setSection} />
        </SidePane>
        <SidePane
          title={t("Sezioni del progetto")}
          fill
          expanded={panes.has("sections")}
          onToggle={() => {
            toggle("sections");
          }}
        >
          <nav aria-label={t("Sezioni del progetto")} className="-mx-3 -mt-1">
            <ul>
              {list.map((x) => {
                const Icon = x.icon;
                const on = x.id === section;
                return (
                  <li key={x.id}>
                    <button
                      type="button"
                      aria-current={on ? "page" : undefined}
                      onClick={() => {
                        setSection(x.id);
                      }}
                      className={`flex h-[22px] w-full items-center gap-1.5 pr-3 pl-5 text-left ${
                        on ? "bg-vsc-list-active text-vsc-fg" : "hover:bg-vsc-list-hover"
                      }`}
                    >
                      <Icon size={16} aria-hidden="true" className="shrink-0" />
                      <span className="min-w-0 flex-1 truncate">{x.label}</span>
                      {x.id === "components" && p.missing.length > 0 && (
                        <VscWarning size={12} className="text-vsc-warning" aria-label={t("Mancano componenti")} />
                      )}
                      {x.id === "run" && live.length > 0 && (
                        <span className="text-vsc-success" aria-label={t("avviato")}>
                          ●
                        </span>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </nav>
        </SidePane>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex shrink-0 flex-wrap items-center gap-2 px-3 py-2">
          <h3 className="min-w-0 flex-1 truncate text-[13px] font-semibold">{current?.label}</h3>
          <ActionButton
            icon={VscCode}
            label={t("Editor")}
            primary
            disabled={editor?.installed === false}
            busy={opening === "editor"}
            onClick={() => {
              run("editor", () => editorOpen(p.id));
            }}
          />
          <ActionButton
            icon={VscTerminal}
            label={t("Terminale")}
            busy={opening === "terminal"}
            onClick={() => {
              run("terminal", () => terminalOpen(p.id));
            }}
          />
          <AgentPicker agents={agents} project={p.id} preferred={p.settings.agent} />
        </div>
        <ErrorLine error={error} />
        <div className="min-h-0 flex-1 overflow-y-auto border-t border-vsc-border">
          <div className="max-w-5xl space-y-3 pb-3">
            {section === "run" && (
              <>
                <MainAction p={p} runs={runs} logs={logs} busy={busy} onStart={onStart} onStop={onStop} />
                <ProjectDetails
                  project={p}
                  runs={runs}
                  ports={ports}
                  busy={busy}
                  onStart={onStart}
                  onStop={onStop}
                  onPorts={onPorts}
                  onChanged={onChanged}
                  onlyRun
                />
                {mine.length > 0 && (
                  <RunsPanel
                    runs={mine}
                    logs={logs}
                    shown={shownLog}
                    onShow={onShowLog}
                    onStop={onStop}
                    onForget={onForget}
                  />
                )}
              </>
            )}
            {section === "components" && <Needs p={p} onChanged={changed} />}
            {section === "databases" && <Databases p={p} dbs={dbs} onChanged={reload} />}
            {section === "extensions" && (
              <div className="px-3 py-2">
                <ProjectExtensions project={p} onChanged={onChanged} />
              </div>
            )}
            {section === "editor" && <EditorSettingsBox project={p.id} />}
            {section === "git" && <Git status={status} />}
            {section === "agents" && (
              <>
                <Agents status={status} />
                <ProjectMcpBox project={p.id} />
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
