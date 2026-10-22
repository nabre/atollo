// Main window, laid out like VSCodium: the banner, the activity bar, the
// page (side bar and editor, workbench/), the Activity panel and the
// status bar. The pages (D-76): what concerns the projects on Projects (the
// new-project wizard among its tabs, D-83), what concerns the kit on the
// kit's pages.
// Texts go through t() (i18n.ts) in the language of the settings or of
// Windows, sent by app_info.
import { lazy, Suspense, useCallback, useEffect, useState, type KeyboardEvent, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  appInfo,
  editorStatus,
  onEditorStatus,
  onCloseRequested,
  onOpenProject,
  runStop,
  updateCheck,
  type AppInfo,
  type EditorStatus,
  type OpenNow,
  type RunInfo,
  type SetupSnapshot,
  type UpdateStatus,
} from "./api";
import { updateFound, updateText } from "./update";
import { ProjectsPage, useRuns, WIZARD_TAB } from "./projects";
import { ActivityPanel, useRunning, type Running } from "./activity";
import { TitleBar } from "./titlebar";
import { AgentsPage, useAgents } from "./agents";
import { setupStatusText, useSetup } from "./setup";
import { CloseDialog, EjectPage } from "./eject";
import { setLang, t, type Lang } from "./i18n";
import { DatabasesPage } from "./databases";
import { ModelsPage } from "./models";
import { ServersPage } from "./servers";
import { TerminalsPage } from "./terminals";
import { tauriPty, useTerminals, type Terminals } from "./terminal";
import type { GuideTopicId } from "./guide";
import { Codicon, codicon, Icon, Notice, Sash, useStoredSize, useTabs } from "./workbench";
import { useLoad, useSubscription } from "./hooks";
import { LoadError } from "./common";

// The pages seldom open (the Guide, large; Settings; Tools, with the
// first-start page that shares its rows): each its own chunk, read the
// first time the page is shown, or as soon as the pointer or the focus
// reaches its item in the activity bar (PRELOAD).
const loadGuide = () => import("./guide");
const loadSettings = () => import("./settings/page");
const loadTools = () => import("./components");
const GuidePage = lazy(() => loadGuide().then((m) => ({ default: m.GuidePage })));
const SettingsPage = lazy(() => loadSettings().then((m) => ({ default: m.SettingsPage })));
const ToolsPage = lazy(() => loadTools().then((m) => ({ default: m.ToolsPage })));
const ComponentsPage = lazy(() => loadTools().then((m) => ({ default: m.ComponentsPage })));
const PRELOAD: Partial<Record<PageId, () => Promise<unknown>>> = {
  guide: loadGuide,
  settings: loadSettings,
  components: loadTools,
};
// The terminal panel brings xterm.js: read when a session first opens, on
// every system (D-148, the kit's ConPTY on Windows).
const TerminalPanel = lazy(() => import("./terminal/panel").then((m) => ({ default: m.TerminalPanel })));

type PageId =
  | "projects"
  | "agents"
  | "models"
  | "databases"
  | "servers"
  | "terminals"
  | "components"
  | "guide"
  | "settings"
  | "eject";

interface Page {
  id: PageId;
  label: string;
  icon: IconType;
}

function pages(): Page[] {
  return [
    { id: "projects", label: t("Progetti"), icon: codicon("folder-library") },
    { id: "agents", label: t("Agenti"), icon: codicon("hubot") },
    { id: "models", label: t("Modelli locali"), icon: codicon("chip") },
    { id: "databases", label: t("Database"), icon: codicon("database") },
    { id: "servers", label: t("Server Web"), icon: codicon("globe") },
    { id: "terminals", label: t("Terminali"), icon: codicon("terminal") },
    { id: "components", label: t("Strumenti"), icon: codicon("extensions") },
    { id: "guide", label: t("Guida"), icon: codicon("book") },
    { id: "settings", label: t("Impostazioni"), icon: codicon("settings-gear") },
    { id: "eject", label: t("Rimuovi l'unità"), icon: codicon("debug-disconnect") },
  ];
}

