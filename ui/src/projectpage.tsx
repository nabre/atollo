// The page of one project (user request, 14/10/2026, D-75), as an
// extension's page in the editor: a header with what the project is and its
// buttons, a navigation bar, one section at a time. Starting it, its
// components, databases, editor extensions and settings, its Git state; the
// kit's settings of the project live in
// data\app\projects\<id>\dev-settings.json, never in it.
import { useCallback, useEffect, useState } from "react";
import {
  VscArrowLeft,
  VscCode,
  VscDebugStart,
  VscDebugStop,
  VscFolderOpened,
  VscGithub,
  VscGlobe,
  VscRefresh,
  VscStarEmpty,
  VscStarFull,
  VscTrash,
  VscWarning,
} from "react-icons/vsc";
import {
  editorOpen,
  folderOpen,
  projectDatabases,
  projectDbCreate,
  projectEnvApply,
  projectEnvPreview,
  projectActionDefault,
  projectDelete,
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
import { ProjectVersions } from "./versions";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { alive, KIND_ICON, kindText, LanguageIcons, LogView, PortField, SiteForm, stateText } from "./projects";
import {
  ActionBar,
  Button,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  List,
  ListRow,
  NavBar,
  Notice,
  Properties,
  type NavItem,
} from "./workbench";

type Section = "run" | "site" | "components" | "databases" | "extensions" | "editor" | "git";

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
  android: "Android SDK",
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
    <>
      <EditorSection
        id="need"
        title={t("Componenti")}
        actions={
          p.missing.length > 0 && (
            <ActionButton
              icon={VscDebugStart}
              label={t("Installa ciò che manca")}
              primary
              busy={busy === "install"}
              onClick={() => {
                run("install", () => projectInstallMissing(p.id));
              }}
            />
          )
        }
      >
        {p.requires.length === 0 ? (
          <p className="text-vsc-desc">{t("Nessun componente del kit.")}</p>
        ) : (
          <List label={t("Componenti")} className="-mx-2">
            {p.requires.map((id) => {
              const missing = p.missing.includes(id);
              return (
                <ListRow
                  key={id}
                  icon={<ComponentLogo id={id} />}
                  label={COMPONENT_NAMES[id] ?? id}
                  decorations={
                    <span className={missing ? "text-vsc-warning" : "text-vsc-success"}>
                      {missing ? t("da installare") : t("installato")}
                    </span>
                  }
                />
              );
            })}
          </List>
        )}
        <ErrorLine error={error} />
      </EditorSection>
      <EditorSection id="need-why" title={t("Come è stato riconosciuto")}>
        <p className="select-text">{p.reason}</p>
        {p.warnings.map((w) => (
          <Notice key={w} tone="warning">
            {w}
          </Notice>
        ))}
      </EditorSection>
    </>
  );
}

/** The actions of the project, as a list: the row selects the action, its
 * inline button starts or stops it, the star marks the default one. */
function ActionsList({
  p,
  runs,
  busy,
  selected,
  onSelect,
  onStart,
  onStop,
}: {
  p: ProjectInfo;
  runs: Record<string, RunInfo>;
  busy: string | null;
  selected: string | null;
  onSelect: (id: string) => void;
  onStart: (a: ProjectAction) => void;
  onStop: (key: string) => void;
}) {
  if (p.actions.length === 0) {
    return (
      <p className="px-2 text-vsc-desc">
        {t("Nessuna azione riconosciuta: aggiungine in panel.json o servi una cartella come sito web.")}
      </p>
    );
  }
  return (
    <List label={t("Azioni di {0}", p.name)}>
      {p.actions.map((a) => {
        const key = `${p.id}|${a.id}`;
        const r = runs[key];
        const Icon = KIND_ICON[a.type];
        return (
          <ListRow
            key={a.id}
            icon={<Icon size={16} aria-hidden="true" />}
            label={a.label}
            description={r ? stateText(r) : kindText(a.type)}
            title={a.static ?? a.apache ?? a.command}
            selected={a.id === selected}
            onSelect={() => {
              onSelect(a.id);
            }}
            decorations={
              <>
                {p.default === a.id && (
                  <span role="img" aria-label={t("predefinita")} title={t("predefinita")} className="opacity-70">
                    <VscStarFull size={12} aria-hidden="true" />
                  </span>
                )}
                {r && <Dot tone={r.state === "error" ? "err" : alive(r.state) ? "ok" : "off"} label={stateText(r)} />}
              </>
            }
            actions={
              alive(r?.state) ? (
                <IconButton
                  icon={VscDebugStop}
                  label={t("Ferma")}
                  busy={busy === `stop:${key}`}
                  onClick={() => {
                    onStop(key);
                  }}
                />
              ) : (
                <IconButton
                  icon={VscDebugStart}
                  label={t("Avvia")}
                  busy={busy === `run:${key}`}
                  onClick={() => {
                    onSelect(a.id);
                    onStart(a);
                  }}
                />
              )
            }
          />
        );
      })}
    </List>
  );
}

