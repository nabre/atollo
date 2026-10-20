// The page of one project (user request, 14/10/2026, D-75), as an
// extension's page in the editor: a header with what the project is and its
// buttons, a navigation bar, one section at a time. Starting it, its
// components, databases, editor extensions and settings, its Git state; the
// kit's settings of the project live in
// data\projects\<id>\dev-settings.json, never in it. Deleting an empty
// project asks first, with what goes with it (./delete.tsx).
import { useCallback, useState } from "react";
import {
  editorOpen,
  folderOpen,
  terminalOpen,
  type AgentInfo,
  type EditorStatus,
  type Ports,
  type ProjectAction,
  type ProjectInfo,
  type RunInfo,
} from "../api";
import { useAction, useLoad } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { t } from "../i18n";
import { Button, Codicon, codicon, Dot, IconButton } from "../workbench";
import { projectDatabases, projectStatus, type ProjectDatabases, type ProjectStatus } from "../api";
import { AgentPicker } from "../agents";
import { ProjectExtensions } from "../extensions";
import { EditorSettingsBox } from "./editorsettings";
import { ProjectMcpBox } from "./mcp";
import { ProjectVersions } from "../versions";
import { ActionBar, EditorBody, EditorHeader, EditorSection, NavBar, type NavItem } from "../workbench";
import { alive } from "./runs";
import { KIND_ICON, kindText, LanguageIcons } from "./kind";
import { ActionDetail, ActionsList } from "./run";
import { ActionForm } from "./actionform";
import { SiteForm } from "./site";
import { COMPONENT_NAMES, Needs } from "./needs";
import { Databases } from "./databases";
import { Git } from "./git";
import { DeleteProjectDialog } from "./delete";

export type ProjectSection = "run" | "site" | "components" | "databases" | "extensions" | "editor" | "git";
type Section = ProjectSection;

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
  initial,
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
  /** Where the page opens (the home's «Installa», «Vedi l'output»). */
  initial?: { section?: ProjectSection; action?: string } | undefined;
}) {
  const [section, setSection] = useState<Section>(initial?.section ?? "run");
  /** The action shown in Avvio (null: the default one). */
  const [chosen, setChosen] = useState<string | null>(initial?.action ?? null);
  /** The confirmation of the deletion is open. */
  const [deleting, setDeleting] = useState(false);
  const readStatus = useCallback(() => projectStatus(p.id), [p.id]);
  const readDbs = useCallback(() => projectDatabases(p.id), [p.id]);
  const { data: status, reload: reloadStatus } = useLoad<ProjectStatus | null>(readStatus, null, { onError: "keep" });
  const { data: dbs, reload: reloadDbs } = useLoad<ProjectDatabases | null>(readDbs, null, { onError: "keep" });
  const { busy: opening, error, run } = useAction();
  const reload = useCallback(() => {
    reloadStatus();
    reloadDbs();
  }, [reloadStatus, reloadDbs]);
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
  /** «Nuova azione» is open in Azioni. */
  const [adding, setAdding] = useState(false);
  /** An action saved (or removed) in panel.json: the project is read again,
   * with that action chosen (gone, the default one). */
  const saved = (id: string) => {
    setChosen(id);
    changed();
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
          <Codicon name="warning" size={12} className="text-vsc-warning" label={t("Mancano componenti")} />
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
              <Codicon name="github" size={14} />
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
              <Codicon name="warning" size={14} />
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
                icon={codicon("folder-opened")}
                label={t("Apri la cartella")}
                onClick={() => {
                  run("folder", () => folderOpen("projects", p.id));
                }}
              />
              <IconButton icon={codicon("refresh")} label={t("Aggiorna")} onClick={changed} />
            </ActionBar>
          </>
        }
      />
      <div className="px-5 empty:hidden">
        <ErrorLine error={error} />
      </div>
      <NavBar label={t("Sezioni del progetto")} items={items} active={section} onSelect={setSection} />
      <EditorBody fill={section === "run"}>
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
                onSaved={saved}
              />
            )}
            <EditorSection
              id="actions"
              title={t("Azioni")}
              fill
              actions={
                !adding && (
                  <ActionButton
                    icon={codicon("add")}
                    label={t("Nuova azione")}
                    small
                    onClick={() => {
                      setAdding(true);
                    }}
                  />
                )
              }
            >
              {adding && (
                <div className="px-2 pb-2">
                  <ActionForm
                    p={p}
                    onSaved={(id) => {
                      setAdding(false);
                      saved(id);
                    }}
                    onCancel={() => {
                      setAdding(false);
                    }}
                  />
                </div>
              )}
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
                icon={codicon("trash")}
                label={t("Elimina la cartella vuota")}
                onClick={() => {
                  setDeleting(true);
                }}
              />
            )}
            {deleting && (
              <DeleteProjectDialog
                project={p.id}
                onCancel={() => {
                  setDeleting(false);
                }}
                onDeleted={() => {
                  setDeleting(false);
                  onChanged();
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
            <ProjectVersions p={p} />
          </>
        )}
        {section === "databases" && <Databases p={p} dbs={dbs} onChanged={reload} />}
        {section === "extensions" && <ProjectExtensions project={p} onChanged={onChanged} />}
        {section === "editor" && (
          <>
            <EditorSettingsBox project={p.id} />
            <ProjectMcpBox project={p.id} />
          </>
        )}
        {section === "git" && <Git status={status} />}
      </EditorBody>
    </div>
  );
}