export function App() {
  const [page, setPage] = useState<PageId>("projects");
  const [guideTopic, setGuideTopic] = useState<GuideTopicId>("welcome");
  const openGuide = useCallback((topic: GuideTopicId = "welcome") => {
    setGuideTopic(topic);
    setPage("guide");
  }, []);
  /** The projects open in the editor, kept while other pages are shown. */
  const projectTabs = useTabs();
  /** The tools open in the editor of the Tools page, kept the same way. */
  const toolTabs = useTabs();
  const [closing, setClosing] = useState<OpenNow | null>(null);
  const setup = useSetup();
  const { agents } = useAgents();
  const { runs, logs, loadLog, forget } = useRuns();
  // Actions, database versions and web server pools (UX-V5-01).
  const running = useRunning(runs);
  const live = running.count;

  const {
    data: editor,
    setData: setEditor,
    reload: refreshEditor,
  } = useLoad<EditorStatus | null>(editorStatus, null, { onError: { fallback: null }, every: 5000 });
  useSubscription(onEditorStatus, setEditor);

  // The kit's state: a read that works clears the error of the one before.
  // The texts take the language before the page renders with it.
  const readInfo = useCallback(
    () =>
      appInfo().then((i) => {
        setLang(i.lang);
        return i;
      }),
    [],
  );
  const {
    data: info,
    error,
    reload: loadInfo,
    retry: retryInfo,
  } = useLoad<AppInfo | null>(readInfo, null, { onError: "show" });
  const lang: Lang = info?.lang ?? "it";

  useSubscription(onCloseRequested, setClosing);
  // A project chosen in the tray menu (D-145): its page on Projects.
  useSubscription(onOpenProject, (id) => {
    setPage("projects");
    projectTabs.show(id);
  });

  /** Ctrl+P (⌘P on macOS), as the editor's Quick Open: Projects, its box. */
  const [quickOpen, setQuickOpen] = useState(false);
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.shiftKey || e.altKey || e.key.toLowerCase() !== "p") return;
      // Ctrl+P in a terminal is the shell's (the previous command).
      if (e.target instanceof Element && e.target.closest(".xterm")) return;
      e.preventDefault();
      setPage("projects");
      setQuickOpen(true);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const all = pages();
  const current = all.find((p) => p.id === page) ?? all[0];
  const [panelOpen, setPanelOpen] = useState(false);
  const [panelHeight, setPanelHeight] = useStoredSize("panel", 240, 120, 600);
  // The app's terminal on every system (D-109, D-148): read once the app
  // answered.
  const terminals = useTerminals(tauriPty, info !== null);
  const [terminalHeight, setTerminalHeight] = useStoredSize("terminal", 280, 120, 800);

  return (
    // The key re-renders every text when the language changes.
    <div key={lang} className="flex h-full flex-col bg-vsc-activitybar">
      <a
        href="#content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-1 focus:left-14 focus:z-50 focus:bg-vsc-button focus:px-2 focus:text-vsc-button-fg"
      >
        {t("Vai al contenuto")}
      </a>
      <TitleBar title={current?.label} />
      <Banner info={info} error={error} onRetry={retryInfo} />
      {/* VSCodium's floating parts: 4px apart on the workbench color, the
          activity bar on the window's edge. */}
      <div className="flex min-h-0 flex-1 gap-vsc-size40 pr-vsc-size40 pb-vsc-size40">
        <ActivityBar
          pages={all}
          page={page}
          onSelect={setPage}
          badges={{ projects: { count: live, label: t("In esecuzione: {0}", String(live)) } }}
        />
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          {/* The lazy pages' fallback: the Guide's, as before. */}
          <Suspense fallback={<div className="flex-1 p-5 text-vsc-desc">{t("Caricamento…")}</div>}>
            {info?.firstRun ? (
              <ComponentsPage setup={setup} onDone={loadInfo} platform={info.platform} onSettings={loadInfo} />
            ) : page === "projects" ? (
              <ProjectsPage
                tabs={projectTabs}
                status={editor}
                agents={agents}
                onChanged={refreshEditor}
                runs={runs}
                logs={logs}
                loadLog={loadLog}
                forget={forget}
                onGuide={openGuide}
                onSettings={() => {
                  setPage("settings");
                }}
                quickOpen={quickOpen}
                onQuickOpen={setQuickOpen}
              />
            ) : page === "agents" ? (
              <AgentsPage onGuide={openGuide} />
            ) : page === "models" ? (
              <ModelsPage onGuide={openGuide} />
            ) : page === "components" ? (
              <ToolsPage setup={setup} tabs={toolTabs} onGuide={openGuide} />
            ) : page === "databases" ? (
              <DatabasesPage onGuide={openGuide} />
            ) : page === "servers" ? (
              <ServersPage onGuide={openGuide} />
            ) : page === "terminals" ? (
              <TerminalsPage
                onTools={() => {
                  setPage("components");
                }}
                onGuide={openGuide}
              />
            ) : page === "guide" ? (
              <GuidePage
                key={guideTopic}
                info={info}
                initialTopic={guideTopic}
                onNavigate={(target) => {
                  // The wizard is a tab of Projects.
                  if (target === "new") projectTabs.show(WIZARD_TAB);
                  setPage(target === "new" ? "projects" : target);
                }}
              />
            ) : page === "settings" ? (
              <SettingsPage
                onSaved={loadInfo}
                editorRunning={editor?.running ?? false}
                info={info}
                onGuide={openGuide}
              />
            ) : (
              <EjectPage onGuide={openGuide} />
            )}
          </Suspense>
          {terminals.shown && (
            <>
              <Sash
                vertical={false}
                label={t("Ridimensiona i terminali")}
                size={terminalHeight}
                min={120}
                max={800}
                onDrag={(d) => {
                  setTerminalHeight((h) => h - d);
                }}
                onReset={() => {
                  setTerminalHeight(280);
                }}
              />
              <Suspense fallback={<div style={{ height: terminalHeight }} className="shrink-0 bg-vsc-panel" />}>
                <TerminalPanel terminals={terminals} backend={tauriPty} height={terminalHeight} />
              </Suspense>
            </>
          )}
          {panelOpen && (
            <>
              <Sash
                vertical={false}
                label={t("Ridimensiona il pannello")}
                size={panelHeight}
                min={120}
                max={600}
                onDrag={(d) => {
                  setPanelHeight((h) => h - d);
                }}
                onReset={() => {
                  setPanelHeight(240);
                }}
              />
              <ActivityPanel
                height={panelHeight}
                runs={runs}
                running={running}
                logs={logs}
                loadLog={loadLog}
                onStop={runStop}
                onForget={forget}
                setup={setup}
                onClose={() => {
                  setPanelOpen(false);
                }}
              />
            </>
          )}
        </div>
      </div>
      <StatusBar
        info={info}
        editor={editor}
        setup={setup}
        runs={runs}
        running={running}
        panelOpen={panelOpen}
        onPanel={() => {
          setPanelOpen(!panelOpen);
        }}
        terminals={terminals}
      />
      {closing && (
        <CloseDialog
          open={closing}
          onCancel={() => {
            setClosing(null);
          }}
        />
      )}
    </div>
  );
}

