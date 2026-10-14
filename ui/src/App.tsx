// Main window, laid out like VSCodium: activity bar, content, the Activity
// panel and the status bar. Six pages (D-76): what concerns one project is
// on its page, what concerns the kit on the kit's pages.
// Texts go through t() (i18n.ts) in the language of the settings or of
// Windows, sent by app_info.
import { useCallback, useEffect, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  VscRemote,
  VscDebugDisconnect,
  VscDatabase,
  VscExtensions,
  VscFolderLibrary,
  VscHubot,
  VscSettingsGear,
  VscWarning,
  VscCode,
  VscLoading,
} from "react-icons/vsc";
import {
  appInfo,
  browserPreview,
  editorStatus,
  onEditorStatus,
  onCloseRequested,
  type AppInfo,
  type EditorStatus,
  type OpenNow,
  type SetupSnapshot,
} from "./api";
import { ProjectsPage, useRuns } from "./projects";
import { ActivityPanel } from "./activity";
import { runStop } from "./api";
import { TitleBar } from "./titlebar";
import { ComponentsPage } from "./components";
import { AgentsPage, useAgents } from "./agents";
import { setupStatusText, useSetup } from "./setup";
import { SettingsPage } from "./settings";
import { CloseDialog, EjectPage } from "./eject";
import { setLang, t, type Lang } from "./i18n";
import { DatabasesPage } from "./databases";
import { Badge } from "./tags";

type PageId = "projects" | "agents" | "databases" | "components" | "settings" | "eject";

interface Page {
  id: PageId;
  label: string;
  icon: IconType;
}

