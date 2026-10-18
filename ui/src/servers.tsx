// The Web Servers page: shared Caddy and Apache VirtualHost server pools
// (automatic by engine + PHP version or manual profiles), with .localhost
// VirtualHosts and portable relative path aliases.

import { useCallback, useState } from "react";
import {
  VscAdd,
  VscBook,
  VscDebugStart,
  VscDebugStop,
  VscGlobe,
  VscLinkExternal,
  VscRefresh,
  VscTrash,
} from "react-icons/vsc";
import {
  serverReload,
  serverRemove,
  serverSave,
  serverStart,
  serverStop,
  serversList,
  urlOpen,
  type WebServerEngine,
  type WebServerPool,
  type WebServersView,
} from "./api";
import { ActionButton, ErrorLine } from "./common";
import { useEvery } from "./hooks";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
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
  Notice,
  PaneStack,
  Properties,
  Select,
  SideBarTitle,
  TextField,
  ViewContainer,
  ViewPane,
} from "./workbench";

/** The two options of a profile, as in the creation form and the details. */
function ProfileOptions({
  keepAlive,
  autostart,
  disabled = false,
  onChange,
}: {
  keepAlive: boolean;
  autostart: boolean;
  disabled?: boolean;
  onChange: (keepAlive: boolean, autostart: boolean) => void;
}) {
  return (
    <div className="space-y-1.5">
      <CheckboxLabel
        checked={keepAlive}
        disabled={disabled}
        onChange={(v) => {
          onChange(v, autostart);
        }}
      >
        {t("Mantieni attivo anche senza progetti")}
      </CheckboxLabel>
      <CheckboxLabel
        checked={autostart}
        disabled={disabled}
        onChange={(v) => {
          onChange(keepAlive, v);
        }}
      >
        {t("Avvia con Atollo")}
      </CheckboxLabel>
    </div>
  );
}

