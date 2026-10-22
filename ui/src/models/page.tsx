// The Local models page (D-151): the llama.cpp server of the kit, on this PC
// only, and the GGUF models in data\models (shared by every system of the
// kit), from Atollo's catalog or any Hugging Face repository. As the
// Database page: the server and the models in the side bar, what to do in
// the editor. Downloads go on in the background, resume where they
// stopped, and are checked against the drive and their SHA-256.

import { useState } from "react";
import {
  folderOpen,
  llmHfInspect,
  llmHfTokenSet,
  llmKey,
  llmModels,
  llmPull,
  llmPullCancel,
  llmRemove,
  llmSet,
  llmStart,
  llmStatus,
  llmStop,
  onLlmPull,
  urlOpen,
  type CatalogModel,
  type HfRepo,
  type LlmInfo,
  type LocalModel,
  type ModelsView,
  type PullEvent,
  type ServiceState,
} from "../api";
import { locale, t } from "../i18n";
import { ComponentLogo } from "../logos";
import { ActionButton, ErrorLine } from "../common";
import { useAction, useCopy, useLoad, useSubscription } from "../hooks";
import {
  Button,
  CheckboxLabel,
  codicon,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  InlineConfirm,
  List,
  ListItem,
  Meter,
  Notice,
  PaneStack,
  Properties,
  Select,
  SideBarTitle,
  TextField,
  ViewContainer,
  ViewPane,
} from "../workbench";