/** The action chosen in the list: what it runs, its port, its output. */
function ActionDetail({
  p,
  action: a,
  runs,
  logs,
  ports,
  onPorts,
  onForget,
  onChanged,
}: {
  p: ProjectInfo;
  action: ProjectAction;
  runs: Record<string, RunInfo>;
  logs: Record<string, string[]>;
  ports: Ports;
  onPorts: (p: Ports) => void;
  onForget: (key: string) => void;
  onChanged: () => void;
}) {
  const { busy: working, error, run } = useAction(onChanged);
  const key = `${p.id}|${a.id}`;
  const r = runs[key];
  const isDefault = p.default === a.id;
  const rows: [string, React.ReactNode][] = [
    [t("Stato"), r ? stateText(r) : t("fermo")],
    [
      t("Comando"),
      <span key="cmd" className="font-mono text-[12px] break-all select-text">
        {a.static !== null
          ? t("file server del kit: {0}", a.static)
          : a.apache !== null
            ? t("Apache del progetto: {0}", a.apache)
            : a.command}
      </span>,
    ],
  ];
  if (a.folder) rows.push([t("Cartella"), <span className="font-mono text-[12px]">{a.folder}</span>]);
  if (a.type === "web") {
    rows.push([
      t("Porta"),
      // Remounted when the saved port changes: the field shows it.
      <PortField key={`${key}:${String(ports.ports[key] ?? "")}`} actionKey={key} ports={ports} onPorts={onPorts} />,
    ]);
  }
  if (r?.url && r.state === "running") {
    rows.push([
      t("Indirizzo"),
      <button
        type="button"
        className="text-vsc-link hover:underline"
        onClick={() => {
          void urlOpen(r.url ?? "");
        }}
      >
        {r.url}
      </button>,
    ]);
  }
  if (a.custom) rows.push([t("Origine"), "panel.json"]);
  return (
    <EditorSection
      id={`action-${a.id}`}
      title={a.label}
      actions={
        <ActionBar>
          {/* The default action: the project's Avvia starts it. */}
          <IconButton
            icon={isDefault ? VscStarFull : VscStarEmpty}
            label={t("Predefinita")}
            title={t("La avvia il pulsante Avvia del progetto")}
            checked={isDefault}
            busy={working === "default"}
            onClick={() => {
              if (!isDefault) run("default", () => projectActionDefault(p.id, a.id));
            }}
          />
          {r?.url && r.state === "running" && (
            <IconButton
              icon={VscGlobe}
              label={t("Apri nel browser")}
              onClick={() => {
                void urlOpen(r.url ?? "");
              }}
            />
          )}
          {r && !alive(r.state) && (
            <IconButton
              icon={VscTrash}
              label={t("Togli l'output")}
              onClick={() => {
                onForget(key);
              }}
            />
          )}
        </ActionBar>
      }
    >
      <Properties rows={rows} />
      <ErrorLine error={error} />
      {r && <LogView title={a.label} lines={logs[key] ?? []} />}
    </EditorSection>
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
    <div className="space-y-2">
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
        <Notice tone="success">{t("Il file .env ha già questa connessione.")}</Notice>
      ) : (
        <>
          <p className="text-vsc-desc">{t("Righe di .env che cambiano:")}</p>
          <div className="overflow-x-auto rounded border border-vsc-border">
            <table className="w-full text-[12px]">
              <tbody>
                {changes.map((c) => (
                  <tr key={c.key} className="border-t border-vsc-border first:border-t-0">
                    <td className="px-2 py-1 font-mono">{c.key}</td>
                    <td className="px-2 py-1 font-mono text-vsc-desc line-through">{c.old ?? ""}</td>
                    <td className="px-2 py-1 font-mono">
                      {c.key.includes("PASSWORD") || c.key.includes("URI") || c.key.includes("URL") ? "••••" : c.new}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {!envIgnored && (
            <Notice tone="warning">
              {t(".env non è in .gitignore: con la password dentro finirebbe nel repository. Aggiungilo a .gitignore.")}
            </Notice>
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
    <>
      {engines.map((engine) => {
        const db = dbs.dbs.find((d) => d.engine === engine);
        const used = dbs.detected.includes(engine);
        return (
          <EditorSection
            key={engine}
            id={`pdb-${engine}`}
            title={COMPONENT_NAMES[engine] ?? engine}
            actions={used ? <span className="text-[12px] text-vsc-desc">{t("usato dal progetto")}</span> : undefined}
          >
            {db ? (
              <div className="space-y-3">
                <Properties
                  rows={[
                    [t("Database"), <span className="font-mono">{db.database}</span>],
                    [t("Utente"), <span className="font-mono">{db.user}</span>],
                    [t("Password"), <SecretField value={db.password} label={t("la password di {0}", db.user)} />],
                    [
                      t("Indirizzo"),
                      <span className="font-mono">
                        {db.host}:{String(db.port)}
                      </span>,
                    ],
                    [t("Stringa"), <SecretField value={db.url} label={t("la stringa di connessione")} />],
                  ]}
                />
                <EnvPreview project={p.id} db={db} envIgnored={dbs.envIgnored} onDone={onChanged} />
                <DatabaseCopies project={p.id} engine={db.engine} />
              </div>
            ) : (
              <div className="flex flex-wrap items-center gap-3">
                <p className="min-w-0 flex-1 text-vsc-desc">
                  {t("Un database e un utente solo per questo progetto, con una password generata dal kit.")}
                </p>
                <ActionButton
                  icon={VscCode}
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
          </EditorSection>
        );
      })}
      {dbs.detected.includes("sqlite") && (
        <EditorSection id="pdb-sqlite" title="SQLite">
          <p className="text-vsc-desc">
            {t("Il progetto usa SQLite: il database è un file del progetto, nessun server da avviare.")}
          </p>
        </EditorSection>
      )}
      <ErrorLine error={error} />
    </>
  );
}

function Git({ status }: { status: ProjectStatus | null }) {
  const git = status?.git;
  if (!status) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  if (!git) return <p className="text-vsc-desc">{t("La cartella non è un repository Git.")}</p>;
  const web = git.remote?.replace(/\.git$/, "").replace(/^git@github\.com:/, "https://github.com/") ?? null;
  return (
    <EditorSection
      id="repo"
      title={t("Repository")}
      actions={
        web?.startsWith("https://") && (
          <ActionButton
            icon={VscGithub}
            label={t("Apri sul web")}
            onClick={() => {
              void urlOpen(web);
            }}
          />
        )
      }
    >
      <Properties
        rows={[
          [t("Remoto"), <span className="truncate font-mono select-text">{git.remote ?? t("nessuno")}</span>],
          [t("Ramo"), <span className="font-mono">{git.branch ?? t("nessun ramo")}</span>],
          [
            t("Remoto del ramo"),
            <span className="font-mono">{git.upstream ?? t("ramo non collegato al remoto")}</span>,
          ],
          [
            t("Sincronizzato"),
            git.synced === null
              ? t("non si sa")
              : git.synced
                ? t("sì, allo stesso commit (all'ultimo aggiornamento dal remoto)")
                : t("no: ci sono commit da inviare o da ricevere"),
          ],
        ]}
      />
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Commit, push e l'elenco delle modifiche sono nell'editor o in un agente. Atollo legge lo stato dai file di .git, senza eseguire Git nel progetto.",
        )}
      </p>
    </EditorSection>
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
  onForget,
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
  onForget: (key: string) => void;
  onChanged: () => void;
}) {
  const [section, setSection] = useState<Section>("run");
  /** The action shown in Avvio (null: the default one). */
  const [chosen, setChosen] = useState<string | null>(null);
  const [status, setStatus] = useState<ProjectStatus | null>(null);
  const [dbs, setDbs] = useState<ProjectDatabases | null>(null);
  const { busy: opening, error, run } = useAction();
  const reload = useCallback(() => {
    projectStatus(p.id).then(setStatus, () => undefined);
    projectDatabases(p.id).then(setDbs, () => undefined);
  }, [p.id]);
  useEffect(reload, [reload]);
  const live = Object.values(runs).filter((r) => r.project === p.id && alive(r.state));
  const action =
    p.actions.find((a) => a.id === chosen) ?? p.actions.find((a) => a.id === p.default) ?? p.actions[0] ?? null;
  const main = p.actions.find((a) => a.id === p.default) ?? p.actions[0] ?? null;
  const mainRunning = main !== null && alive(runs[`${p.id}|${main.id}`]?.state);
  const select = (id: string) => {
    setChosen(id);
    setSection("run");
    if (runs[`${p.id}|${id}`]) onShowLog(`${p.id}|${id}`);
  };
  const changed = () => {
    onChanged();
    reload();
  };
  const Icon = KIND_ICON[p.type];
  const items: NavItem<Section>[] = [
    { id: "run", label: t("Avvio"), mark: live.length > 0 ? <Dot tone="ok" label={t("avviato")} /> : undefined },
    { id: "site", label: t("Sito web") },
    {
      id: "components",
      label: t("Componenti"),
      mark:
        p.missing.length > 0 ? (
          <VscWarning size={12} className="text-vsc-warning" aria-label={t("Mancano componenti")} />
        ) : undefined,
    },
    { id: "databases", label: t("Database") },
    { id: "extensions", label: t("Estensioni") },
    { id: "editor", label: t("Editor") },
    { id: "git", label: t("Git") },
  ];

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<Icon size={56} aria-hidden="true" className="text-vsc-icon opacity-80" />}
        title={p.name}
        meta={[
          <span key="kind">
            {p.platform} · {kindText(p.type)}
          </span>,
          p.languages.length > 0 ? <LanguageIcons key="langs" names={p.languages} /> : null,
          status?.git ? (
            <span key="git" className="flex items-center gap-1" title={t("Ramo")}>
              <VscGithub size={14} aria-hidden="true" />
              <span className="font-mono">{status.git.branch ?? t("nessun ramo")}</span>
            </span>
          ) : null,
          live.length > 0 ? (
            <span key="live" className="flex items-center gap-1 text-vsc-success">
              <Dot tone="ok" />
              {live.some((r) => r.type === "web") ? t("in ascolto") : t("in esecuzione")}
            </span>
          ) : null,
          p.missing.length > 0 ? (
            <button
              key="missing"
              type="button"
              className="flex items-center gap-1 text-vsc-warning hover:underline"
              onClick={() => {
                setSection("components");
              }}
            >
              <VscWarning size={14} aria-hidden="true" />
              {t("Mancano: {0}", p.missing.map((m) => COMPONENT_NAMES[m] ?? m).join(", "))}
            </button>
          ) : null,
        ]}
        actions={
          <>
            {main &&
              (mainRunning ? (
                <Button
                  busy={busy === `stop:${p.id}|${main.id}`}
                  onClick={() => {
                    onStop(`${p.id}|${main.id}`);
                  }}
                >
                  {t("Ferma")}
                </Button>
              ) : (
                <Button
                  variant="primary"
                  busy={busy === `run:${p.id}|${main.id}`}
                  title={main.label}
                  onClick={() => {
                    select(main.id);
                    onStart(main);
                  }}
                >
                  {t("Avvia")}
                </Button>
              ))}
            <Button
              variant={main ? "secondary" : "primary"}
              disabled={editor?.installed === false}
              busy={opening === "editor"}
              onClick={() => {
                run("editor", () => editorOpen(p.id));
              }}
            >
              {t("Editor")}
            </Button>
            <Button
              busy={opening === "terminal"}
              onClick={() => {
                run("terminal", () => terminalOpen(p.id));
              }}
            >
              {t("Terminale")}
            </Button>
            <AgentPicker agents={agents} project={p.id} preferred={p.settings.agent} />
            <ActionBar className="ml-1">
              <IconButton
                icon={VscFolderOpened}
                label={t("Apri la cartella")}
                onClick={() => {
                  run("folder", () => folderOpen("projects", p.id));
                }}
              />
              <IconButton icon={VscRefresh} label={t("Aggiorna")} onClick={changed} />
            </ActionBar>
          </>
        }
      />
      <div className="px-5 empty:hidden">
        <ErrorLine error={error} />
      </div>
      <NavBar label={t("Sezioni del progetto")} items={items} active={section} onSelect={setSection} />
      <EditorBody narrow fill={section === "run"}>
        {section === "run" && (
          <>
            {action && (
              <ActionDetail
                p={p}
                action={action}
                runs={runs}
                logs={logs}
                ports={ports}
                onPorts={onPorts}
                onForget={onForget}
                onChanged={onChanged}
              />
            )}
            <EditorSection id="actions" title={t("Azioni")} fill>
              <ActionsList
                p={p}
                runs={runs}
                busy={busy}
                selected={action?.id ?? null}
                onSelect={select}
                onStart={onStart}
                onStop={onStop}
              />
            </EditorSection>
            {p.empty && (
              <ActionButton
                icon={VscTrash}
                label={t("Elimina la cartella vuota")}
                busy={opening === "delete"}
                onClick={() => {
                  run("delete", () => projectDelete(p.id));
                }}
              />
            )}
          </>
        )}
        {section === "site" && (
          <EditorSection id="site" title={t("Servi come sito web")}>
            <SiteForm project={p} onChanged={onChanged} />
          </EditorSection>
        )}
        {section === "components" && (
          <>
            <Needs p={p} onChanged={changed} />
            <ProjectVersions project={p.id} />
          </>
        )}
        {section === "databases" && <Databases p={p} dbs={dbs} onChanged={reload} />}
        {section === "extensions" && <ProjectExtensions project={p} onChanged={onChanged} />}
        {section === "editor" && <EditorSettingsBox project={p.id} />}
        {section === "git" && <Git status={status} />}
      </EditorBody>
    </div>
  );
}