/** Pages at the bottom of the bar, as VSCodium's Accounts and Manage. */
const BOTTOM: PageId[] = ["guide", "settings", "eject"];

/** VSCodium's activity bar (modern layout): on the window's left edge, 36px
 * items 8px apart, the active one and the hovered one on a rounded square
 * 4px smaller, a badge with what runs, the kit's own pages at the bottom.
 * Arrow keys move between items, as in the editor. */
function ActivityBar({
  pages: list,
  page,
  onSelect,
  badges,
}: {
  pages: Page[];
  page: PageId;
  onSelect: (id: PageId) => void;
  badges: Partial<Record<PageId, { count: number; label: string }>>;
}) {
  const move = (e: KeyboardEvent<HTMLElement>) => {
    const all = Array.from(e.currentTarget.querySelectorAll<HTMLButtonElement>("button"));
    const at = all.findIndex((b) => b === document.activeElement);
    let next = -1;
    if (e.key === "ArrowDown" && at >= 0) next = (at + 1) % all.length;
    else if (e.key === "ArrowUp" && at >= 0) next = (at - 1 + all.length) % all.length;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = all.length - 1;
    if (next < 0) return;
    e.preventDefault();
    all[next]?.focus();
  };
  const item = ({ id, label, icon: Glyph }: Page) => {
    const active = id === page;
    const badge = badges[id];
    const shown = badge && badge.count > 0 ? badge : null;
    // A lazy page's chunk, read ahead; a failure shows when the page opens.
    const preload = () => {
      PRELOAD[id]?.().catch(() => undefined);
    };
    return (
      <button
        key={id}
        type="button"
        title={shown ? `${label} - ${shown.label}` : label}
        aria-label={label}
        aria-current={active ? "page" : undefined}
        onPointerEnter={preload}
        onFocus={preload}
        onClick={() => {
          onSelect(id);
        }}
        className={[
          "group relative flex size-vsc-activitybar-action shrink-0 items-center justify-center focus-visible:outline-none",
          active
            ? "text-vsc-activitybar-item-active-fg"
            : "text-vsc-activitybar-inactive hover:text-vsc-activitybar-item-hover-fg",
        ].join(" ")}
      >
        {/* No mark on the left edge: the modern UI drops the
            activityBar.activeBorder. */}
        <span
          className={[
            "flex size-[calc(var(--vsc-size-activitybar-action)-var(--vscode-spacing-size40))] items-center justify-center rounded-vsc-small group-focus-visible:outline group-focus-visible:outline-1 group-focus-visible:outline-vsc-focus",
            active ? "bg-vsc-activitybar-item-active" : "group-hover:bg-vsc-activitybar-item-hover",
          ].join(" ")}
        >
          {/* The workbench's 24px codicon (.action-label.codicon). */}
          <Glyph size="var(--vsc-size-activitybar-icon)" aria-hidden="true" />
        </span>
        {shown && (
          // Not vscode-badge: its activity-bar-counter keeps an 18px minimum
          // and a 1px border, where the editor's badge is 16px high. 3px
          // from the right and 18px from the top (2px from the bottom), as
          // the modern UI's .badge-content.
          <span
            aria-label={shown.label}
            className="pointer-events-none absolute right-[calc(var(--vscode-spacing-size20)+var(--vscode-strokeThickness))] bottom-vsc-size20 flex h-vsc-size160 min-w-vsc-size160 items-center justify-center rounded-vsc-circle bg-vsc-activitybar-badge px-vsc-size40 text-vsc-label3 leading-none font-vsc-regular text-vsc-activitybar-badge-fg"
          >
            {shown.count > 99 ? "99+" : shown.count}
          </span>
        )}
      </button>
    );
  };
  return (
    <nav
      aria-label={t("Pagine")}
      onKeyDown={move}
      // 3px above and below, as VSCodium's composite bar and global actions.
      className="flex w-[calc(var(--vsc-size-activitybar)+var(--vscode-spacing-size80))] shrink-0 flex-col gap-vsc-activitybar-gap rounded-r-vsc-medium bg-vsc-activitybar py-[calc(var(--vscode-spacing-size20)+var(--vscode-strokeThickness))] pr-vsc-size20 pl-vsc-size60"
    >
      {list.filter((p) => !BOTTOM.includes(p.id)).map(item)}
      <span className="flex-1" />
      {list.filter((p) => BOTTOM.includes(p.id)).map(item)}
    </nav>
  );
}

