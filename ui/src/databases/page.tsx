// The Database page (DECISIONS.md D-73): the kit's MySQL and MongoDB
// servers, one for every project, on 127.0.0.1 only. SQLite needs no server:
// it is a tool (Tools) and a section of the project's page, not listed here.
// As the Extensions view: the servers in the side bar, the chosen one's
// page in the editor.

import { useState } from "react";
import {
  dbAdminOpen,
  folderOpen,
  serviceLine,
  serviceLinePort,
  serviceSet,
  serviceStart,
  serviceStop,
  servicesList,
  type LineInfo,
  type ServiceInfo,
  type ServiceState,
} from "../ipc";
import { t } from "../i18n";
import { ComponentLogo } from "../shared/logos";
import { ActionButton, ErrorLine, portRangeText, usePortField } from "../shared/common";
import { useAction, useCopy, useLoad } from "../shared/hooks";
import {
  Button,
  CheckboxLabel,
  codicon,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  List,
  ListItem,
  Notice,
  PaneStack,
  Properties,
  Select,
  SideBarTitle,
  TextField,
  ViewContainer,
  ViewPane,
  type DotTone,
} from "../workbench";

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

function stateTone(state: ServiceState): DotTone {
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
        icon={shown ? codicon("eye-closed") : codicon("eye")}
        label={shown ? t("Nascondi {0}", label) : t("Mostra {0}", label)}
        title={shown ? t("Nascondi") : t("Mostra")}
        checked={shown}
        onClick={() => {
          setShown(!shown);
        }}
      />
      <IconButton
        icon={codicon("copy")}
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

/** The tool that shows a server's database in the browser (D-107, D-147):
 * phpMyAdmin for MySQL, mongo-express for MongoDB; none for SQLite. */
export interface DbViewer {
  engine: "mysql" | "mongodb";
  /** «Apri phpMyAdmin». */
  open: string;
  title: string;
  /** «phpMyAdmin su 8.4». */
  onLine: (line: string) => string;
}

export function dbViewer(id: string): DbViewer | null {
  switch (id) {
    case "mysql":
      return {
        engine: "mysql",
        open: t("Apri phpMyAdmin"),
        title: t("MySQL nel browser; al primo uso si installa e il server si avvia"),
        onLine: (line) => t("phpMyAdmin su {0}", line),
      };
    case "mongodb":
      return {
        engine: "mongodb",
        open: t("Apri mongo-express"),
        title: t("MongoDB nel browser; al primo uso si installano mongo-express e Node.js e il server si avvia"),
        onLine: (line) => t("mongo-express su {0}", line),
      };
    default:
      return null;
  }
}

/** Another version of a server, on its own port (D-102): state,
 * port and start/stop. */
function OtherLine({
  id,
  o,
  busy,
  onStart,
  onStop,
  onPort,
  viewer,
  onViewer,
}: {
  id: string;
  o: LineInfo;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onPort: (port: number) => void;
  /** MySQL and MongoDB: their viewer on this version. */
  viewer?: DbViewer | null;
  onViewer?: (() => void) | undefined;
}) {
  const { text: port, setText: setPort, value: portValue, valid: portValid } = usePortField(o.port);
  const on = o.state === "running" || o.state === "starting";
  const portId = `port-${id}-${o.line}`;
  return (
    <div className="space-y-2 rounded border border-vsc-border p-3">
      <div className="flex flex-wrap items-center gap-3">
        <span className="font-mono font-semibold">{o.line}</span>
        <span className="flex items-center gap-1.5" role="status">
          <Dot tone={stateTone(o.state)} />
          {stateLabel(o.state)}
        </span>
        <span className="flex-1" />
        {on ? (
          <ActionButton
            icon={codicon("debug-stop")}
            label={t("Ferma {0}", o.line)}
            small
            busy={busy}
            onClick={onStop}
          />
        ) : (
          <ActionButton
            icon={codicon("debug-start")}
            label={t("Avvia {0}", o.line)}
            small
            busy={busy}
            onClick={onStart}
          />
        )}
        {viewer && onViewer && (
          <ActionButton
            icon={codicon("link-external")}
            label={viewer.onLine(o.line)}
            title={t("Questa versione nel browser; il server si avvia se è fermo")}
            small
            busy={busy}
            onClick={onViewer}
          />
        )}
      </div>
      {o.state === "error" && o.message && (
        <Notice tone="error">
          <pre className="m-0 font-mono text-[12px] whitespace-pre-wrap">{o.message}</pre>
        </Notice>
      )}
      <div className="flex flex-wrap items-center gap-2">
        <label htmlFor={portId}>{t("Porta")}</label>
        <TextField
          id={portId}
          inputMode="numeric"
          mono
          value={port}
          onChange={(v) => {
            setPort(v.replace(/\D/g, ""));
          }}
          onBlur={() => {
            if (portValid && portValue !== o.port) onPort(portValue);
          }}
          invalid={!portValid}
          className="w-24"
        />
        <span className="text-vsc-desc">
          {portValid ? (on ? t("Vale dal prossimo avvio.") : null) : portRangeText()}
        </span>
      </div>
      <code className="block font-mono text-[12px] break-all text-vsc-desc select-text">{o.dataDir}</code>
    </div>
  );
}

function ServiceDetails({
  s,
  busy,
  onStart,
  onStop,
  onSet,
  onLine,
  onViewer,
  lineBusy,
  onLineStart,
  onLineStop,
  onLinePort,
  onLineViewer,
}: {
  s: ServiceInfo;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  /** MySQL and MongoDB: their viewer in the browser ([`dbViewer`]). */
  onViewer?: (() => void) | undefined;
  /** The same on another version. */
  onLineViewer?: ((line: string) => void) | undefined;
  onSet: (port: number | null, autostart: boolean) => void;
  onLine: (line: string | null) => void;
  /** The other versions (D-102). */
  lineBusy: (line: string) => boolean;
  onLineStart: (line: string) => void;
  onLineStop: (line: string) => void;
  onLinePort: (line: string, port: number) => void;
}) {
  const { text: port, setText: setPort, value: portValue, valid: portValid } = usePortField(s.port);
  const on = s.state === "running" || s.state === "starting";
  const portId = `port-${s.id}`;
  const viewer = dbViewer(s.id);
  // The content is in the server's web admin (D-107, D-147, D-150): the
  // page shows only the details.
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
              <ActionButton icon={codicon("debug-stop")} label={t("Ferma")} busy={busy} onClick={onStop} />
            ) : (
              <ActionButton icon={codicon("debug-start")} label={t("Avvia")} primary busy={busy} onClick={onStart} />
            )}
            {viewer && onViewer && (
              <ActionButton
                icon={codicon("link-external")}
                label={viewer.open}
                title={viewer.title}
                busy={busy}
                onClick={onViewer}
              />
            )}
          </>
        }
      />
      <EditorBody>
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
              <p className="text-vsc-desc">{portValid ? t("Vale dal prossimo avvio.") : portRangeText()}</p>
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
              {t("Avvia con Atollo")}
            </CheckboxLabel>
            {s.others.length > 0 && (
              <p className="text-vsc-desc">
                {t("Con il server partono anche le altre versioni su cui sono i database dei progetti.")}
              </p>
            )}
          </div>
        </EditorSection>
        {s.others.length > 0 && (
          <EditorSection id={`db-others-${s.id}`} title={t("Altre versioni")}>
            <div className="space-y-3">
              <p className="text-vsc-desc">
                {t(
                  "Accese insieme a {0}, ciascuna sulla sua porta e con i suoi dati, per i progetti che le chiedono: si sceglie nella pagina del progetto, e la porta arriva al progetto come DB_PORT.",
                  s.line ?? "",
                )}
              </p>
              {s.others.map((o) => (
                <OtherLine
                  // A new key when the port changes: the field starts from it.
                  key={`${o.line}-${String(o.port)}`}
                  id={s.id}
                  o={o}
                  busy={lineBusy(o.line)}
                  onStart={() => {
                    onLineStart(o.line);
                  }}
                  onStop={() => {
                    onLineStop(o.line);
                  }}
                  onPort={(port) => {
                    onLinePort(o.line, port);
                  }}
                  viewer={viewer}
                  onViewer={
                    onLineViewer
                      ? () => {
                          onLineViewer(o.line);
                        }
                      : undefined
                  }
                />
              ))}
            </div>
          </EditorSection>
        )}
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