function pages(): Page[] {
  return [
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
  /** Bumped when the Projects page is chosen again: back to the list. */
  const [projectsKey, setProjectsKey] = useState(0);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [lang, setLangState] = useState<Lang>("it");
  const [error, setError] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorStatus | null>(null);
  const [closing, setClosing] = useState<OpenNow | null>(null);
  const setup = useSetup();
  const { agents } = useAgents();
  const { runs, logs, loadLog, forget } = useRuns();

  const refreshEditor = useCallback(() => {
    editorStatus().then(setEditor, () => {
      setEditor(null);
    });
  }, []);
  useEffect(() => {
    refreshEditor();
    const timer = window.setInterval(refreshEditor, 5000);
    let stop: (() => void) | null = null;
    let gone = false;
    void onEditorStatus(setEditor).then((f) => {
      if (gone) f();
      else stop = f;
    });
    return () => {
      gone = true;
      stop?.();
      window.clearInterval(timer);
    };
  }, [refreshEditor]);

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

  useEffect(() => {
    let stop: (() => void) | null = null;
    let gone = false;
    void onCloseRequested(setClosing).then((f) => {
      if (gone) f();
      else stop = f;
    });
    return () => {
      gone = true;
      stop?.();
    };
  }, []);

  const all = pages();
  const current = all.find((p) => p.id === page) ?? all[0];

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
      {/* VSCodium's floating parts: 4px apart on the workbench color. */}
      <div className="flex min-h-0 flex-1 gap-1 px-1 pb-1">
        <ActivityBar
          pages={all}
          page={page}
          onSelect={(id) => {
            if (id === "projects") setProjectsKey((k) => k + 1);
            setPage(id);
          }}
        />
        <main
          className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-vsc-surface-border bg-vsc-bg"
          aria-labelledby="page-title"
        >
          <header className="flex h-8 shrink-0 items-center gap-2 border-b border-vsc-border px-3">
            <h1 id="page-title" className="text-[12px] font-semibold text-vsc-sidebar-title">
              {current?.label}
            </h1>
            {browserPreview && <Badge className="ml-auto">{t("Anteprima nel browser: dati di esempio")}</Badge>}
          </header>
          {/* Never scrolls itself: each page scrolls in its own box, as the
              editor's views, so nothing slides under another. */}
          <div id="content" tabIndex={-1} className="flex min-h-0 flex-1 flex-col outline-none">
            {!info?.firstRun && page === "projects" ? (
              <>
                <div className="shrink-0 space-y-2 px-3 pt-3 empty:hidden">
                  <KitNotices info={info} error={error} />
                </div>
                <div className="relative min-h-0 flex-1">
                  <ProjectsPage
                    key={projectsKey}
                    status={editor}
                    agents={agents}
                    onChanged={refreshEditor}
                    runs={runs}
                    logs={logs}
                    loadLog={loadLog}
                    forget={forget}
                  />
                </div>
              </>
            ) : (
              <div className="min-h-0 flex-1 overflow-auto p-3">
                {info?.firstRun ? (
                  <ComponentsPage setup={setup} firstRun onDone={loadInfo} />
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
              </div>
            )}
          </div>
        </main>
      </div>
      <div className="px-1 pb-1">
        <ActivityPanel
          runs={runs}
          logs={logs}
          loadLog={loadLog}
          onStop={(key) => {
            void runStop(key);
          }}
          onForget={forget}
          setup={setup}
        />
      </div>
      <StatusBar info={info} editor={editor} setup={setup} />
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

function ActivityBar({ pages: list, page, onSelect }: { pages: Page[]; page: PageId; onSelect: (id: PageId) => void }) {
  return (
    <nav
      aria-label={t("Pagine")}
      className="flex w-11 shrink-0 flex-col items-center rounded-lg border border-vsc-surface-border bg-vsc-activitybar py-1"
    >
      {list.map(({ id, label, icon: Icon }) => {
        const active = id === page;
        return (
          <button
            key={id}
            type="button"
            title={label}
            aria-label={label}
            aria-current={active ? "page" : undefined}
            onClick={() => {
              onSelect(id);
            }}
            className={[
              // 36px items, a 32px rounded mark on the active one.
              "flex size-9 items-center justify-center",
              active ? "text-vsc-activitybar-fg" : "text-vsc-activitybar-inactive hover:text-vsc-activitybar-fg",
            ].join(" ")}
          >
            <span
              className={`flex size-8 items-center justify-center rounded ${active ? "bg-vsc-activitybar-item-active" : "hover:bg-vsc-activitybar-item-hover"}`}
            >
              <Icon size={24} aria-hidden="true" />
            </span>
          </button>
        );
      })}
    </nav>
  );
}

/** Warnings about the kit itself, above the projects (were on the Home). */
function KitNotices({ info, error }: { info: AppInfo | null; error: string | null }) {
  if (error) {
    return (
      <Notice icon={VscWarning} tone="error">
        {t("Impossibile leggere lo stato del kit: {0}", error)}
      </Notice>
    );
  }
  if (!info) return null;
  return (
    <>
      {info.movedFrom && (
        <Notice icon={VscWarning} tone="warning">
          {t("Il kit è stato spostato: prima era in {0}, ora in {1}.", info.movedFrom, info.root)}
        </Notice>
      )}
      {info.otherKits.length > 0 && (
        <Notice icon={VscWarning} tone="warning">
          {t(
            "Su questo PC sono aperti programmi di un'altra copia del kit ({0}). Le due copie non condividono nulla: progetti, accessi e impostazioni restano separati.",
            info.otherKits.join(", "),
          )}
        </Notice>
      )}
      {info.watcherUnsafeFolder && (
        <Notice icon={VscWarning} tone="warning">
          {t(
            "La cartella «{0}» nel percorso del kit impedisce all'editor di accorgersi dei file cambiati fuori dall'editor (per esempio dagli agenti): rinominala in modo che non inizi con una cifra, «x» o «u», oppure sposta il kit.",
            info.watcherUnsafeFolder,
          )}
        </Notice>
      )}
    </>
  );
}

function Notice({
  icon: Icon,
  tone,
  children,
}: {
  icon: IconType;
  tone: "error" | "warning" | "success";
  children: ReactNode;
}) {
  const color = { error: "text-vsc-error", warning: "text-vsc-warning", success: "text-vsc-success" }[tone];
  return (
    <div role={tone === "error" ? "alert" : "status"} className="flex items-start gap-2">
      <Icon className={`mt-0.5 shrink-0 ${color}`} size={16} aria-hidden="true" />
      <span>{children}</span>
    </div>
  );
}

function StatusBar({
  info,
  editor,
  setup,
}: {
  info: AppInfo | null;
  editor: EditorStatus | null;
  setup: SetupSnapshot | null;
}) {
  const installing = setupStatusText(setup);
  return (
    <footer
      aria-label={t("Barra di stato")}
      className="flex h-7 shrink-0 items-center bg-vsc-statusbar px-1.5 pb-1 text-[12px] leading-none text-vsc-statusbar-fg"
    >
      {/* First, as VSCodium's remote indicator: the icon only, the name is
          in the tooltip. Items are 24px high and rounded, as the editor's. */}
      <span
        role="img"
        aria-label="Atollo"
        title="Atollo"
        className="flex h-6 items-center rounded bg-vsc-statusbar-remote px-1.5 text-vsc-statusbar-remote-fg"
      >
        <VscRemote size={16} aria-hidden="true" />
      </span>
      {info && (
        <>
          <span className="flex h-6 items-center rounded px-1.5">{info.platform}</span>
          {editor?.running && (
            <span
              className="flex h-6 items-center gap-1 rounded px-1.5"
              title={t("Server dell'editor su 127.0.0.1:{0}", String(editor.port))}
            >
              <VscCode size={14} aria-hidden="true" />
              {t("Editor attivo")}
            </span>
          )}
          {installing && (
            <span className="flex h-6 items-center gap-1 rounded px-1.5" role="status">
              <VscLoading size={14} aria-hidden="true" className="animate-spin" />
              {installing}
            </span>
          )}
          <span className="ml-auto flex h-6 items-center rounded px-1.5">v{info.version}</span>
        </>
      )}
    </footer>
  );
}