function PoolDetails({
  pool,
  busy,
  onStart,
  onStop,
  onReload,
  onRemove,
  onOptions,
}: {
  pool: WebServerPool;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onReload: () => void;
  onRemove?: (() => void) | undefined;
  onOptions?: ((keepAlive: boolean, autostart: boolean) => void) | undefined;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={pool.engine} size={56} />}
        title={pool.name}
        titleId={`srv-${pool.id}`}
        meta={[
          <span key="state" role="status" className="flex items-center gap-1.5">
            <Dot tone={pool.running ? "ok" : "off"} />
            {pool.running ? t("Avviato") : t("Fermo")}
          </span>,
          <span key="addr" className="font-mono">
            127.0.0.1:{String(pool.port)}
          </span>,
          ...(pool.pid !== null
            ? [
                <span key="pid" className="font-mono text-vsc-desc">
                  PID {String(pool.pid)}
                </span>,
              ]
            : []),
        ]}
        actions={
          <>
            {pool.running ? (
              <ActionButton icon={VscDebugStop} label={t("Ferma")} busy={busy} onClick={onStop} />
            ) : (
              <ActionButton icon={VscDebugStart} label={t("Avvia")} primary busy={busy} onClick={onStart} />
            )}
            {!pool.orphan && (
              <ActionButton icon={VscRefresh} label={t("Ricarica configurazione")} busy={busy} onClick={onReload} />
            )}
            {onRemove && <ActionButton icon={VscTrash} label={t("Elimina profilo")} busy={busy} onClick={onRemove} />}
          </>
        }
      />
      <EditorBody narrow>
        {pool.orphan && (
          <Notice tone="warning">
            {t("Questo pool è ancora acceso ma non corrisponde più ai progetti e ai profili: fermalo.")}
          </Notice>
        )}
        <EditorSection id={`srv-sites-${pool.id}`} title={t("VirtualHost collegati")}>
          {pool.sites.length === 0 ? (
            <Notice tone="info">
              {t(
                "Nessun sito in questo pool. Un sito Caddy o Apache compare qui quando il suo progetto lo avvia o lo sceglie nella scheda Sito web (pool automatico delle sue versioni, o questo profilo).",
              )}
            </Notice>
          ) : (
            <div className="space-y-2">
              {pool.sites.map((site) => {
                const aliasEntries = Object.entries(site.aliases);
                return (
                  <div
                    key={`${site.project}:${site.action}`}
                    data-detached={site.detached ? "true" : undefined}
                    className={`rounded border px-3 py-2.5 space-y-1.5 ${
                      site.detached
                        ? "border-dashed border-vsc-border opacity-60"
                        : "border-vsc-border bg-vsc-sidebar/40"
                    }`}
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold text-vsc-fg">{site.project}</span>
                        <code className="rounded bg-vsc-badge px-1.5 py-0.5 font-mono text-[11px] text-vsc-badge-fg">
                          {site.folder}
                        </code>
                        {site.detached && (
                          <span className="rounded border border-vsc-border px-1.5 py-0.5 text-[11px] text-vsc-desc">
                            {t("Sganciato: il progetto è fermo")}
                          </span>
                        )}
                      </div>
                      <Button
                        small
                        variant="secondary"
                        onClick={() => {
                          void urlOpen(site.url);
                        }}
                      >
                        <VscLinkExternal className="mr-1 inline" />
                        {site.url}
                      </Button>
                    </div>
                    <div className="flex flex-wrap items-center gap-1.5 text-[12px] text-vsc-desc">
                      <VscGlobe className="shrink-0" />
                      <code className="font-mono text-vsc-fg">{site.serverName}</code>
                      {site.domains.map((d) => (
                        <code key={d} className="rounded border border-vsc-border px-1.5 py-0.5 font-mono text-[11px]">
                          {d}
                        </code>
                      ))}
                    </div>
                    {aliasEntries.length > 0 && (
                      <div className="flex flex-wrap items-center gap-1.5 pt-0.5 text-[11px]">
                        <span className="text-vsc-desc">{t("Alias di percorso:")}</span>
                        {aliasEntries.map(([u, target]) => (
                          <code key={u} className="rounded bg-vsc-input px-1.5 py-0.5 font-mono text-vsc-fg">
                            {u} → {target}
                          </code>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </EditorSection>
        <EditorSection id={`srv-info-${pool.id}`} title={t("Dettagli del pool")}>
          <Properties
            rows={[
              [t("Motore"), pool.engine === "apache" ? "Apache httpd" : "Caddy"],
              [t("Versione server"), pool.serverLine ?? t("Predefinita")],
              [t("Versione PHP"), pool.phpLine ?? t("Nessuna (solo file statici)")],
              [t("Porta"), <code className="font-mono">127.0.0.1:{String(pool.port)}</code>],
              [t("Tipo"), pool.manual ? t("Profilo manuale") : t("Pool automatico")],
              [
                t("Configurazione"),
                <code className="font-mono text-[12px] break-all select-text">{pool.confDir}</code>,
              ],
            ]}
          />
        </EditorSection>
        <EditorSection id={`srv-options-${pool.id}`} title={t("Avvio e arresto")}>
          {pool.profile && onOptions ? (
            <ProfileOptions
              keepAlive={pool.profile.keepAlive ?? true}
              autostart={pool.profile.autostart ?? false}
              disabled={busy}
              onChange={onOptions}
            />
          ) : (
            <p className="text-[12px] text-vsc-desc">
              {t(
                "Un pool automatico parte con il primo progetto che lo usa e si ferma con l'ultimo, se non l'hai avviato da questa pagina.",
              )}
            </p>
          )}
        </EditorSection>
      </EditorBody>
    </div>
  );
}

export function ServersPage({ onGuide }: { onGuide?: (topic: "feat-services") => void } = {}) {
  const [view, setView] = useState<WebServersView | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [listOpen, setListOpen] = useState(true);
  const [createOpen, setCreateOpen] = useState(false);

  const [newName, setNewName] = useState("");
  const [newEngine, setNewEngine] = useState<WebServerEngine>("caddy");
  const [newServerLine, setNewServerLine] = useState("");
  const [newPhpLine, setNewPhpLine] = useState("");
  const [newPort, setNewPort] = useState("");
  const [newKeepAlive, setNewKeepAlive] = useState(true);
  const [newAutostart, setNewAutostart] = useState(false);

  const load = useCallback((fresh: boolean) => {
    serversList(fresh).then(setView, (e: unknown) => {
      setError(String(e));
    });
  }, []);
  const refresh = useCallback(() => {
    load(false);
  }, [load]);
  const refreshFresh = useCallback(() => {
    load(true);
  }, [load]);
  useEvery(refresh, 3000);

  const run = (id: string, fn: () => Promise<void>) => {
    setBusy(id);
    setError(null);
    fn()
      .then(() => {
        setBusy(null);
        refresh();
      })
      .catch((e: unknown) => {
        setError(String(e));
        setBusy(null);
        refresh();
      });
  };

  const pools = view?.pools ?? [];
  const current = chosen ?? pools[0]?.id ?? null;
  const activePool = pools.find((p) => p.id === current) ?? pools[0];
  const runningCount = pools.filter((p) => p.running).length;
  const engineLines = newEngine === "apache" ? (view?.apacheLines ?? []) : (view?.caddyLines ?? []);
  const phpLines = view?.phpLines ?? [];

  const side = (
    <>
      <SideBarTitle
        title={t("Server Web")}
        actions={
          <>
            <IconButton
              icon={VscAdd}
              label={t("Nuovo profilo server")}
              onClick={() => {
                setCreateOpen(true);
              }}
            />
            <IconButton icon={VscRefresh} label={t("Aggiorna")} onClick={refreshFresh} />
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
          fill={!createOpen}
          title={t("Pool attivi e profili")}
          badge={runningCount > 0 ? runningCount : undefined}
          expanded={listOpen}
          onToggle={() => {
            setListOpen(!listOpen);
          }}
        >
          {view === null ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
          ) : pools.length === 0 ? (
            <p className="px-5 py-2 text-[12px] text-vsc-desc">
              {t("Nessun pool configurato. Crea un profilo manuale o aggiungi un progetto Caddy/Apache.")}
            </p>
          ) : (
            <List label={t("Server Web")}>
              {pools.map((p) => (
                <ListItem
                  key={p.id}
                  icon={<ComponentLogo id={p.engine} size={32} />}
                  name={p.name}
                  meta={`:${String(p.port)}`}
                  description={
                    <span className="flex items-center gap-1.5">
                      <Dot tone={p.running ? "ok" : "off"} />
                      {p.running ? t("Avviato") : t("Fermo")} · {t("{0} siti", String(p.sites.length))}
                    </span>
                  }
                  selected={activePool?.id === p.id}
                  onSelect={() => {
                    setChosen(p.id);
                  }}
                  actions={
                    <Button
                      small
                      variant={p.running ? "secondary" : "primary"}
                      busy={busy === p.id}
                      onClick={() => {
                        run(p.id, () => (p.running ? serverStop(p.id) : serverStart(p.id)));
                      }}
                    >
                      {p.running ? t("Ferma") : t("Avvia")}
                    </Button>
                  }
                />
              ))}
            </List>
          )}
        </ViewPane>
        <ViewPane
          title={t("Nuovo profilo server")}
          expanded={createOpen}
          onToggle={() => {
            setCreateOpen(!createOpen);
          }}
        >
          <div className="space-y-2.5 px-4 py-2 text-[12px]">
            <div>
              <label htmlFor="srv-new-name" className="mb-1 block font-semibold">
                {t("Nome del profilo")}
              </label>
              <TextField
                id="srv-new-name"
                value={newName}
                onChange={setNewName}
                placeholder="Apache 2.4 Legacy PHP 5.6"
              />
            </div>
            <div>
              <label htmlFor="srv-new-engine" className="mb-1 block font-semibold">
                {t("Server web")}
              </label>
              <Select
                id="srv-new-engine"
                value={newEngine}
                onChange={(v) => {
                  setNewEngine(v);
                  setNewServerLine("");
                }}
                options={[
                  { value: "caddy", label: "Caddy" },
                  { value: "apache", label: "Apache httpd" },
                ]}
              />
            </div>
            {engineLines.length > 0 && (
              <div>
                <label htmlFor="srv-new-sline" className="mb-1 block font-semibold">
                  {t("Versione server")}
                </label>
                <Select
                  id="srv-new-sline"
                  value={newServerLine}
                  onChange={setNewServerLine}
                  options={[
                    { value: "", label: t("Predefinita") },
                    ...engineLines.map((l) => ({ value: l, label: l })),
                  ]}
                />
              </div>
            )}
            {phpLines.length > 0 && (
              <div>
                <label htmlFor="srv-new-pline" className="mb-1 block font-semibold">
                  {t("Versione PHP")}
                </label>
                <Select
                  id="srv-new-pline"
                  value={newPhpLine}
                  onChange={setNewPhpLine}
                  options={[
                    { value: "", label: t("Predefinita") },
                    ...phpLines.map((l) => ({ value: l, label: `PHP ${l}` })),
                  ]}
                />
              </div>
            )}
            <div>
              <label htmlFor="srv-new-port" className="mb-1 block font-semibold">
                {t("Porta (opzionale)")}
              </label>
              <TextField
                id="srv-new-port"
                inputMode="numeric"
                mono
                value={newPort}
                onChange={(v) => {
                  setNewPort(v.replace(/\D/g, ""));
                }}
                placeholder="8081"
              />
            </div>
            <ProfileOptions
              keepAlive={newKeepAlive}
              autostart={newAutostart}
              onChange={(keepAlive, autostart) => {
                setNewKeepAlive(keepAlive);
                setNewAutostart(autostart);
              }}
            />
            <Button
              small
              disabled={newName.trim().length === 0 || busy === "__create"}
              onClick={() => {
                const portNum = newPort.trim() ? Number(newPort.trim()) : null;
                run("__create", async () => {
                  await serverSave({
                    id: "",
                    name: newName.trim(),
                    engine: newEngine,
                    serverLine: newServerLine || null,
                    phpLine: newPhpLine || null,
                    port: portNum,
                    keepAlive: newKeepAlive,
                    autostart: newAutostart,
                  });
                  setNewName("");
                  setNewPort("");
                  // The next profile starts from the defaults, not from
                  // this one's options.
                  setNewKeepAlive(true);
                  setNewAutostart(false);
                  setCreateOpen(false);
                });
              }}
            >
              {t("Crea profilo")}
            </Button>
          </div>
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="servers" side={side}>
      <div className="px-5 pt-3 empty:hidden">
        <ErrorLine error={error} />
      </div>
      {activePool ? (
        <PoolDetails
          key={activePool.id}
          pool={activePool}
          busy={busy === activePool.id}
          onStart={() => {
            run(activePool.id, () => serverStart(activePool.id));
          }}
          onStop={() => {
            run(activePool.id, () => serverStop(activePool.id));
          }}
          onReload={() => {
            run(activePool.id, () => serverReload(activePool.id));
          }}
          onRemove={
            activePool.manual
              ? () => {
                  run(activePool.id, () => serverRemove(activePool.id));
                }
              : undefined
          }
          onOptions={
            activePool.profile
              ? (keepAlive, autostart) => {
                  const profile = activePool.profile;
                  if (!profile) return;
                  run(activePool.id, () => serverSave({ ...profile, keepAlive, autostart }));
                }
              : undefined
          }
        />
      ) : (
        <div className="flex-1 p-5 text-vsc-desc">
          {view === null ? t("Caricamento…") : t("Seleziona o crea un profilo server.")}
        </div>
      )}
      <p className="shrink-0 border-t border-vsc-border px-5 py-2 text-[12px] text-vsc-desc">
        {t(
          "Ogni pool Caddy o Apache può servire più progetti contemporaneamente tramite VirtualHost .localhost (RFC 6761, senza toccare il file hosts di Windows) e alias di percorso 100% portabili.",
        )}
      </p>
    </ViewContainer>
  );
}
