// The Database page (DECISIONS.md D-73): the kit's MariaDB and MongoDB
// servers, one for every project, on 127.0.0.1 only; SQLite needs no server.
// As the Extensions view: the servers in the side bar, the chosen one's
// page in the editor.

import { useCallback, useEffect, useState } from "react";
import {
  VscCopy,
  VscDebugStart,
  VscDebugStop,
  VscEye,
  VscEyeClosed,
  VscFolderOpened,
  VscRefresh,
} from "react-icons/vsc";
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
import {
  Button,
  CheckboxLabel,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  List,
  ListItem,
  Notice,
  PaneStack,
  Properties,
  SideBarTitle,
  ViewContainer,
  ViewPane,
} from "./workbench";

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

function ServiceDetails({
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
  const on = s.state === "running" || s.state === "starting";
  const portValue = Number(port);
  const portValid = Number.isInteger(portValue) && portValue >= 1024 && portValue <= 65535;
  const portId = `port-${s.id}`;
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={s.id} size={56} />}
        title={s.name}
        titleId={`db-${s.id}`}
        meta={[
          <span key="state" role="status" className={stateColor(s.state)}>
            {s.installed ? stateLabel(s.state) : t("Non installato: si installa al primo avvio")}
          </span>,
          <span key="addr" className="font-mono">
            127.0.0.1:{String(s.port)}
          </span>,
        ]}
        actions={
          on ? (
            <ActionButton icon={VscDebugStop} label={t("Ferma")} busy={busy} onClick={onStop} />
          ) : (
            <ActionButton icon={VscDebugStart} label={t("Avvia")} primary busy={busy} onClick={onStart} />
          )
        }
      />
      <EditorBody narrow>
        {s.state === "error" && s.message && (
          <Notice tone="error">
            <pre className="m-0 font-mono text-[12px] whitespace-pre-wrap">{s.message}</pre>
          </Notice>
        )}
        <EditorSection id={`db-conn-${s.id}`} title={t("Connessione")}>
          <Properties
            rows={[
              [t("Stringa"), <SecretField value={s.url} label={t("la stringa di connessione")} />],

              [
                t("Password di {0}", s.user),
                s.password ? (
                  <SecretField value={s.password} label={t("la password di {0}", s.user)} />
                ) : (
                  <span className="text-vsc-desc">{t("Generata al primo avvio.")}</span>
                ),
              ],
            ]}
          />
        </EditorSection>
        <EditorSection id={`db-set-${s.id}`} title={t("Impostazioni")}>
          <div className="space-y-4">
            <div className="space-y-1">
              <label htmlFor={portId} className="block font-semibold">
                {t("Porta")}
              </label>
              <p className="text-vsc-desc">
                {portValid ? t("Vale dal prossimo avvio.") : t("Una porta fra 1024 e 65535.")}
              </p>
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
                className="w-24 input font-mono"
              />
            </div>
            <CheckboxLabel
              checked={s.autostart}
              onChange={(v) => {
                onSet(portValid ? portValue : null, v);
              }}
            >
              {t("Avvia con Atollo")}
            </CheckboxLabel>
          </div>
        </EditorSection>
        <EditorSection id={`db-files-${s.id}`} title={t("File")}>
          <Properties
            rows={[
              [t("Dati"), <code className="font-mono text-[12px] break-all select-text">{s.dataDir}</code>],
              [t("Log"), <code className="font-mono text-[12px] break-all select-text">{s.log}</code>],
            ]}
          />
        </EditorSection>
      </EditorBody>
    </div>
  );
}

export function DatabasesPage() {
  const [list, setList] = useState<ServiceInfo[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [listOpen, setListOpen] = useState(true);
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
  const current = chosen ?? list?.[0]?.id ?? "sqlite";
  const service = list?.find((s) => s.id === current);

  const side = (
    <>
      <SideBarTitle
        title={t("Database")}
        actions={
          <>
            <IconButton icon={VscRefresh} label={t("Aggiorna")} onClick={refresh} />
            <IconButton
              icon={VscFolderOpened}
              label={t("Apri la cartella dei dati")}
              onClick={() => {
                void folderOpen("db");
              }}
            />
          </>
        }
      />
      <PaneStack>
        <ViewPane
          first
          fill
          title={t("Server")}
          expanded={listOpen}
          onToggle={() => {
            setListOpen(!listOpen);
          }}
        >
          {list === null ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
          ) : (
            <List label={t("Database")}>
              {list.map((s) => {
                const on = s.state === "running" || s.state === "starting";
                return (
                  <ListItem
                    key={s.id}
                    icon={<ComponentLogo id={s.id} size={32} />}
                    name={s.name}
                    meta={`:${String(s.port)}`}
                    description={s.installed ? stateLabel(s.state) : t("Non installato: si installa al primo avvio")}
                    footer={s.autostart ? t("Avvia con Atollo") : undefined}
                    selected={current === s.id}
                    onSelect={() => {
                      setChosen(s.id);
                    }}
                    actions={
                      <Button
                        small
                        variant={on ? "secondary" : "primary"}
                        busy={busy === s.id}
                        onClick={() => {
                          act(s.id, () => (on ? serviceStop(s.id) : serviceStart(s.id)));
                        }}
                      >
                        {on ? t("Ferma") : t("Avvia")}
                      </Button>
                    }
                  />
                );
              })}
              <ListItem
                icon={<ComponentLogo id="sqlite" size={32} />}
                name="SQLite"
                description={t("Nessun server: un file del progetto")}
                selected={current === "sqlite"}
                onSelect={() => {
                  setChosen("sqlite");
                }}
              />
            </List>
          )}
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="databases" side={side}>
      <div className="px-5 pt-3 empty:hidden">
        <ErrorLine error={error} />
      </div>
      {service ? (
        <ServiceDetails
          // A new key when the port changes: the field starts from it.
          key={`${service.id}-${String(service.port)}`}
          s={service}
          busy={busy === service.id}
          onStart={() => {
            act(service.id, () => serviceStart(service.id));
          }}
          onStop={() => {
            act(service.id, () => serviceStop(service.id));
          }}
          onSet={(port, autostart) => {
            act(service.id, () => serviceSet(service.id, port, autostart));
          }}
        />
      ) : (
        <div className="flex min-h-0 flex-1 flex-col">
          <EditorHeader
            icon={<ComponentLogo id="sqlite" size={56} />}
            title="SQLite"
            titleId="db-sqlite"
            description={t(
              "Nessun server: il database è un file del progetto. Nel terminale del kit c'è sqlite3; si installa da Strumenti.",
            )}
          />
        </div>
      )}
      <p className="shrink-0 border-t border-vsc-border px-5 py-2 text-[12px] text-vsc-desc">
        {t(
          "Un server per tipo, condiviso da tutti i progetti, raggiungibile solo da questo PC (127.0.0.1), con password. MariaDB è compatibile con MySQL; nel terminale del kit il comando mariadb entra già come root. Ogni progetto ha il suo database e il suo utente: si creano dalla pagina del progetto.",
        )}
      </p>
    </ViewContainer>
  );
}