/** Bytes as GB with one decimal, in the interface's language. */
export function gb(bytes: number): string {
  const n = new Intl.NumberFormat(locale(), { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(bytes / 1e9);
  return `${n} GB`;
}

/** Memory (RAM, video) in GB of 1024³ bytes, as the system shows it. */
export function mem(bytes: number): string {
  const n = new Intl.NumberFormat(locale(), { maximumFractionDigits: 0 }).format(bytes / GIB);
  return `${n} GB`;
}

const GIB = 1024 ** 3;

/** Whether a model needing `needGb` fits the PC: on the GPU with the most
 * memory or in the RAM (llama.cpp splits a model between the two). */
export type Fit = "ok" | "tight" | "no" | "unknown";

export function fitOf(needGb: number, hw: ModelsView["hardware"]): Fit {
  const gpu = Math.max(0, ...hw.gpus.map((g) => g.memory / GIB));
  const ram = (hw.ram ?? 0) / GIB;
  const room = Math.max(gpu, ram);
  if (room === 0) return "unknown";
  if (needGb <= room * 0.8) return "ok";
  return needGb <= room ? "tight" : "no";
}

/** The memory a model needs: the catalog's figure, else its size and a
 * margin for the context. */
function needOf(m: { bytes: number; ramGb?: number }): number {
  return m.ramGb ?? m.bytes / 1e9 + 1.5;
}

function FitText({ fit }: { fit: Fit }) {
  if (fit === "unknown") return null;
  const [tone, text] =
    fit === "ok"
      ? (["ok", t("entra")] as const)
      : fit === "tight"
        ? (["warn", t("al limite")] as const)
        : (["err", t("troppo grande per questo PC")] as const);
  return (
    <span className="flex items-center gap-1">
      <Dot tone={tone} />
      {text}
    </span>
  );
}

function stateText(info: LlmInfo | null): { tone: "ok" | "warn" | "err" | "off"; text: string } {
  const state: ServiceState = info?.state ?? "stopped";
  if (info && !info.installed) return { tone: "off", text: t("Non installato: si installa al primo avvio") };
  switch (state) {
    case "running":
      return { tone: "ok", text: t("Avviato") };
    case "starting":
      return { tone: "warn", text: t("Avvio in corso…") };
    case "error":
      return { tone: "err", text: t("Errore") };
    default:
      return { tone: "off", text: t("Fermo") };
  }
}

/** The key of a download, as Rust names it. */
function pullKey(repo: string, id: string): string {
  return `${repo.replace("/", "--")}/${id}`;
}

/** The progress of a download under way, or of one left halfway. */
function Progress({ done, total, label }: { done: number; total: number; label: string }) {
  const pct = total > 0 ? Math.floor((done * 100) / total) : 0;
  return (
    <span className="flex min-w-40 items-center gap-2">
      <Meter value={pct} label={label} valueText={`${String(pct)}%`} tone="ok" className="w-28" />
      <span className="text-vsc-desc tabular-nums">
        {gb(done)} / {gb(total)}
      </span>
    </span>
  );
}

export function ModelsPage() {
  const {
    data: info,
    error: infoError,
    reload: reloadInfo,
    setData: setInfo,
  } = useLoad<LlmInfo | null>(llmStatus, null, { onError: "show", every: 3000 });
  const {
    data: view,
    error: viewError,
    reload: reloadView,
    setData: setView,
  } = useLoad<ModelsView | null>(llmModels, null, { onError: "show", every: 10000 });
  const { busy, error, run } = useAction();
  const [events, setEvents] = useState<Record<string, PullEvent>>({});
  const [confirm, setConfirm] = useState<string | null>(null);
  const [repo, setRepo] = useState("");
  const [found, setFound] = useState<HfRepo | null>(null);
  const [token, setToken] = useState("");
  const [listOpen, setListOpen] = useState(true);
  const { copied, copy } = useCopy(1500);

  useSubscription(onLlmPull, (e) => {
    setEvents((all) => ({ ...all, [e.key]: e }));
    if (e.state !== "running") reloadView();
  });

  const onInfo = (action: () => Promise<LlmInfo>) => () =>
    action().then(
      (i) => {
        setInfo(i);
        reloadView();
      },
      (e: unknown) => {
        reloadInfo();
        throw e;
      },
    );

  const pull = (key: string, what: Parameters<typeof llmPull>[0]) => {
    run(`pull:${key}`, () =>
      llmPull(what).then(
        () => {
          reloadView();
          reloadInfo();
        },
        (e: unknown) => {
          reloadView();
          throw e;
        },
      ),
    );
  };

  const st = stateText(info);
  const on = info?.state === "running" || info?.state === "starting";
  const local = view?.local ?? [];
  const complete = local.filter((m) => m.complete);
  const pulling = new Set(view?.pulling ?? []);
  const live = (key: string): PullEvent | undefined => {
    const e = events[key];
    return e && (e.state === "running" || pulling.has(key)) ? e : undefined;
  };
  const fat32 = view?.fs?.toUpperCase() === "FAT32";

  /** Download, progress with «Sospendi», or what is on disk. */
  const downloadControls = (key: string, start: () => void, partial: LocalModel | undefined) => {
    const e = live(key);
    if (e || pulling.has(key) || busy === `pull:${key}`) {
      return (
        <span className="flex flex-wrap items-center gap-2">
          {e && <Progress done={e.done} total={e.total} label={t("Scaricamento")} />}
          <Button
            small
            onClick={() => {
              void llmPullCancel(key);
            }}
          >
            {t("Sospendi")}
          </Button>
        </span>
      );
    }
    if (partial && !partial.complete) {
      return (
        <span className="flex flex-wrap items-center gap-2">
          <Progress done={partial.downloaded} total={partial.bytes} label={t("Scaricamento sospeso")} />
          <Button small variant="primary" onClick={start}>
            {t("Riprendi")}
          </Button>
        </span>
      );
    }
    return (
      <Button small variant="primary" onClick={start}>
        {t("Scarica")}
      </Button>
    );
  };

  const removeControls = (m: LocalModel) =>
    confirm === m.id ? (
      <InlineConfirm
        row
        question={t("Cancellare {0}?", m.name)}
        onCancel={() => {
          setConfirm(null);
        }}
        actions={
          <>
            <ActionButton
              icon={codicon("trash")}
              label={t("Cancella")}
              primary
              busy={busy === `rm:${m.id}`}
              onClick={() => {
                setConfirm(null);
                run(`rm:${m.id}`, () => llmRemove(m.id).then(setView));
              }}
            />
            <Button
              onClick={() => {
                setConfirm(null);
              }}
            >
              {t("Annulla")}
            </Button>
          </>
        }
      />
    ) : (
      <IconButton
        icon={codicon("trash")}
        label={t("Cancella {0}", m.name)}
        title={t("Cancella il modello dal kit")}
        onClick={() => {
          setConfirm(m.id);
        }}
      />
    );

  const catalogRow = (c: CatalogModel) => {
    const here = local.find((m) => m.catalog === c.id || m.id === c.id);
    const key = pullKey(c.repo, here?.id ?? c.id);
    const tooBig = fat32 && c.files.some((f) => f.bytes > 4294967295);
    return (
      <li key={c.id} className="flex flex-wrap items-start gap-3 border-b border-vsc-border py-2 last:border-b-0">
        <div className="min-w-0 flex-1">
          <p className="font-semibold">
            {c.name}{" "}
            <span className="font-normal text-vsc-desc">
              {c.quant} · {gb(c.bytes)} · {c.license}
            </span>
          </p>
          <p className="text-vsc-desc">{c.desc}</p>
          {view && (
            <p className="flex flex-wrap items-center gap-3 text-[12px] text-vsc-desc">
              <FitText fit={fitOf(needOf(c), view.hardware)} />
              <span>{t("circa {0} GB di memoria", String(c.ramGb))}</span>
              <span>
                {c.tools
                  ? t("usa gli strumenti (adatto agli agenti)")
                  : t("solo chat: con gli agenti non usa bene gli strumenti")}
              </span>
            </p>
          )}
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {here?.complete ? (
            <>
              <span className="flex items-center gap-1 text-vsc-desc">
                <Dot tone="ok" />
                {t("nel kit")}
              </span>
              {removeControls(here)}
            </>
          ) : tooBig ? (
            <span className="text-[12px] text-vsc-desc">{t("non entra su FAT32")}</span>
          ) : (
            downloadControls(
              key,
              () => {
                pull(key, { catalog: c.id });
              },
              here,
            )
          )}
        </div>
      </li>
    );
  };

  const others = local.filter((m) => !view?.catalog.some((c) => c.id === m.catalog || c.id === m.id));

  const side = (
    <>
      <SideBarTitle
        title={t("Modelli locali")}
        actions={
          <>
            <IconButton
              icon={codicon("refresh")}
              label={t("Aggiorna")}
              onClick={() => {
                reloadInfo();
                reloadView();
              }}
            />
            <IconButton
              icon={codicon("folder-opened")}
              label={t("Apri la cartella dei modelli")}
              onClick={() => {
                run("folder", () => folderOpen("models"));
              }}
            />
          </>
        }
      />
      <PaneStack>
        <ViewPane
          first
          fill
          title={t("Nel kit")}
          badge={complete.length > 0 ? complete.length : undefined}
          expanded={listOpen}
          onToggle={() => {
            setListOpen(!listOpen);
          }}
        >
          <List label={t("Modelli locali")}>
            <ListItem
              icon={<ComponentLogo id="llama" size={32} />}
              name="llama.cpp"
              meta={info ? `:${String(info.port)}` : undefined}
              description={
                <span className="flex items-center gap-1.5">
                  <Dot tone={st.tone} />
                  {st.text}
                </span>
              }
              footer={info?.loaded.length ? t("In memoria: {0}", info.loaded.join(", ")) : undefined}
              actions={
                <Button
                  small
                  variant={on ? "secondary" : "primary"}
                  busy={busy === "server"}
                  disabled={!on && complete.length === 0}
                  onClick={() => {
                    run("server", onInfo(on ? llmStop : llmStart));
                  }}
                >
                  {on ? t("Ferma") : t("Avvia")}
                </Button>
              }
            />
            {local.map((m) => (
              <ListItem
                key={`${m.folder}/${m.id}`}
                icon={<ComponentLogo id="llama" size={24} />}
                name={m.name}
                meta={gb(m.bytes)}
                description={m.repo || m.folder}
                footer={
                  !m.complete
                    ? t("scaricamento sospeso")
                    : info?.piModel === m.id
                      ? t("modello di Pi")
                      : info?.loaded.includes(m.id)
                        ? t("in memoria")
                        : undefined
                }
              />
            ))}
          </List>
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="models" side={side}>
      <div className="flex min-h-0 flex-1 flex-col">
        <EditorHeader
          icon={<ComponentLogo id="llama" size={56} />}
          title={t("Modelli locali")}
          meta={[
            <span key="state" className="flex items-center gap-1">
              <Dot tone={st.tone} />
              {st.text}
            </span>,
            info ? <span key="url">{info.url}</span> : null,
          ]}
          description={t(
            "I modelli linguistici sul PC, senza rete e senza account: li usa Pi, e una chat nel browser. Il server ascolta solo su questo PC (127.0.0.1), con una chiave.",
          )}
          actions={
            <>
              <ActionButton
                icon={codicon(on ? "debug-stop" : "play")}
                label={on ? t("Ferma") : t("Avvia")}
                primary={!on}
                busy={busy === "server"}
                disabled={!on && complete.length === 0}
                title={complete.length === 0 ? t("Scarica prima un modello") : undefined}
                onClick={() => {
                  run("server", onInfo(on ? llmStop : llmStart));
                }}
              />
              <ActionButton
                icon={codicon("comment-discussion")}
                label={t("Apri la chat")}
                disabled={info?.state !== "running"}
                title={t("La chat di llama.cpp nel browser: nelle sue impostazioni incolla la chiave copiata qui")}
                onClick={() => {
                  if (info) run("chat", () => urlOpen(info.url));
                }}
              />
              <ActionButton
                icon={codicon("key")}
                label={copied ? t("Copiata") : t("Copia la chiave")}
                onClick={() => {
                  run("key", () => llmKey().then(copy));
                }}
              />
            </>
          }
        />
        <EditorBody>
          <ErrorLine error={infoError ?? viewError} />
          <ErrorLine error={error} />
          {info?.error && <Notice tone="error">{info.error}</Notice>}

          <EditorSection id="llm-server" title={t("Server")}>
            <Properties
              rows={[
                [t("Indirizzo"), <span className="font-mono">{info?.url ?? "-"}</span>],
                [t("In memoria"), info?.loaded.length ? info.loaded.join(", ") : t("nessun modello")],
                [
                  t("Avvio"),
                  <CheckboxLabel
                    checked={info?.autostart ?? false}
                    onChange={(v) => {
                      run(
                        "autostart",
                        onInfo(() => llmSet({ autostart: v })),
                      );
                    }}
                  >
                    {t("Avvia con Atollo")}
                  </CheckboxLabel>,
                ],
              ]}
            />
            <p className="text-[12px] text-vsc-desc">
              {t(
                "Un modello si carica quando un agente o la chat lo chiede, uno alla volta, e si libera dopo dieci minuti senza richieste.",
              )}
            </p>
          </EditorSection>

          <EditorSection id="llm-pi" title={t("Modello di Pi")}>
            {complete.length === 0 ? (
              <p className="text-vsc-desc">{t("Scarica un modello qui sotto: Pi lo usa senza altre impostazioni.")}</p>
            ) : (
              <label className="flex flex-wrap items-center gap-2">
                <span>{t("Pi parte con")}</span>
                <Select
                  value={info?.piModel ?? ""}
                  onChange={(v) => {
                    run(
                      "pi",
                      onInfo(() => llmSet({ piModel: v })),
                    );
                  }}
                  options={[
                    { value: "", label: t("il primo modello del kit") },
                    ...complete.map((m) => ({ value: m.id, label: m.name })),
                  ]}
                />
              </label>
            )}
            <p className="text-[12px] text-vsc-desc">
              {t(
                "Con un provider in cloud scelto in Pi (/login), Pi resta su quello; i modelli locali restano nel suo elenco (/model).",
              )}
            </p>
          </EditorSection>

          <EditorSection id="llm-catalog" title={t("Catalogo di Atollo")}>
            {fat32 && (
              <Notice tone="warning">
                {t(
                  "Il disco del kit è FAT32: non tiene file oltre i 4 GB. Su exFAT o NTFS entrano tutti; qui solo i modelli in parti più piccole.",
                )}
              </Notice>
            )}
            {view === null ? (
              <p className="text-vsc-desc">{t("Caricamento…")}</p>
            ) : (
              <ul className="m-0 list-none p-0">{view.catalog.map(catalogRow)}</ul>
            )}
          </EditorSection>

          {others.length > 0 && (
            <EditorSection id="llm-others" title={t("Altri modelli nel kit")}>
              <ul className="m-0 list-none p-0">
                {others.map((m) => {
                  const key = pullKey(m.repo || m.folder, m.id);
                  return (
                    <li
                      key={`${m.folder}/${m.id}`}
                      className="flex flex-wrap items-center gap-3 border-b border-vsc-border py-2 last:border-b-0"
                    >
                      <div className="min-w-0 flex-1">
                        <p className="font-semibold">
                          {m.name} <span className="font-normal text-vsc-desc">{gb(m.bytes)}</span>
                        </p>
                        <p className="text-[12px] text-vsc-desc">
                          {m.byHand ? t("messo a mano in {0}", m.folder) : m.repo}
                        </p>
                      </div>
                      {m.complete || m.byHand
                        ? removeControls(m)
                        : downloadControls(
                            key,
                            () => {
                              if (m.repo) pull(key, { repo: m.repo, choice: m.id });
                            },
                            m,
                          )}
                    </li>
                  );
                })}
              </ul>
            </EditorSection>
          )}

          <EditorSection id="llm-hf" title={t("Da Hugging Face")}>
            <form
              className="flex flex-wrap items-center gap-2"
              onSubmit={(e) => {
                e.preventDefault();
                setFound(null);
                run("hf", () => llmHfInspect(repo.trim()).then(setFound));
              }}
            >
              <TextField
                value={repo}
                onChange={setRepo}
                label={t("Repository di Hugging Face")}
                placeholder="unsloth/Qwen3-8B-GGUF"
                mono
                className="w-80"
              />
              <ActionButton
                icon={codicon("search")}
                label={t("Cerca i file GGUF")}
                submit
                busy={busy === "hf"}
                disabled={!repo.includes("/")}
                onClick={() => undefined}
              />
            </form>
            {found && (
              <div className="space-y-1">
                <p className="text-[12px] text-vsc-desc">
                  {found.license
                    ? t("Licenza: {0}", found.license)
                    : t("Licenza non indicata: leggila sulla pagina del modello")}
                </p>
                <ul className="m-0 list-none p-0">
                  {found.choices.map((c) => {
                    const key = pullKey(found.repo, c.id);
                    const partial = local.find((m) => m.repo === found.repo && m.id === c.id);
                    const tooBig = fat32 && c.files.some((f) => f.bytes > 4294967295);
                    return (
                      <li
                        key={c.id}
                        className="flex flex-wrap items-center gap-3 border-b border-vsc-border py-1.5 last:border-b-0"
                      >
                        <span className="min-w-0 flex-1">
                          <span className="font-mono">{c.quant ?? c.label}</span>{" "}
                          <span className="text-vsc-desc">
                            {gb(c.bytes)}
                            {c.files.length > 1 ? ` · ${t("{0} parti", String(c.files.length))}` : ""}
                          </span>{" "}
                          {view && <FitText fit={fitOf(needOf(c), view.hardware)} />}
                        </span>
                        {partial?.complete ? (
                          <span className="text-vsc-desc">{t("nel kit")}</span>
                        ) : tooBig ? (
                          <span className="text-[12px] text-vsc-desc">{t("non entra su FAT32")}</span>
                        ) : (
                          downloadControls(
                            key,
                            () => {
                              pull(key, { repo: found.repo, choice: c.id });
                            },
                            partial,
                          )
                        )}
                      </li>
                    );
                  })}
                </ul>
              </div>
            )}
            <form
              className="flex flex-wrap items-center gap-2"
              onSubmit={(e) => {
                e.preventDefault();
                run("token", () =>
                  llmHfTokenSet(token).then(() => {
                    setToken("");
                    reloadView();
                  }),
                );
              }}
            >
              <TextField
                value={token}
                onChange={setToken}
                type="password"
                label={t("Token di Hugging Face")}
                placeholder={view?.hfToken ? t("salvato nel kit") : "hf_…"}
                className="w-80"
              />
              <Button submit busy={busy === "token"} disabled={token.trim() === "" && !view?.hfToken}>
                {token.trim() === "" && view?.hfToken ? t("Togli il token") : t("Salva il token")}
              </Button>
            </form>
            <p className="text-[12px] text-vsc-desc">
              {t(
                "Il token serve solo per i modelli ad accesso controllato; va solo a huggingface.co ed è salvato nel kit, leggibile solo dal tuo utente.",
              )}
            </p>
          </EditorSection>

          {view && (
            <EditorSection id="llm-pc" title={t("Questo PC")}>
              <Properties
                rows={[
                  [t("Memoria (RAM)"), view.hardware.ram ? mem(view.hardware.ram) : "-"],
                  [
                    t("Schede video"),
                    view.hardware.gpus.length
                      ? view.hardware.gpus.map((g) => `${g.name} (${mem(g.memory)})`).join(", ")
                      : info?.installed
                        ? t("nessuna usabile: si usa la CPU")
                        : t("si vedono dopo l'installazione di llama.cpp"),
                  ],
                  [
                    t("Disco del kit"),
                    `${view.fs ?? "-"}${view.free !== null ? ` · ${t("liberi {0}", gb(view.free))}` : ""}`,
                  ],
                  [t("Cartella"), <span className="font-mono">{view.folder}</span>],
                ]}
              />
            </EditorSection>
          )}
        </EditorBody>
      </div>
    </ViewContainer>
  );
}
