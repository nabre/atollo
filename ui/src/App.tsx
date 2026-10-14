// Main window, laid out like VSCodium: activity bar, content and status bar.
// Texts go through t() (i18n.ts) in the language of the settings or of
// Windows, sent by app_info.
import { useCallback, useEffect, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  VscDebugDisconnect,
  VscExtensions,
  VscFolderLibrary,
  VscGithub,
  VscHome,
  VscHubot,
  VscSettingsGear,
  VscWarning,
  VscCheck,
  VscRemote,
  VscCode,
  VscLoading,
} from "react-icons/vsc";
import {
  appInfo,
  browserPreview,
  editorStatus,
  onEditorStatus,
  onCloseRequested,
  type AgentInfo,
  type AppInfo,
  type EditorStatus,
  type OpenNow,
  type SetupSnapshot,
} from "./api";
import { ProjectsPage, RecentProjects } from "./projects";
import { GithubPage } from "./github";
import { TitleBar } from "./titlebar";
import { ComponentsPage } from "./components";
import { AgentsPage, useAgents } from "./agents";
import { SetupProgress, setupStatusText, useSetup } from "./setup";
import { SettingsPage } from "./settings";
import { CloseDialog, EjectPage } from "./eject";
import { setLang, t, type Lang } from "./i18n";
import logo from "./assets/logo-128.png";

type PageId = "home" | "projects" | "github" | "agents" | "components" | "settings" | "eject";

interface Page {
  id: PageId;
  label: string;
  icon: IconType;
}

function pages(): Page[] {
  return [
    { id: "home", label: t("Home"), icon: VscHome },
    { id: "projects", label: t("Progetti"), icon: VscFolderLibrary },
    { id: "github", label: "GitHub", icon: VscGithub },
    { id: "agents", label: t("Agenti"), icon: VscHubot },
    { id: "components", label: t("Linguaggi e strumenti"), icon: VscExtensions },
    { id: "settings", label: t("Impostazioni"), icon: VscSettingsGear },
    { id: "eject", label: t("Rimuovi l'unità"), icon: VscDebugDisconnect },
  ];
}

export function App() {
  const [page, setPage] = useState<PageId>("home");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [lang, setLangState] = useState<Lang>("it");
  const [error, setError] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorStatus | null>(null);
  const [closing, setClosing] = useState<OpenNow | null>(null);
  const setup = useSetup();
  const { agents } = useAgents();

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
    <div key={lang} className="flex h-full flex-col">
      <a
        href="#content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-1 focus:left-14 focus:z-50 focus:bg-vsc-button focus:px-2 focus:text-vsc-button-fg"
      >
        {t("Vai al contenuto")}
      </a>
      <TitleBar title={current?.label} />
      <div className="flex min-h-0 flex-1">
        <ActivityBar pages={all} page={page} onSelect={setPage} />
        <main className="flex min-w-0 flex-1 flex-col bg-vsc-bg" aria-labelledby="page-title">
          <header className="flex h-9 items-center gap-3 border-b border-vsc-border px-5">
            <h1 id="page-title" className="text-[11px] font-normal tracking-wide uppercase text-vsc-desc">
              {current?.label}
            </h1>
            {browserPreview && (
              <span className="ml-auto rounded-sm bg-vsc-badge px-1.5 text-[11px] text-vsc-fg">
                {t("Anteprima nel browser: dati di esempio")}
              </span>
            )}
          </header>
          <div id="content" tabIndex={-1} className="relative min-h-0 flex-1 overflow-auto p-5 outline-none">
            {page === "home" && info?.firstRun ? (
              <ComponentsPage setup={setup} firstRun onDone={loadInfo} />
            ) : page === "home" ? (
              <Home
                info={info}
                error={error}
                editor={editor}
                setup={setup}
                agents={agents}
                onChanged={refreshEditor}
                onAllProjects={() => {
                  setPage("projects");
                }}
              />
            ) : page === "projects" ? (
              <ProjectsPage status={editor} agents={agents} onChanged={refreshEditor} />
            ) : page === "github" ? (
              <GithubPage editor={editor} onChanged={refreshEditor} />
            ) : page === "agents" ? (
              <AgentsPage />
            ) : page === "components" ? (
              <ComponentsPage setup={setup} />
            ) : page === "settings" ? (
              <SettingsPage onSaved={loadInfo} editorRunning={editor?.running ?? false} />
            ) : (
              <EjectPage />
            )}
          </div>
        </main>
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
    <nav aria-label={t("Pagine")} className="flex w-12 flex-col border-r border-vsc-border bg-vsc-activitybar">
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
              "flex h-12 w-12 items-center justify-center border-l-2",
              active
                ? "border-vsc-activitybar-border text-vsc-activitybar-fg"
                : "border-transparent text-vsc-activitybar-inactive hover:text-vsc-activitybar-fg",
            ].join(" ")}
          >
            <Icon size={24} aria-hidden="true" />
          </button>
        );
      })}
    </nav>
  );
}