/** Warnings about the kit itself, as VSCodium's banner: one line each
 * over the whole window. */
function Banner({
  info,
  error,
  onRetry,
}: {
  info: AppInfo | null;
  error: string | null;
  onRetry: () => Promise<unknown>;
}) {
  const notes: { tone: "warning"; text: string }[] = [];
  if (info?.movedFrom) {
    notes.push({
      tone: "warning",
      text: t("Il kit è stato spostato: prima era in {0}, ora in {1}.", info.movedFrom, info.root),
    });
  }
  if (info && info.otherKits.length > 0) {
    notes.push({
      tone: "warning",
      text: t(
        "Su questo PC sono aperti programmi di un'altra copia del kit ({0}). Le due copie non condividono nulla: progetti, accessi e impostazioni restano separati.",
        info.otherKits.join(", "),
      ),
    });
  }
  if (info?.watcherUnsafeFolder) {
    notes.push({
      tone: "warning",
      text: t(
        "La cartella «{0}» nel percorso del kit impedisce all'editor di accorgersi dei file cambiati fuori dall'editor (per esempio dagli agenti): rinominala in modo che non inizi con una cifra, «x» o «u», oppure sposta il kit.",
        info.watcherUnsafeFolder,
      ),
    });
  }
  if (!error && notes.length === 0) return null;
  return (
    <div className="mx-1 mb-1 space-y-1 rounded-lg border border-vsc-surface-border bg-vsc-widget px-3 py-1.5">
      <LoadError error={error && t("Impossibile leggere lo stato del kit: {0}", error)} onRetry={onRetry} />
      {notes.map((n) => (
        <Notice key={n.text} tone={n.tone}>
          {n.text}
        </Notice>
      ))}
    </div>
  );
}

