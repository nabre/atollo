// The Database page (DECISIONS.md D-73): the kit's MariaDB and MongoDB
// servers, one for every project, on 127.0.0.1 only. SQLite needs no server:
// it is a tool (Tools) and a section of the project's page, not listed here.
// As the Extensions view: the servers in the side bar, the chosen one's
// page in the editor.

import { useCallback, useState } from "react";
import {
  VscBook,
  VscCopy,
  VscDebugStart,
  VscDebugStop,
  VscEye,
  VscEyeClosed,
  VscFolderOpened,
  VscLinkExternal,
  VscRefresh,
} from "react-icons/vsc";
import {
  dbAdminOpen,
  folderOpen,
  serviceLine,
  serviceSet,
  serviceStart,
  serviceStop,
  servicesList,
  type ServiceInfo,
  type ServiceState,
} from "./api";
import { t } from "./i18n";
import { DbBrowser } from "./dbbrowser";
import { ComponentLogo } from "./logos";
import { ActionButton, ErrorLine } from "./common";
import { useCopy, useEvery } from "./hooks";
import {
  Button,
  CheckboxLabel,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  List,
  ListItem,
  NavBar,
  Notice,
  PaneStack,
  Properties,
  Select,
  SideBarTitle,
  TextField,
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

function stateTone(state: ServiceState): "ok" | "warn" | "err" | "off" {
  switch (state) {
    case "running":
      return "ok";
    case "starting":
      return "warn";
    case "error":
      return "err";
    default:
      return "off";
  }
}

/** The state of a server with its dot, or "not installed". */
function StateText({ s }: { s: ServiceInfo }) {
  if (!s.installed) return <>{t("Non installato: si installa al primo avvio")}</>;
  return (
    <span className="flex items-center gap-1.5">
      <Dot tone={stateTone(s.state)} />
      {stateLabel(s.state)}
    </span>
  );
}

/** A password: hidden until shown, copied without showing it. */
export function SecretField({ value, label }: { value: string; label: string }) {
  const [shown, setShown] = useState(false);
  const { copied, copy } = useCopy(1500);
  return (
    <span className="flex min-w-0 items-center gap-2">
      <code className="truncate font-mono text-vsc-fg">{shown ? value : "••••••••••••"}</code>
      <IconButton
        icon={shown ? VscEyeClosed : VscEye}
        label={shown ? t("Nascondi {0}", label) : t("Mostra {0}", label)}
        title={shown ? t("Nascondi") : t("Mostra")}
        checked={shown}
        onClick={() => {
          setShown(!shown);
        }}
      />
      <IconButton
        icon={VscCopy}
        label={t("Copia {0}", label)}
        title={t("Copia")}
        onClick={() => {
          copy(value);
        }}
      />
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
  onLine,
  onAdminer,
}: {
  s: ServiceInfo;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  /** MariaDB only: Adminer in the browser. */
  onAdminer?: (() => void) | undefined;
  onSet: (port: number | null, autostart: boolean) => void;
  onLine: (line: string | null) => void;
}) {
  const [port, setPort] = useState(String(s.port));
  const on = s.state === "running" || s.state === "starting";
  const portValue = Number(port);
  const portValid = Number.isInteger(portValue) && portValue >= 1024 && portValue <= 65535;
  const portId = `port-${s.id}`;
  // MariaDB's content is in Adminer (D-85): no explorer, only the details.
  const explorer = s.id !== "mariadb";
  const [chosen, setSection] = useState<"content" | "details">(s.state === "running" ? "content" : "details");
  const section = explorer ? chosen : "details";
  // The explorer stays while the server is on: a check that misses once
  // ("starting" for a moment) must not close it and lose the table chosen.
  const [browsable, setBrowsable] = useState(s.state === "running");
  if (s.state === "running" && !browsable) setBrowsable(true);
  if (!on && browsable) setBrowsable(false);
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={s.id} size={56} />}
        title={s.name}
        titleId={`db-${s.id}`}
        meta={[
          <span key="state" role="status">
            <StateText s={s} />
          </span>,
          <span key="addr" className="font-mono">
            127.0.0.1:{String(s.port)}
          </span>,
          ...(s.line
            ? [
                <span key="line" className="font-mono">
                  {s.line}
                </span>,
              ]
            : []),
        ]}
        actions={
          <>
            {on ? (
              <ActionButton icon={VscDebugStop} label={t("Ferma")} busy={busy} onClick={onStop} />
            ) : (
              <ActionButton icon={VscDebugStart} label={t("Avvia")} primary busy={busy} onClick={onStart} />
            )}
            {onAdminer && (
              <ActionButton
                icon={VscLinkExternal}
                label={t("Apri Adminer")}
                title={t("MariaDB nel browser, come phpMyAdmin; al primo uso si installa e il server si avvia")}
                busy={busy}
                onClick={onAdminer}
              />
            )}
          </>
        }
      />
      {explorer && (
        <NavBar
          label={t("Sezioni del server")}
          items={[
            { id: "content", label: t("Contenuto") },
            { id: "details", label: t("Dettagli") },
          ]}
          active={section}
          onSelect={setSection}
        />
      )}
      {section === "content" &&
        (browsable ? (
          <DbBrowser engine={s.id} />
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-vsc-desc">
            <p>{t("Avvia il server per vederne il contenuto.")}</p>
            {!on && <ActionButton icon={VscDebugStart} label={t("Avvia")} primary busy={busy} onClick={onStart} />}
          </div>
        ))}
      {section === "details" && (
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
              {s.lines.length > 1 && (
                <div className="space-y-1">
                  <label htmlFor={`line-${s.id}`} className="block font-semibold">
                    {t("Versione")}
                  </label>
                  <p className="text-vsc-desc">
                    {on
                      ? t("Ferma il server per cambiarla.")
                      : t(
                          "Un server per tutti i progetti. Ogni versione ha i suoi dati: al primo avvio di una più recente si copiano quelli della precedente, che restano com'erano.",
                        )}
                  </p>
                  <Select
                    id={`line-${s.id}`}
                    value={s.line ?? ""}
                    disabled={on || busy}
                    onChange={(line) => {
                      onLine(line === s.lines[0] ? null : line);
                    }}
                    options={s.lines.map((l, i) => ({
                      value: l,
                      label: i === 0 ? t("{0} (la più recente)", l) : l,
                    }))}
                    className="w-40"
                  />
                </div>
              )}
              <div className="space-y-1">
                <label htmlFor={portId} className="block font-semibold">
                  {t("Porta")}
                </label>
                <p className="text-vsc-desc">
                  {portValid ? t("Vale dal prossimo avvio.") : t("Una porta fra 1024 e 65535.")}
                </p>
                <TextField
                  id={portId}
                  inputMode="numeric"
                  mono
                  value={port}
                  onChange={(v) => {
                    setPort(v.replace(/\D/g, ""));
                  }}
                  onBlur={() => {
                    if (portValid && portValue !== s.port) onSet(portValue, s.autostart);
                  }}
                  invalid={!portValid}
                  className="w-24"
                />
              </div>
              <CheckboxLabel
                checked={s.autostart}
                onChange={(v) => {
                  onSet(portValid ? portValue : null, v);
                }}
              >
                {t("Avvia con NEXKORA")}
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
      )}
    </div>
  );
}

