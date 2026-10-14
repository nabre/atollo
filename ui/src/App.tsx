// Main window, laid out like VSCodium: activity bar, content and status bar.
// Home and Projects open the editor and the console (phase 3); the other
// pages arrive in the next phases. Texts are Italian for now; the catalogs arrive in phase 6.
import { useCallback, useEffect, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import {
  VscDebugDisconnect,
  VscExtensions,
  VscFolderLibrary,
  VscHome,
  VscHubot,
  VscSettingsGear,
  VscWarning,
  VscCheck,
  VscRemote,
  VscCode,
} from "react-icons/vsc";
import { appInfo, browserPreview, editorStatus, type AppInfo, type EditorStatus } from "./api";
import { HomeActions, ProjectsPage } from "./pages";

type PageId = "home" | "projects" | "agents" | "components" | "settings" | "eject";

interface Page {
  id: PageId;
  label: string;
  icon: IconType;
}

const HOME: Page = { id: "home", label: "Home", icon: VscHome };

const PAGES: Page[] = [
  HOME,
  { id: "projects", label: "Progetti", icon: VscFolderLibrary },
  { id: "agents", label: "Agenti", icon: VscHubot },
  { id: "components", label: "Linguaggi e strumenti", icon: VscExtensions },
  { id: "settings", label: "Impostazioni", icon: VscSettingsGear },
  { id: "eject", label: "Rimuovi l'unità", icon: VscDebugDisconnect },
];

export function App() {
  const [page, setPage] = useState<PageId>("home");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorStatus | null>(null);

  const refreshEditor = useCallback(() => {
    editorStatus().then(setEditor, () => {
      setEditor(null);
    });
  }, []);
  useEffect(() => {
    refreshEditor();
    const timer = window.setInterval(refreshEditor, 5000);
    return () => {
      window.clearInterval(timer);
    };
  }, [refreshEditor]);

  useEffect(() => {
    appInfo().then(setInfo, (e: unknown) => {
      setError(String(e));
    });
  }, []);

  const current = PAGES.find((p) => p.id === page) ?? HOME;

  return (
    <div className="flex h-full flex-col">
      <div className="flex min-h-0 flex-1">
        <ActivityBar page={page} onSelect={setPage} />
        <main className="flex min-w-0 flex-1 flex-col bg-vsc-bg" aria-labelledby="page-title">
          <header className="flex h-9 items-center gap-3 border-b border-vsc-border px-5">
            <h1 id="page-title" className="text-[11px] font-normal tracking-wide uppercase text-vsc-desc">
              {current.label}
            </h1>
            {browserPreview && (
              <span className="ml-auto rounded-sm bg-vsc-badge px-1.5 text-[11px] text-vsc-fg">
                Anteprima nel browser: dati di esempio
              </span>
            )}
          </header>
          <div className="min-h-0 flex-1 overflow-auto p-5">
            {page === "home" ? (
              <Home info={info} error={error} editor={editor} onChanged={refreshEditor} />
            ) : page === "projects" ? (
              <ProjectsPage status={editor} onChanged={refreshEditor} />
            ) : (
              <Placeholder />
            )}
          </div>
        </main>
      </div>
      <StatusBar info={info} editor={editor} />
    </div>
  );
}

function ActivityBar({ page, onSelect }: { page: PageId; onSelect: (id: PageId) => void }) {
  return (
    <nav aria-label="Pagine" className="flex w-12 flex-col border-r border-vsc-border bg-vsc-activitybar">
      {PAGES.map(({ id, label, icon: Icon }) => {
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
  onChanged,
}: {
  info: AppInfo | null;
  error: string | null;
  editor: EditorStatus | null;
  onChanged: () => void;
}) {
  if (error) {
    return (
      <Notice icon={VscWarning} tone="error">
        Impossibile leggere lo stato del kit: {error}
      </Notice>
    );
  }
  if (!info) {
    return <p className="text-vsc-desc">Caricamento…</p>;
  }
  return (
    <div className="max-w-3xl space-y-5">
      {info.movedFrom ? (
        <Notice icon={VscWarning} tone="warning">
          Il kit è stato spostato: prima era in {info.movedFrom}, ora in {info.root}.
        </Notice>
      ) : (
        <Notice icon={VscCheck} tone="success">
          Dev-Kit è pronto. Tutto ciò che scarica e salva resta in questa cartella.
        </Notice>
      )}
      <HomeActions status={editor} onChanged={onChanged} />
      <section aria-labelledby="where" className="rounded border border-vsc-border bg-vsc-sidebar">
        <h2 id="where" className="border-b border-vsc-border px-4 py-2 text-[13px] font-semibold">
          Dove si trova il kit
        </h2>
        <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1.5 px-4 py-3">
          <Row label="Cartella del kit" value={info.root} />
          <Row label="Progetti" value={info.projects} />
          <Row label="Dati e programmi" value={info.data} />
          <Row label="Piattaforma" value={info.platform} />
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

function Placeholder() {
  return <p className="text-vsc-desc">Questa pagina arriva nelle prossime fasi.</p>;
}

function StatusBar({ info, editor }: { info: AppInfo | null; editor: EditorStatus | null }) {
  return (
    <footer
      aria-label="Barra di stato"
      className="flex h-[22px] items-center gap-3 border-t border-vsc-border bg-vsc-statusbar px-2 text-[12px] text-vsc-statusbar-fg"
    >
      <span className="flex h-full items-center gap-1 bg-vsc-button px-2 text-vsc-button-fg">
        <VscRemote size={14} aria-hidden="true" />
        Dev-Kit
      </span>
      {info && (
        <>
          <span>{info.platform}</span>
          {editor?.running && (
            <span className="flex items-center gap-1" title={`Server dell'editor su 127.0.0.1:${String(editor.port)}`}>
              <VscCode size={14} aria-hidden="true" />
              Editor attivo
            </span>
          )}
          <span className="ml-auto">v{info.version}</span>
        </>
      )}
    </footer>
  );
}
