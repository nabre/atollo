// The page of one project (user request, 14/10/2026, D-75): every setting
// and choice of the project in tabs. Starting it, its databases, its editor
// extensions, its agent and its Git state; the kit's settings of the
// project live in data\app\projects\<id>\dev-settings.json, never in it.
import { useCallback, useEffect, useState, type KeyboardEvent } from "react";
import {
  VscArrowLeft,
  VscCloudDownload,
  VscCode,
  VscDatabase,
  VscFolderOpened,
  VscGithub,
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
import { t } from "./i18n";
import { ComponentLogo, LanguageTag } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { alive, ProjectDetails, RunsPanel, stateText } from "./projects";

type Tab = "overview" | "run" | "db" | "editor" | "agents" | "git";

function tabs(): { id: Tab; label: string }[] {
  return [
    { id: "overview", label: t("Panoramica") },
    { id: "run", label: t("Avvio") },
    { id: "db", label: t("Database") },
    { id: "editor", label: t("Editor") },
    { id: "agents", label: t("Agenti") },
    { id: "git", label: t("Git e GitHub") },
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
    <section className="min-w-0 rounded-sm border border-vsc-border bg-vsc-sidebar" aria-labelledby={id}>
      <h2
        id={id}
        className="flex items-center gap-2 border-b border-vsc-border px-4 py-2 text-[12px] font-semibold tracking-wide text-vsc-desc uppercase"
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

function Overview({
  p,
  status,
  dbs,
  runs,
  onChanged,
}: {
  p: ProjectInfo;
  status: ProjectStatus | null;
  dbs: ProjectDatabases | null;
  runs: RunInfo[];
  onChanged: () => void;
}) {
  const { busy, error, run } = useAction(onChanged);
  const running = runs.filter((r) => alive(r.state));
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Box
        id="need"
        title={t("Cosa serve al progetto")}
        right={
          p.missing.length > 0 ? (
            <span className="text-vsc-warning">{t("Mancano: {0}", String(p.missing.length))}</span>
          ) : undefined
        }
      >
        <ul className="divide-y divide-vsc-border">
          {p.requires.length === 0 && <li className="px-4 py-2 text-vsc-desc">{t("Nessun componente del kit.")}</li>}
          {p.requires.map((id) => {
            const missing = p.missing.includes(id);
            return (
              <li key={id} className="flex items-center gap-3 px-4 py-2">
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
      <Box id="now" title={t("Adesso")}>
        <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-5 gap-y-2 px-4 py-3">
          <dt className="text-vsc-desc">{t("Avviati")}</dt>
          <dd className="min-w-0">
            {running.length === 0
              ? t("niente")
              : running.map((r) => (
                  <div key={r.key}>
                    {r.label}: {stateText(r)}
                    {r.url && (
                      <button
                        type="button"
                        className="ml-2 text-vsc-link hover:underline"
                        onClick={() => {
                          void urlOpen(r.url ?? "");
                        }}
                      >
                        {r.url}
                      </button>
                    )}
                  </div>
                ))}
          </dd>
          <dt className="text-vsc-desc">{t("Database")}</dt>
          <dd>
            {dbs === null || dbs.dbs.length === 0
              ? t("nessuno creato")
              : dbs.dbs.map((d) => `${COMPONENT_NAMES[d.engine] ?? d.engine} · ${d.database}`).join(", ")}
          </dd>
          <dt className="text-vsc-desc">{t("Git")}</dt>
          <dd>
            {status?.git
              ? `${status.git.branch ?? t("nessun ramo")}${status.git.synced === false ? ` · ${t("non sincronizzato")}` : ""}`
              : t("non è un repository Git")}
          </dd>
          <dt className="text-vsc-desc">{t("Cartella")}</dt>
          <dd className="truncate font-mono text-[12px]">projects\{p.id}</dd>
          <dt className="text-vsc-desc">{t("Impostazioni del kit")}</dt>
          <dd className="truncate font-mono text-[12px]">data\app\projects\{p.id}\dev-settings.json</dd>
        </dl>
      </Box>
    </div>
  );
}

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
    <div className="space-y-4">
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
                <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-5 gap-y-2 px-4 py-3">
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
              </>
            ) : (
              <div className="flex flex-wrap items-center gap-3 px-4 py-3">
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
          <p className="px-4 py-3 text-vsc-desc">
            {t("Il progetto usa SQLite: il database è un file del progetto, nessun server da avviare.")}
          </p>
        </Box>
      )}
      <ErrorLine error={error} />
    </div>
  );
}

function Agents({ p, agents, status }: { p: ProjectInfo; agents: AgentInfo[] | null; status: ProjectStatus | null }) {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Box id="agent" title={t("Agente")}>
        <div className="space-y-2 px-4 py-3">
          <p className="text-vsc-desc">{t("Quello scelto qui è il predefinito per questo progetto.")}</p>
          <AgentPicker agents={agents} project={p.id} preferred={p.settings.agent} />
        </div>
      </Box>
      <Box id="instructions" title={t("Istruzioni del progetto")}>
        <ul className="divide-y divide-vsc-border">
          {(status?.instructions ?? []).map((f) => (
            <li key={f} className="px-4 py-2 font-mono">
              {f}
            </li>
          ))}
          <li className="px-4 py-2 text-vsc-desc">
            {t("Istruzioni, skill e MCP del kit valgono anche qui (data\\agents).")}
          </li>
        </ul>
      </Box>
    </div>
  );
}

function Git({ status }: { status: ProjectStatus | null }) {
  const git = status?.git;
  if (!status) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  if (!git) return <p className="text-vsc-desc">{t("La cartella non è un repository Git.")}</p>;
  const web = git.remote?.replace(/\.git$/, "").replace(/^git@github\.com:/, "https://github.com/") ?? null;
  return (
    <Box id="repo" title={t("Repository")}>
      <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-5 gap-y-2 px-4 py-3">
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
  const [tab, setTab] = useState<Tab>("overview");
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
  const list = tabs();
  const onKey = (e: KeyboardEvent<HTMLButtonElement>, i: number) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const next = list[(i + d + list.length) % list.length];
    if (next) {
      setTab(next.id);
      document.getElementById(`tab-${next.id}`)?.focus();
    }
  };
  return (
    <div className="max-w-5xl space-y-4">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <button type="button" className="flex items-center gap-1 text-vsc-link hover:underline" onClick={onBack}>
          <VscArrowLeft size={14} aria-hidden="true" />
          {t("Progetti")}
        </button>
        <h2 className="text-[20px] font-semibold text-vsc-fg">{p.name}</h2>
        <span className="flex flex-wrap gap-1">
          {p.languages.map((l) => (
            <LanguageTag key={l} name={l} />
          ))}
        </span>
        <span className="ml-auto flex flex-wrap gap-2">
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
          <ActionButton
            icon={VscFolderOpened}
            label={t("Apri la cartella")}
            onClick={() => {
              run("folder", () => folderOpen("projects", p.id));
            }}
          />
        </span>
      </div>
      <ErrorLine error={error} />
      <div
        role="tablist"
        aria-label={t("Sezioni del progetto")}
        className="flex gap-1 overflow-x-auto border-b border-vsc-border"
      >
        {list.map((x, i) => (
          <button
            key={x.id}
            id={`tab-${x.id}`}
            type="button"
            role="tab"
            aria-selected={tab === x.id}
            aria-controls={`panel-${x.id}`}
            tabIndex={tab === x.id ? 0 : -1}
            onClick={() => {
              setTab(x.id);
            }}
            onKeyDown={(e) => {
              onKey(e, i);
            }}
            className={`border-b-2 px-3 py-2 whitespace-nowrap ${
              tab === x.id ? "border-vsc-focus text-vsc-fg" : "border-transparent text-vsc-desc hover:text-vsc-fg"
            }`}
          >
            {x.label}
            {x.id === "overview" && p.missing.length > 0 && (
              <VscWarning size={12} className="ml-1 inline text-vsc-warning" aria-label={t("Mancano componenti")} />
            )}
            {x.id === "run" && mine.some((r) => alive(r.state)) && (
              <span className="ml-1 text-vsc-success" aria-label={t("avviato")}>
                ●
              </span>
            )}
          </button>
        ))}
      </div>
      <div role="tabpanel" id={`panel-${tab}`} aria-labelledby={`tab-${tab}`}>
        {tab === "overview" && (
          <Overview
            p={p}
            status={status}
            dbs={dbs}
            runs={mine}
            onChanged={() => {
              onChanged();
              reload();
            }}
          />
        )}
        {tab === "run" && (
          <div className="space-y-4">
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
          </div>
        )}
        {tab === "db" && <Databases p={p} dbs={dbs} onChanged={reload} />}
        {tab === "editor" && <ProjectExtensions project={p} onChanged={onChanged} />}
        {tab === "agents" && <Agents p={p} agents={agents} status={status} />}
        {tab === "git" && <Git status={status} />}
      </div>
    </div>
  );
}
