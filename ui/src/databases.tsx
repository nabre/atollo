// The Database page (DECISIONS.md D-73): the kit's MariaDB and MongoDB
// servers, one for every project, on 127.0.0.1 only; SQLite needs no server.
import { useCallback, useEffect, useState } from "react";
import { VscCopy, VscDebugStart, VscDebugStop, VscEye, VscEyeClosed, VscFolderOpened } from "react-icons/vsc";
import {
  folderOpen,
  serviceSet,
  serviceStart,
  serviceStop,
  servicesList,
  type ServiceInfo,
  type ServiceState,
} from "./api";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine } from "./pages";

function stateLabel(state: ServiceState): string {
  switch (state) {
    case "running":
      return t("Avviato");
    case "starting":
      return t("Avvio in corso…");
    case "error":
      return t("Errore");
    default:
      return t("Fermo");
  }
}

function stateColor(state: ServiceState): string {
  switch (state) {
    case "running":
      return "text-vsc-success";
    case "error":
      return "text-vsc-error";
    default:
      return "text-vsc-desc";
  }
}

/** A password: hidden until shown, copied without showing it. */
export function SecretField({ value, label }: { value: string; label: string }) {
  const [shown, setShown] = useState(false);
  const [copied, setCopied] = useState(false);
  return (
    <span className="flex min-w-0 items-center gap-2">
      <code className="truncate font-mono text-vsc-fg">{shown ? value : "••••••••••••"}</code>
      <button
        type="button"
        className="rounded-sm p-1 text-vsc-fg hover:bg-vsc-list-hover"
        aria-label={shown ? t("Nascondi {0}", label) : t("Mostra {0}", label)}
        title={shown ? t("Nascondi") : t("Mostra")}
        aria-pressed={shown}
        onClick={() => {
          setShown(!shown);
        }}
      >
        {shown ? <VscEyeClosed size={14} aria-hidden="true" /> : <VscEye size={14} aria-hidden="true" />}
      </button>
      <button
        type="button"
        className="rounded-sm p-1 text-vsc-fg hover:bg-vsc-list-hover"
        aria-label={t("Copia {0}", label)}
        title={t("Copia")}
        onClick={() => {
          void navigator.clipboard.writeText(value).then(() => {
            setCopied(true);
            setTimeout(() => {
              setCopied(false);
            }, 1500);
          });
        }}
      >
        <VscCopy size={14} aria-hidden="true" />
      </button>
      {copied && <span className="text-vsc-desc">{t("Copiata")}</span>}
    </span>
  );
}