function Home({
  info,
  error,
  editor,
  setup,
  agents,
  onChanged,
  onAllProjects,
}: {
  info: AppInfo | null;
  error: string | null;
  editor: EditorStatus | null;
  setup: SetupSnapshot | null;
  agents: AgentInfo[] | null;
  onChanged: () => void;
  onAllProjects: () => void;
}) {
  if (error) {
    return (
      <Notice icon={VscWarning} tone="error">
        {t("Impossibile leggere lo stato del kit: {0}", error)}
      </Notice>
    );
  }
  if (!info) {
    return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  }
  return (
    <div className="max-w-3xl space-y-5">
      <div className="flex items-center gap-4">
        <img src={logo} alt="" width={56} height={56} draggable={false} />
        <div>
          <p className="text-xl font-semibold text-vsc-fg">Atollo</p>
          <p className="text-vsc-desc">
            {t("Ambiente di sviluppo portabile: linguaggi, agenti ed editor in una cartella.")}
          </p>
        </div>
      </div>
      {info.movedFrom ? (
        <Notice icon={VscWarning} tone="warning">
          {t("Il kit è stato spostato: prima era in {0}, ora in {1}.", info.movedFrom, info.root)}
        </Notice>
      ) : (
        <Notice icon={VscCheck} tone="success">
          {t("Atollo è pronto. Tutto ciò che scarica e salva resta in questa cartella.")}
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
      <RecentProjects
        status={editor}
        setupRunning={setup?.running ?? false}
        agents={agents}
        onChanged={onChanged}
        onAll={onAllProjects}
      />
      {setup && setup.steps.length > 0 && <SetupProgress snapshot={setup} />}
      <section aria-labelledby="where" className="rounded border border-vsc-border bg-vsc-sidebar">
        <h2 id="where" className="border-b border-vsc-border px-4 py-2 text-[13px] font-semibold">
          {t("Dove si trova il kit")}
        </h2>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1.5 px-4 py-3">
          <Row label={t("Cartella del kit")} value={info.root} />
          <Row label={t("Progetti")} value={info.projects} />
          <Row label={t("Dati e programmi")} value={info.data} />
          <Row label={t("Piattaforma")} value={info.platform} />
          <Row label="WebView2" value={info.webview} />
        </dl>
      </section>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt className="text-vsc-desc">{label}</dt>
      <dd className="m-0 font-mono text-[12px] break-all select-text">{value}</dd>
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
      className="flex h-[22px] items-center gap-3 border-t border-vsc-border bg-vsc-statusbar px-2 text-[12px] text-vsc-statusbar-fg"
    >
      <span className="flex h-full items-center gap-1 bg-vsc-button px-2 text-vsc-button-fg">
        <VscRemote size={14} aria-hidden="true" />
        Atollo
      </span>
      {info && (
        <>
          <span>{info.platform}</span>
          {editor?.running && (
            <span
              className="flex items-center gap-1"
              title={t("Server dell'editor su 127.0.0.1:{0}", String(editor.port))}
            >
              <VscCode size={14} aria-hidden="true" />
              {t("Editor attivo")}
            </span>
          )}
          {installing && (
            <span className="flex items-center gap-1" role="status">
              <VscLoading size={14} aria-hidden="true" className="animate-spin" />
              {installing}
            </span>
          )}
          <span className="ml-auto">v{info.version}</span>
        </>
      )}
    </footer>
  );
}