/** A status bar entry, as VSCodium's label: the bar's height, 5px beside
 * the text and 3px apart, rounded, a hover and a pressed state; a button
 * when it acts. `last` is the right edge's item, 6px beside the text and 2px
 * from the edge. `tone` paints it as the editor's error items. */
function StatusItem({
  children,
  title,
  onClick,
  className = "",
  pressed,
  tone,
  last = false,
}: {
  children: ReactNode;
  title?: string;
  onClick?: () => void;
  className?: string;
  pressed?: boolean;
  tone?: "error" | undefined;
  last?: boolean;
}) {
  const color = tone === "error" ? "bg-vsc-statusbar-error text-vsc-statusbar-error-fg" : "";
  const space = last ? "mr-vsc-size20 px-vsc-size60" : "mx-vsc-statusbar-item-margin px-vsc-statusbar-item-padding";
  const look = `flex h-full items-center gap-vsc-size40 rounded-vsc-small whitespace-nowrap ${space} ${color} ${className}`;
  return onClick ? (
    <button
      type="button"
      title={title}
      aria-pressed={pressed}
      onClick={onClick}
      className={[
        look,
        "focus-visible:outline-vsc-statusbar-fg",
        tone ? "hover:brightness-110" : pressed ? "bg-vsc-statusbar-active" : "hover:bg-vsc-statusbar-hover",
      ].join(" ")}
    >
      {children}
    </button>
  ) : (
    <span title={title} className={look}>
      {children}
    </span>
  );
}

/** The version, the bar's last item: a click looks for an update, as
 * «Cerca aggiornamenti» in Settings › Updates. While it looks the item
 * spins; then an icon tells what it found (up to date, a newer version, a
 * failed check), and the hover and screen readers say it in full. */
function VersionItem({ version }: { version: string }) {
  const [checking, setChecking] = useState(false);
  const [found, setFound] = useState<UpdateStatus | null>(null);
  const check = () => {
    if (checking) return;
    setChecking(true);
    updateCheck().then(
      (st) => {
        setChecking(false);
        setFound(st);
      },
      (e: unknown) => {
        setChecking(false);
        setFound({ current: version, latest: null, staged: null, checking: false, error: String(e), manual: false });
      },
    );
  };
  const text = found ? updateText(found) : null;
  const newer = found ? updateFound(found) : null;
  const label = checking ? t("Ricerca degli aggiornamenti…") : t("Cerca aggiornamenti");
  return (
    <StatusItem title={text ? `${text}\n${label}` : label} onClick={check} last>
      <span className="sr-only">{label}</span>
      {checking ? (
        <Icon name="loading" spin size={14} className="[--vscode-icon-foreground:currentColor]" />
      ) : found?.error ? (
        <Codicon name="warning" size={14} />
      ) : newer ? (
        <Codicon name="arrow-circle-up" size={14} />
      ) : found ? (
        <Codicon name="check" size={14} />
      ) : null}
      <span>v{version}</span>
      {newer && !checking && <span aria-hidden="true">→ v{newer}</span>}
      <span role="status" className="sr-only">
        {checking ? "" : (text ?? "")}
      </span>
    </StatusItem>
  );
}