function ServiceCard({
  s,
  busy,
  onStart,
  onStop,
  onSet,
}: {
  s: ServiceInfo;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onSet: (port: number | null, autostart: boolean) => void;
}) {
  const [port, setPort] = useState(String(s.port));
  const [copied, setCopied] = useState(false);
  const on = s.state === "running" || s.state === "starting";
  const portValue = Number(port);
  const portValid = Number.isInteger(portValue) && portValue >= 1024 && portValue <= 65535;
  const portId = `port-${s.id}`;
  return (
    <section className="rounded-sm border border-vsc-border bg-vsc-sidebar" aria-labelledby={`db-${s.id}`}>
      <div className="flex flex-wrap items-center gap-3 border-b border-vsc-border px-4 py-3">
        <ComponentLogo id={s.id} size={22} />
        <h2 id={`db-${s.id}`} className="text-[14px] font-semibold text-vsc-fg">
          {s.name}
        </h2>
        <span className={`text-[13px] ${stateColor(s.state)}`} role="status">
          {s.installed ? stateLabel(s.state) : t("Non installato: si installa al primo avvio")}
        </span>
        <span className="ml-auto flex gap-2">
          {on ? (
            <ActionButton icon={VscDebugStop} label={t("Ferma")} busy={busy} onClick={onStop} />
          ) : (
            <ActionButton icon={VscDebugStart} label={t("Avvia")} primary busy={busy} onClick={onStart} />
          )}
        </span>
      </div>
      <div className="grid gap-x-6 gap-y-2 px-4 py-3 text-[13px] sm:grid-cols-[max-content_1fr]">
        <span className="text-vsc-desc">{t("Connessione")}</span>
        <span className="flex min-w-0 items-center gap-2">
          <code className="truncate font-mono text-vsc-fg">{s.url}</code>
          <button
            type="button"
            className="rounded-sm p-1 text-vsc-fg hover:bg-vsc-list-hover"
            title={t("Copia")}
            aria-label={t("Copia la stringa di connessione")}
            onClick={() => {
              void navigator.clipboard.writeText(s.url).then(() => {
                setCopied(true);
                setTimeout(() => {
                  setCopied(false);
                }, 1500);
              });
            }}
          >
            <VscCopy size={14} aria-hidden="true" />
          </button>
          {copied && <span className="text-vsc-desc">{t("Copiata")}</span>}
        </span>
        <span className="text-vsc-desc">{t("Password di {0}", s.user)}</span>
        {s.password ? (
          <SecretField value={s.password} label={t("la password di {0}", s.user)} />
        ) : (
          <span className="text-vsc-desc">{t("Generata al primo avvio.")}</span>
        )}
        <label htmlFor={portId} className="text-vsc-desc">
          {t("Porta")}
        </label>
        <span className="flex items-center gap-2">
          <input
            id={portId}
            inputMode="numeric"
            value={port}
            onChange={(e) => {
              setPort(e.target.value.replace(/\D/g, ""));
            }}
            onBlur={() => {
              if (portValid && portValue !== s.port) onSet(portValue, s.autostart);
            }}
            aria-invalid={!portValid}
            className="w-24 rounded-sm border border-vsc-input-border bg-vsc-input px-2 py-1 font-mono text-vsc-fg"
          />
          <span className="text-vsc-desc">
            {portValid ? t("Vale dal prossimo avvio.") : t("Una porta fra 1024 e 65535.")}
          </span>
        </span>
        <span className="text-vsc-desc">{t("Dati")}</span>
        <code className="truncate font-mono text-vsc-fg">{s.dataDir}</code>
        <span className="text-vsc-desc">{t("Log")}</span>
        <code className="truncate font-mono text-vsc-fg">{s.log}</code>
        <span />
        <label className="flex items-center gap-2 text-vsc-fg">
          <input
            type="checkbox"
            checked={s.autostart}
            onChange={(e) => {
              onSet(portValid ? portValue : null, e.target.checked);
            }}
          />
          {t("Avvia con Dev-Kit")}
        </label>
      </div>
      {s.state === "error" && s.message && (
        <pre className="mx-4 mb-3 overflow-auto rounded-sm bg-vsc-bg p-2 font-mono text-[12px] whitespace-pre-wrap text-vsc-error">
          {s.message}
        </pre>
      )}
    </section>
  );
}

export function DatabasesPage() {
  const [list, setList] = useState<ServiceInfo[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(() => {
    servicesList().then(setList, (e: unknown) => {
      setError(String(e));
    });
  }, []);
  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, 3000);
    return () => {
      clearInterval(timer);
    };
  }, [refresh]);
  const act = (id: string, action: () => Promise<ServiceInfo[]>) => {
    setBusy(id);
    setError(null);
    action().then(
      (l) => {
        setList(l);
        setBusy(null);
      },
      (e: unknown) => {
        setError(String(e));
        setBusy(null);
        refresh();
      },
    );
  };
  return (
    <div className="max-w-3xl space-y-4">
      <p className="text-vsc-desc">
        {t(
          "Un server per tipo, condiviso da tutti i progetti, raggiungibile solo da questo PC (127.0.0.1), con password. MariaDB è compatibile con MySQL; nel terminale del kit il comando mariadb entra già come root. Ogni progetto ha il suo database e il suo utente: si creano dalla pagina del progetto.",
        )}
      </p>
      <ErrorLine error={error} />
      {list === null ? (
        <p className="text-vsc-desc">{t("Caricamento…")}</p>
      ) : (
        list.map((s) => (
          <ServiceCard
            // A new key when the port changes: the field starts from it.
            key={`${s.id}-${String(s.port)}`}
            s={s}
            busy={busy === s.id}
            onStart={() => {
              act(s.id, () => serviceStart(s.id));
            }}
            onStop={() => {
              act(s.id, () => serviceStop(s.id));
            }}
            onSet={(port, autostart) => {
              act(s.id, () => serviceSet(s.id, port, autostart));
            }}
          />
        ))
      )}
      <section className="rounded-sm border border-vsc-border bg-vsc-sidebar px-4 py-3" aria-labelledby="db-sqlite">
        <div className="flex items-center gap-3">
          <ComponentLogo id="sqlite" size={22} />
          <h2 id="db-sqlite" className="text-[14px] font-semibold text-vsc-fg">
            SQLite
          </h2>
        </div>
        <p className="mt-2 text-[13px] text-vsc-desc">
          {t(
            "Nessun server: il database è un file del progetto. Nel terminale del kit c'è sqlite3; si installa da Strumenti.",
          )}
        </p>
      </section>
      <ActionButton
        icon={VscFolderOpened}
        label={t("Apri la cartella dei dati")}
        onClick={() => {
          void folderOpen("db");
        }}
      />
    </div>
  );
}