export function DatabasesPage({ onGuide }: { onGuide?: (topic: "databases") => void } = {}) {
  // The list is read now and every 3 s; an action answers with the new list.
  // The read's error and the action's are kept apart: a read that works
  // clears the first, not the second. After an action the list is read
  // again, which also drops a read still running from before it.
  const {
    data: list,
    error: loadError,
    reload: refresh,
    setData: setList,
  } = useLoad<ServiceInfo[] | null>(servicesList, null, { onError: "show", every: 3000 });
  const { busy, error, run } = useAction();
  const [chosen, setChosen] = useState<string | null>(null);
  const [listOpen, setListOpen] = useState(true);
  const act = (id: string, action: () => Promise<ServiceInfo[]>) => {
    run(id, () =>
      action().then(
        (l) => {
          setList(l);
          refresh();
        },
        (e: unknown) => {
          refresh();
          throw e;
        },
      ),
    );
  };
  const current = chosen ?? list?.[0]?.id;
  const service = list?.find((s) => s.id === current);
  const viewerOf = service ? dbViewer(service.id) : null;
  const running = (list ?? []).reduce((n, s) => n + [s, ...s.others].filter((x) => x.state === "running").length, 0);

  const side = (
    <>
      <SideBarTitle
        title={t("Database")}
        actions={
          <>
            <IconButton icon={codicon("refresh")} label={t("Aggiorna")} onClick={refresh} />
            <IconButton
              icon={codicon("folder-opened")}
              label={t("Apri la cartella dei dati")}
              onClick={() => {
                void folderOpen("db");
              }}
            />
            {onGuide && (
              <IconButton
                icon={codicon("book")}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("databases");
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
            </List>
          )}
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="databases" side={side}>
      <div className="px-5 pt-3 empty:hidden">
        <ErrorLine error={loadError} />
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
          lineBusy={(line) => busy === `${service.id}@${line}`}
          onLineStart={(line) => {
            act(`${service.id}@${line}`, () => serviceStart(service.id, line));
          }}
          onLineStop={(line) => {
            act(`${service.id}@${line}`, () => serviceStop(service.id, line));
          }}
          onLinePort={(line, port) => {
            act(`${service.id}@${line}`, () => serviceLinePort(service.id, line, port));
          }}
          onViewer={
            viewerOf
              ? () => {
                  act(service.id, () => dbAdminOpen(null, viewerOf.engine));
                }
              : undefined
          }
          onLineViewer={
            viewerOf
              ? (line) => {
                  act(`${service.id}@${line}`, () => dbAdminOpen(line, viewerOf.engine));
                }
              : undefined
          }
        />
      ) : (
        <div className="flex-1 p-5 text-vsc-desc">{list === null ? t("Caricamento…") : null}</div>
      )}
      <p className="shrink-0 border-t border-vsc-border px-5 py-2 text-[12px] text-vsc-desc">
        {t(
          "Un server per tipo, condiviso da tutti i progetti, raggiungibile solo da questo PC (127.0.0.1), con password. Nel terminale del kit il comando mysql entra già come root; «Apri phpMyAdmin» apre MySQL nel browser e «Apri mongo-express» MongoDB. Ogni progetto ha il suo database e il suo utente: si creano dalla pagina del progetto.",
        )}
      </p>
    </ViewContainer>
  );
}
