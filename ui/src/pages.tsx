// Home and Projects pages (phase 3): open the integrated editor and the kit's
// console, on the projects folder or on one project.
import { useCallback, useEffect, useState } from "react";
import type { IconType } from "react-icons";
import { VscCode, VscFolder, VscRefresh, VscTerminal, VscWarning } from "react-icons/vsc";
import { editorOpen, projectsList, terminalOpen, type EditorStatus, type ProjectEntry } from "./api";

/** Runs an action, keeping track of what is busy and of the last error. */
export function useAction(onDone?: () => void) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = useCallback(
    (key: string, action: () => Promise<void>) => {
      setBusy(key);
      setError(null);
      action().then(
        () => {
          setBusy(null);
          onDone?.();
        },
        (e: unknown) => {
          setBusy(null);
          setError(String(e));
        },
      );
    },
    [onDone],
  );
  return { busy, error, run };
}

export function ActionButton({
  icon: Icon,
  label,
  onClick,
  busy = false,
  disabled = false,
  primary = false,
  title,
}: {
  icon: IconType;
  label: string;
  onClick: () => void;
  busy?: boolean;
  disabled?: boolean;
  primary?: boolean;
  title?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled || busy}
      title={title}
      aria-busy={busy}
      className={[
        "flex items-center gap-2 rounded-sm px-3 py-1.5 text-[13px] disabled:cursor-not-allowed disabled:opacity-50",
        primary
          ? "bg-vsc-button text-vsc-button-fg enabled:hover:bg-vsc-button-hover"
          : "bg-vsc-button2 text-vsc-button2-fg enabled:hover:bg-vsc-button2-hover",
      ].join(" ")}
    >
      <Icon size={16} aria-hidden="true" className={busy ? "animate-pulse" : undefined} />
      {busy ? `${label}…` : label}
    </button>
  );
}

function ErrorLine({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <p role="alert" className="flex items-start gap-2 text-vsc-error">
      <VscWarning className="mt-0.5 shrink-0" size={16} aria-hidden="true" />
      <span className="select-text">{error}</span>
    </p>
  );
}

function editorHint(status: EditorStatus | null): string | undefined {
  if (!status) return undefined;
  if (!status.installed) return "L'editor non è installato: installalo da Linguaggi e strumenti.";
  if (status.emulated) return "Su questo PC (ARM64) l'editor gira in emulazione x64: può essere più lento.";
  return undefined;
}

export function HomeActions({ status, onChanged }: { status: EditorStatus | null; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  const hint = editorHint(status);
  return (
    <section aria-labelledby="start" className="space-y-3">
      <h2 id="start" className="text-[13px] font-semibold">
        Inizia
      </h2>
      <div className="flex flex-wrap gap-2">
        <ActionButton
          icon={VscCode}
          label="Editor"
          primary
          busy={busy === "editor"}
          disabled={status?.installed === false}
          title={hint ?? "Apre VSCodium sulla cartella dei progetti"}
          onClick={() => {
            run("editor", () => editorOpen(null));
          }}
        />
        <ActionButton
          icon={VscTerminal}
          label="Terminale"
          busy={busy === "terminal"}
          title="Console del kit con PowerShell 7 nella cartella dei progetti"
          onClick={() => {
            run("terminal", () => terminalOpen(null));
          }}
        />
      </div>
      {hint && <p className="text-vsc-desc">{hint}</p>}
      <ErrorLine error={error} />
    </section>
  );
}

const DATE = new Intl.DateTimeFormat("it-IT", { dateStyle: "medium", timeStyle: "short" });

export function ProjectsPage({ status, onChanged }: { status: EditorStatus | null; onChanged: () => void }) {
  const [projects, setProjects] = useState<ProjectEntry[] | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const { busy, error, run } = useAction(onChanged);

  const load = useCallback(() => {
    projectsList().then(setProjects, (e: unknown) => {
      setLoadError(String(e));
    });
  }, []);
  useEffect(load, [load]);

  return (
    <div className="max-w-4xl space-y-4">
      <div className="flex items-center gap-2">
        <ActionButton icon={VscRefresh} label="Aggiorna" onClick={load} />
      </div>
      <p className="text-vsc-desc">
        Le cartelle in <span className="font-mono">projects</span> sono attendibili per Claude e Codex: apri qui solo
        codice di cui ti fidi.
      </p>
      <ErrorLine error={loadError ?? error} />
      {projects === null ? (
        <p className="text-vsc-desc">Caricamento…</p>
      ) : projects.length === 0 ? (
        <p className="text-vsc-desc">Nessun progetto. Crea una cartella in projects o clonane uno dall'editor.</p>
      ) : (
        <ul aria-label="Progetti" className="divide-y divide-vsc-border rounded border border-vsc-border">
          {projects.map((p) => (
            <li key={p.name} className="flex items-center gap-3 px-3 py-2 hover:bg-vsc-list-hover">
              <VscFolder size={16} aria-hidden="true" className="shrink-0 text-vsc-desc" />
              <span className="min-w-0 flex-1 truncate">{p.name}</span>
              <span className="text-[12px] text-vsc-desc">{DATE.format(new Date(p.modified * 1000))}</span>
              <ActionButton
                icon={VscCode}
                label="Editor"
                busy={busy === `editor:${p.name}`}
                disabled={status?.installed === false}
                title={`Apre ${p.name} nell'editor`}
                onClick={() => {
                  run(`editor:${p.name}`, () => editorOpen(p.name));
                }}
              />
              <ActionButton
                icon={VscTerminal}
                label="Terminale"
                busy={busy === `terminal:${p.name}`}
                title={`Console del kit in ${p.name}`}
                onClick={() => {
                  run(`terminal:${p.name}`, () => terminalOpen(p.name));
                }}
              />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
