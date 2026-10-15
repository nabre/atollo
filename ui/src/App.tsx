// Main window, laid out like VSCodium: the banner, the activity bar, the
// page (side bar and editor, workbench/), the Activity panel and the
// status bar. Seven pages (D-76, D-83): the new-project wizard, then what
// concerns one project on its page, what concerns the kit on the kit's pages.
// Texts go through t() (i18n.ts) in the language of the settings or of
// Windows, sent by app_info.
import { useCallback, useEffect, useState, type KeyboardEvent, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  VscCode,
  VscDatabase,
  VscDebugDisconnect,
  VscError,
  VscExtensions,
  VscFolderLibrary,
  VscHubot,
  VscLayoutPanel,
  VscPlay,
  VscRemote,
  VscSettingsGear,
  VscWand,
} from "react-icons/vsc";
import {
  appInfo,
  editorStatus,
  onEditorStatus,
  onCloseRequested,
  runStop,
  type AppInfo,
  type EditorStatus,
  type OpenNow,
  type RunInfo,
  type SetupSnapshot,
} from "./api";
import { alive, ProjectsPage, useRuns } from "./projects";
import { ActivityPanel } from "./activity";
import { TitleBar } from "./titlebar";
import { ComponentsPage } from "./components";
import { AgentsPage, useAgents } from "./agents";
import { setupStatusText, useSetup } from "./setup";
import { SettingsPage } from "./settings";
import { CloseDialog, EjectPage } from "./eject";
import { setLang, t, type Lang } from "./i18n";
import { DatabasesPage } from "./databases";
import { NewProjectPage } from "./wizard";
import { Icon, Notice, Sash, useStoredSize, useTabs } from "./workbench";
import { useLoad, useSubscription } from "./hooks";

type PageId = "new" | "projects" | "agents" | "databases" | "components" | "settings" | "eject";

interface Page {
  id: PageId;
  label: string;
  icon: IconType;
}

function pages(): Page[] {
  return [
    { id: "new", label: t("Nuovo progetto"), icon: VscWand },
    { id: "projects", label: t("Progetti"), icon: VscFolderLibrary },
    { id: "agents", label: t("Agenti"), icon: VscHubot },
    { id: "databases", label: t("Database"), icon: VscDatabase },
    { id: "components", label: t("Strumenti"), icon: VscExtensions },
    { id: "settings", label: t("Impostazioni"), icon: VscSettingsGear },
    { id: "eject", label: t("Rimuovi l'unità"), icon: VscDebugDisconnect },
  ];
}

