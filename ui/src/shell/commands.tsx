// The command palette (Ctrl+Shift+P, report v6/04 N-8, D-160), as the
// editor's: every action of the kit found by typing a few letters. Start or
// stop a project, open its page or the editor, an agent in it, a database
// server, a page; «Ferma tutto» when something runs.
import { useCallback } from "react";
import type { IconType } from "react-icons";
import {
  agentLaunch,
  editorOpen,
  llmStop,
  projectsList,
  runStart,
  runStop,
  serverStop,
  serviceStart,
  servicesList,
  serviceStop,
  type AgentInfo,
  type EditorStatus,
  type ProjectInfo,
  type RunInfo,
  type ServiceInfo,
} from "../api";
import type { Running } from "./activity";
import { useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { Palette, type PaletteItem } from "../shared/palette";
import { mainAction } from "../projects/home";
import { alive } from "../projects/runs";
import { codicon } from "../workbench";

/** Stops what `running` has: the actions, the pools, the database versions
 * and the local models' server; rejects with what failed, a line each. */
export async function stopEverything(running: Running): Promise<void> {
  const results = await Promise.allSettled([
    ...running.liveRuns.map((r) => runStop(r.key)),
    ...running.pools.map((p) => serverStop(p.id)),
    ...running.lines.map((s) => serviceStop(s.id, s.line)),
    ...(running.llm ? [llmStop()] : []),
  ]);
  running.reloadServices();
  running.reloadPools();
  running.reloadLlm();
  const failed = results.flatMap((r) => (r.status === "rejected" ? [String(r.reason)] : []));
  if (failed.length > 0) throw new Error(failed.join("\n"));
}

const icon = (Icon: IconType) => <Icon size={16} aria-hidden="true" />;

/** The newest first, as Quick Open: modified or opened last. */
function byRecent(projects: ProjectInfo[]): ProjectInfo[] {
  return [...projects].sort(
    (a, b) => Math.max(b.modified, b.settings.lastOpened ?? 0) - Math.max(a.modified, a.settings.lastOpened ?? 0),
  );
}

export interface PalettePage {
  id: string;
  label: string;
  icon: IconType;
}

export function CommandPalette({
  pages,
  agents,
  editor,
  runs,
  running,
  onPage,
  onOpenProject,
  onError,
  onClose,
}: {
  pages: PalettePage[];
  agents: AgentInfo[];
  editor: EditorStatus | null;
  runs: Record<string, RunInfo>;
  running: Running;
  onPage: (id: string) => void;
  onOpenProject: (id: string) => void;
  /** What failed after the palette closed. */
  onError: (error: string) => void;
  onClose: () => void;
}) {
  const { data: projects } = useLoad<ProjectInfo[]>(projectsList, [], { onError: "keep" });
  const { data: services } = useLoad<ServiceInfo[]>(servicesList, [], { onError: "keep" });
  const act = useCallback(
    (work: () => Promise<unknown>) => () => {
      work().catch((e: unknown) => {
        onError(e instanceof Error ? e.message : String(e));
      });
    },
    [onError],
  );

  const items: PaletteItem[] = [];
  if (running.count > 0) {
    items.push({
      key: "stop-all",
      icon: icon(codicon("debug-stop")),
      label: t("Ferma tutto"),
      detail: t("progetti, database, modelli locali, server web"),
      run: act(() => stopEverything(running)),
    });
  }
  const installed = agents.filter((a) => a.installed);
  for (const p of byRecent(projects)) {
    const a = mainAction(p);
    if (a) {
      const key = `${p.id}|${a.id}`;
      const r = runs[key];
      items.push(
        r && alive(r.state)
          ? {
              key: `stop:${key}`,
              icon: icon(codicon("debug-stop")),
              label: t("Ferma {0}", p.name),
              detail: a.label,
              run: act(() => runStop(key)),
            }
          : {
              key: `start:${key}`,
              icon: icon(codicon("play")),
              label: t("Avvia {0}", p.name),
              detail: a.label,
              run: act(() => runStart(p.id, a.id)),
            },
      );
    }
    items.push({
      key: `open:${p.id}`,
      icon: icon(codicon("folder-opened")),
      label: t("Apri {0}", p.name),
      detail: t("la pagina del progetto"),
      run: () => {
        onOpenProject(p.id);
      },
    });
    if (editor?.installed) {
      items.push({
        key: `editor:${p.id}`,
        icon: icon(codicon("code")),
        label: t("Apri {0} nell'editor", p.name),
        detail: p.platform,
        run: act(() => editorOpen(p.id)),
      });
    }
    for (const agent of installed) {
      items.push({
        key: `agent:${agent.id}:${p.id}`,
        icon: icon(codicon("hubot")),
        label: t("{0} in {1}", agent.name, p.name),
        detail: t("agente"),
        run: act(() => agentLaunch(agent.id, p.id)),
      });
    }
  }
  for (const s of services.filter((x) => x.installed)) {
    const on = s.state === "running" || s.state === "starting";
    items.push({
      key: `db:${s.id}`,
      icon: icon(codicon("database")),
      label: on ? t("Ferma {0}", s.name) : t("Avvia {0}", s.name),
      detail: s.line ? `${t("database")} ${s.line}` : t("database"),
      run: act(() => (on ? serviceStop(s.id) : serviceStart(s.id))),
    });
  }
  for (const page of pages) {
    items.push({
      key: `page:${page.id}`,
      icon: icon(page.icon),
      label: t("Vai a {0}", page.label),
      run: () => {
        onPage(page.id);
      },
    });
  }

  return (
    <Palette
      label={t("Comandi")}
      placeholder={t("Cosa vuoi fare? Avvia, ferma, apri, un progetto, un agente…")}
      listLabel={t("Comandi trovati")}
      items={items}
      empty={t("Nessun comando corrisponde alla ricerca.")}
      hint={t("Invio esegue · Esc chiude")}
      onClose={onClose}
    />
  );
}