/** VSCodium's status bar on the brand's band: the kit as the remote item,
 * what runs and what failed (the one item that opens and closes the
 * Activity panel, as the editor's Problems item), the installs; on the
 * right the terminals, the editor and the kit. */
function StatusBar({
  info,
  editor,
  setup,
  runs,
  running,
  panelOpen,
  onPanel,
  terminals,
}: {
  info: AppInfo | null;
  editor: EditorStatus | null;
  setup: SetupSnapshot | null;
  runs: Record<string, RunInfo>;
  /** What runs in the kit, counted by the Activity item. */
  running: Running;
  panelOpen: boolean;
  onPanel: () => void;
  /** The app's terminal sessions (every system, D-148). */
  terminals: Terminals;
}) {
  // The panel's sessions only: a detached one is in its own window, and with
  // only those the toggle would show and hide nothing.
  const sessions = terminals.docked.length;
  const terminalsLabel = terminals.shown ? t("Nascondi i terminali") : t("Mostra i terminali");
  const installing = setupStatusText(setup);
  const live = running.count;
  const errors = Object.values(runs).filter((r) => r.state === "error").length;
  return (
    <footer
      aria-label={t("Barra di stato")}
      // VSCodium's 22px and the 6px the floating parts add below, 2px above
      // and below the items.
      className="flex h-[calc(var(--vsc-size-statusbar)+var(--vscode-spacing-size60))] shrink-0 items-center bg-vsc-statusbar px-vsc-size60 py-vsc-size20 text-vsc-body-small leading-none text-vsc-statusbar-fg"
    >
      {/* First, as VSCodium's remote indicator: 4px from the bar's edge,
          4px and 6px beside the text. */}
      <span
        role="img"
        aria-label="Atollo"
        title={info ? `Atollo - ${info.root}` : "Atollo"}
        className="ml-vsc-size40 flex h-full items-center gap-vsc-size60 rounded-vsc-small bg-vsc-statusbar-remote pr-vsc-size60 pl-vsc-size40 font-semibold tracking-wide text-vsc-statusbar-remote-fg"
      >
        <Codicon name="remote" size={16} />
        <span aria-hidden="true">Atollo</span>
      </span>
      <StatusItem
        title={t("Attività: progetti avviati, database, modelli locali, server web, installazioni")}
        onClick={onPanel}
        pressed={panelOpen}
        tone={errors > 0 ? "error" : undefined}
        // 4px more beside a colored item, as VSCodium's.
        className="ml-[calc(var(--vscode-spacing-size40)+var(--vsc-size-statusbar-item-margin))]"
      >
        <span className="sr-only">{t("Attività")}</span>
        <Codicon name="play" size={14} />
        <span aria-label={t("In esecuzione: {0}", String(live))}>{live}</span>
        <Codicon name="error" size={14} className="ml-1" />
        <span aria-label={t("Errori: {0}", String(errors))}>{errors}</span>
      </StatusItem>
      {installing && (
        <StatusItem>
          {/* The item's color, as the editor's status-bar codicons. */}
          <Icon name="loading" spin size={14} className="[--vscode-icon-foreground:currentColor]" />
          <span role="status">{installing}</span>
        </StatusItem>
      )}
      <span className="flex-1" />
      {sessions > 0 && (
        <StatusItem
          title={terminalsLabel}
          onClick={() => {
            terminals.setShown(!terminals.shown);
          }}
          pressed={terminals.shown}
          last={!info}
        >
          <span className="sr-only">{terminalsLabel}</span>
          <Codicon name="terminal" size={14} />
          <span aria-label={t("Terminali aperti: {0}", String(sessions))}>{sessions}</span>
        </StatusItem>
      )}
      {info && (
        <>
          {editor?.running && (
            <StatusItem title={t("Server dell'editor su 127.0.0.1:{0}", String(editor.port))}>
              <Codicon name="code" size={14} />
              {t("Editor attivo")}
            </StatusItem>
          )}
          <StatusItem>{info.platform}</StatusItem>
          <VersionItem version={info.version} />
        </>
      )}
    </footer>
  );
}