export function App() {
  const [page, setPage] = useState<PageId>("projects");
  /** The projects open in the editor, kept while other pages are shown. */
  const projectTabs = useTabs();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [lang, setLangState] = useState<Lang>("it");
  const [error, setError] = useState<string | null>(null);
  const [closing, setClosing] = useState<OpenNow | null>(null);
  const setup = useSetup();
  const { agents } = useAgents();
  const { runs, logs, loadLog, forget } = useRuns();
  const live = Object.values(runs).filter((r) => alive(r.state)).length;

  const {
    data: editor,
    setData: setEditor,
    reload: refreshEditor,
  } = useLoad<EditorStatus | null>(editorStatus, null, { onError: { fallback: null }, every: 5000 });
  useSubscription(onEditorStatus, setEditor);

  const loadInfo = useCallback(() => {
    appInfo().then(
      (i) => {
        setLang(i.lang);
        setLangState(i.lang);
        setInfo(i);
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  useEffect(loadInfo, [loadInfo]);

  useSubscription(onCloseRequested, setClosing);

  const all = pages();
  const current = all.find((p) => p.id === page) ?? all[0];
  const [panelOpen, setPanelOpen] = useState(false);
  const [panelHeight, setPanelHeight] = useStoredSize("panel", 240, 120, 600);

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
      <Banner info={info} error={error} />
      {/* VSCodium's floating parts: 4px apart on the workbench color. */}
      <div className="flex min-h-0 flex-1 gap-1 px-1 pb-1">
        <ActivityBar
          pages={all}
          page={page}
          onSelect={setPage}
          badges={{ projects: { count: live, label: t("Progetti avviati: {0}", String(live)) } }}
        />
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          {info?.firstRun ? (
            <ComponentsPage setup={setup} firstRun onDone={loadInfo} />
          ) : page === "new" ? (
            <NewProjectPage
              onOpen={(project) => {
                projectTabs.show(project);
                setPage("projects");
              }}
              onSettings={() => {
                setPage("settings");
              }}
            />
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
            />
          ) : page === "agents" ? (
            <AgentsPage />
          ) : page === "components" ? (
            <ComponentsPage setup={setup} />
          ) : page === "databases" ? (
            <DatabasesPage />
          ) : page === "settings" ? (
            <SettingsPage onSaved={loadInfo} editorRunning={editor?.running ?? false} info={info} />
          ) : (
            <EjectPage />
          )}
          {panelOpen && (
            <>
              <Sash
                vertical={false}
                label={t("Ridimensiona il pannello")}
                onDrag={(d) => {
                  setPanelHeight(panelHeight - d);
                }}
                onReset={() => {
                  setPanelHeight(240);
                }}
              />
              <ActivityPanel
                height={panelHeight}
                runs={runs}
                logs={logs}
                loadLog={loadLog}
                onStop={(key) => {
                  void runStop(key);
                }}
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
        panelOpen={panelOpen}
        onPanel={() => {
          setPanelOpen(!panelOpen);
        }}
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
const BOTTOM: PageId[] = ["settings", "eject"];

/** VSCodium's activity bar (modern layout): 36px items with a rounded mark
 * on the active one and the brand's bar on its left edge, a badge with what
 * runs, the kit's own pages at the bottom. Arrow keys move between items,
 * as in the editor. */
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
    return (
      <button
        key={id}
        type="button"
        title={shown ? `${label} - ${shown.label}` : label}
        aria-label={label}
        aria-current={active ? "page" : undefined}
        onClick={() => {
          onSelect(id);
        }}
        className={[
          "group relative flex size-9 items-center justify-center focus-visible:outline-none",
          active ? "text-vsc-activitybar-fg" : "text-vsc-activitybar-inactive hover:text-vsc-activitybar-fg",
        ].join(" ")}
      >
        {/* The brand's mark on the left edge, as the editor's
            activityBar.activeBorder. */}
        <span
          aria-hidden="true"
          className={`absolute top-2 bottom-2 -left-px w-[3px] rounded-full bg-vsc-activitybar-border transition-opacity ${active ? "opacity-100" : "opacity-0"}`}
        />
        <span
          className={[
            "flex size-8 items-center justify-center rounded group-focus-visible:outline group-focus-visible:outline-1 group-focus-visible:outline-vsc-focus",
            active ? "bg-vsc-activitybar-item-active" : "group-hover:bg-vsc-activitybar-item-hover",
          ].join(" ")}
        >
          <Glyph size={24} aria-hidden="true" />
        </span>
        {shown && (
          // Not vscode-badge: its activity-bar-counter keeps an 18px minimum
          // and a 1px border, where the editor's badge is 16px high.
          <span
            aria-label={shown.label}
            className="pointer-events-none absolute right-0.5 bottom-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-vsc-activitybar-badge px-1 text-[9px] leading-none font-semibold text-vsc-activitybar-badge-fg"
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
      className="flex w-11 shrink-0 flex-col items-center rounded-lg border border-vsc-surface-border bg-vsc-activitybar py-1"
    >
      {list.filter((p) => !BOTTOM.includes(p.id)).map(item)}
      <span className="flex-1" />
      {list.filter((p) => BOTTOM.includes(p.id)).map(item)}
    </nav>
  );
}

/** Warnings about the kit itself, as VSCodium's banner: one line each
 * over the whole window. */
function Banner({ info, error }: { info: AppInfo | null; error: string | null }) {
  const notes: { tone: "error" | "warning"; text: string }[] = [];
  if (error) notes.push({ tone: "error", text: t("Impossibile leggere lo stato del kit: {0}", error) });
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
  if (notes.length === 0) return null;
  return (
    <div className="mx-1 mb-1 space-y-1 rounded-lg border border-vsc-surface-border bg-vsc-widget px-3 py-1.5">
      {notes.map((n) => (
        <Notice key={n.text} tone={n.tone}>
          {n.text}
        </Notice>
      ))}
    </div>
  );
}

/** A status bar entry, as VSCodium's: 24px, rounded, a hover and a
 * pressed state; a button when it acts. `tone` paints it as the editor's
 * error items. */
function StatusItem({
  children,
  title,
  onClick,
  className = "",
  pressed,
  tone,
}: {
  children: ReactNode;
  title?: string;
  onClick?: () => void;
  className?: string;
  pressed?: boolean;
  tone?: "error" | undefined;
}) {
  const color = tone === "error" ? "bg-vsc-statusbar-error text-vsc-statusbar-error-fg" : "";
  const look = `flex h-6 items-center gap-1 rounded px-1.5 whitespace-nowrap ${color} ${className}`;
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

/** VSCodium's status bar on the brand's band: the kit as the remote item,
 * what runs and what failed (it opens the Activity panel, as the editor's
 * Problems item), the installs; on the right the editor, the kit and the
 * panel's toggle. */
function StatusBar({
  info,
  editor,
  setup,
  runs,
  panelOpen,
  onPanel,
}: {
  info: AppInfo | null;
  editor: EditorStatus | null;
  setup: SetupSnapshot | null;
  runs: Record<string, RunInfo>;
  panelOpen: boolean;
  onPanel: () => void;
}) {
  const installing = setupStatusText(setup);
  const list = Object.values(runs);
  const live = list.filter((r) => alive(r.state)).length;
  const errors = list.filter((r) => r.state === "error").length;
  const panelLabel = panelOpen ? t("Nascondi il pannello") : t("Mostra il pannello");
  return (
    <footer
      aria-label={t("Barra di stato")}
      className="flex h-7 shrink-0 items-center gap-0.5 bg-vsc-statusbar px-1.5 pb-1 text-[12px] leading-none text-vsc-statusbar-fg"
    >
      {/* First, as VSCodium's remote indicator. */}
      <span
        role="img"
        aria-label="Atollo"
        title={info ? `Atollo - ${info.root}` : "Atollo"}
        className="flex h-6 items-center gap-1.5 rounded bg-vsc-statusbar-remote px-2 font-semibold tracking-wide text-vsc-statusbar-remote-fg"
      >
        <VscRemote size={16} aria-hidden="true" />
        <span aria-hidden="true">Atollo</span>
      </span>
      <StatusItem
        title={t("Attività: progetti avviati, database, installazioni")}
        onClick={onPanel}
        pressed={panelOpen}
        tone={errors > 0 ? "error" : undefined}
        className="ml-1"
      >
        <span className="sr-only">{t("Attività")}</span>
        <VscPlay size={14} aria-hidden="true" />
        <span aria-label={t("Progetti avviati: {0}", String(live))}>{live}</span>
        <VscError size={14} aria-hidden="true" className="ml-1" />
        <span aria-label={t("Errori: {0}", String(errors))}>{errors}</span>
      </StatusItem>
      {installing && (
        <StatusItem onClick={onPanel}>
          {/* The item's color, as the editor's status-bar codicons. */}
          <Icon name="loading" spin size={14} className="[--vscode-icon-foreground:currentColor]" />
          <span role="status">{installing}</span>
        </StatusItem>
      )}
      <span className="flex-1" />
      {info && (
        <>
          {editor?.running && (
            <StatusItem title={t("Server dell'editor su 127.0.0.1:{0}", String(editor.port))}>
              <VscCode size={14} aria-hidden="true" />
              {t("Editor attivo")}
            </StatusItem>
          )}
          <StatusItem>{info.platform}</StatusItem>
          <StatusItem>v{info.version}</StatusItem>
        </>
      )}
      <StatusItem title={panelLabel} onClick={onPanel} pressed={panelOpen}>
        <span className="sr-only">{panelLabel}</span>
        <VscLayoutPanel size={14} aria-hidden="true" />
      </StatusItem>
    </footer>
  );
}