export function DatabasesPage({ onGuide }: { onGuide?: (topic: "feat-services") => void } = {}) {
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
  useEvery(refresh, 3000);
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
  const current = chosen ?? list?.[0]?.id;
  const service = list?.find((s) => s.id === current);
  const running = (list ?? []).filter((s) => s.state === "running").length;

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
            {onGuide && (
              <IconButton
                icon={VscBook}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("feat-services");
                }}
              />
            )}
          </>
        }
      />
      <PaneStack>
        <ViewPane
          first
          fill
          title={t("Server")}
          badge={running > 0 ? running : undefined}
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
                    description={<StateText s={s} />}
                    footer={s.autostart ? t("Avvia con NEXKORA") : undefined}
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
          onLine={(line) => {
            act(service.id, () => serviceLine(service.id, line));
          }}
          onAdminer={
            service.id === "mariadb"
              ? () => {
                  act(service.id, dbAdminOpen);
                }
              : undefined
          }
        />
      ) : (
        <div className="flex-1 p-5 text-vsc-desc">{list === null ? t("Caricamento…") : null}</div>
      )}
      <p className="shrink-0 border-t border-vsc-border px-5 py-2 text-[12px] text-vsc-desc">
        {t(
          "Un server per tipo, condiviso da tutti i progetti, raggiungibile solo da questo PC (127.0.0.1), con password. MariaDB è compatibile con MySQL; nel terminale del kit il comando mariadb entra già come root, e «Apri Adminer» lo apre nel browser, come phpMyAdmin. Ogni progetto ha il suo database e il suo utente: si creano dalla pagina del progetto.",
        )}
      </p>
    </ViewContainer>
  );
}
