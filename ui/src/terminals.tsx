// The Terminals page: kit consoles that open in any folder of the PC. The
// default one has every tool on disk at its newest version; the user's own
// include the tools they choose, each at the newest version or at another
// one on disk (tauri/src/terminals.rs).

import { useCallback, useState } from "react";
import {
  terminalDirPick,
  terminalLaunch,
  terminalRemove,
  terminalSave,
  terminalsList,
  type Terminal,
  type TerminalInfo,
  type TerminalTool,
  type TerminalToolGroup,
  type TerminalsView,
} from "./api";
import { ActionButton, ErrorLine } from "./common";
import { useAction, useLoad } from "./hooks";
import { t } from "./i18n";
import { ComponentLogo } from "./logos";
import {
  Button,
  Checkbox,
  Codicon,
  codicon,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  InlineConfirm,
  Link,
  List,
  ListItem,
  Notice,
  PaneStack,
  Select,
  SideBarTitle,
  TextField,
  ViewContainer,
  ViewPane,
} from "./workbench";

const GROUPS: { id: TerminalToolGroup; label: () => string }[] = [
  { id: "agents", label: () => t("Agenti") },
  { id: "ai", label: () => t("Modelli locali") },
  { id: "languages", label: () => t("Linguaggi") },
  { id: "databases", label: () => t("Database") },
];

/** The terminal as saved, without what only the page shows. */
function plain(x: Terminal): Terminal {
  return { id: x.id, name: x.name, dir: x.dir, tools: { ...x.tools } };
}

function same(a: Terminal, b: Terminal): boolean {
  const keys = (x: Terminal) => Object.keys(x.tools).sort();
  return (
    a.name === b.name &&
    a.dir === b.dir &&
    keys(a).join() === keys(b).join() &&
    keys(a).every((k) => a.tools[k] === b.tools[k])
  );
}

/** "Node.js 22.21", or the name alone for the newest version. */
function toolText(tool: TerminalTool | undefined, id: string, line: string | null): string {
  const name = tool?.name ?? id;
  return line ? `${name} ${line}` : name;
}

function ToolRow({
  tool,
  included,
  line,
  readOnly,
  onChange,
}: {
  tool: TerminalTool;
  included: boolean;
  line: string | null;
  readOnly: boolean;
  onChange: (included: boolean, line: string | null) => void;
}) {
  const id = `term-tool-${tool.id}`;
  const newest = tool.lines[0];
  return (
    <div className="flex min-h-[26px] items-center gap-2">
      <Checkbox
        id={id}
        checked={included}
        disabled={readOnly}
        onChange={(v) => {
          onChange(v, line);
        }}
      />
      <ComponentLogo id={tool.id} size={16} />
      <label htmlFor={id} className="w-36 shrink-0 cursor-pointer truncate">
        {tool.name}
      </label>
      {tool.lines.length > 0 && (
        <Select
          label={t("Versione di {0}", tool.name)}
          value={line ?? ""}
          disabled={readOnly || !included}
          onChange={(v) => {
            onChange(true, v === "" ? null : v);
          }}
          options={[
            { value: "", label: newest ? t("La più recente ({0})", newest) : t("La più recente") },
            ...tool.lines.map((l) => ({
              value: l,
              label: tool.versions[l] ? `${l} (${tool.versions[l]})` : l,
            })),
          ]}
          className="w-56"
        />
      )}
    </div>
  );
}

function TerminalEditor({
  view,
  saved,
  onSaved,
  onRemoved,
  onDuplicate,
  onTools,
}: {
  view: TerminalsView;
  /** The terminal as saved; a new one has an empty id. */
  saved: TerminalInfo;
  onSaved: (id: string) => void;
  onRemoved: () => void;
  onDuplicate: (draft: TerminalInfo) => void;
  onTools?: (() => void) | undefined;
}) {
  const [draft, setDraft] = useState<Terminal>(() => plain(saved));
  const [confirmRemove, setConfirmRemove] = useState(false);
  const { busy, error, run } = useAction();
  const isNew = saved.id === "";
  const readOnly = saved.builtin;
  const dirty = !readOnly && (isNew || !same(draft, saved));
  const count = Object.keys(draft.tools).filter((id) => view.tools.some((x) => x.id === id)).length;
  const byId = new Map(view.tools.map((x) => [x.id, x]));

  const setTool = (id: string, included: boolean, line: string | null) => {
    setDraft((d) => {
      const tools = Object.fromEntries(Object.entries(d.tools).filter(([k]) => k !== id));
      if (included) tools[id] = line;
      return { ...d, tools };
    });
  };
  const setAll = (included: boolean) => {
    setDraft((d) => ({
      ...d,
      tools: included ? Object.fromEntries(view.tools.map((x) => [x.id, d.tools[x.id] ?? null])) : {},
    }));
  };
  /** Saves the changes first, when there are any; resolves to the id. */
  const persist = async (): Promise<string> => {
    if (!dirty) return saved.id;
    const id = await terminalSave(draft);
    onSaved(id);
    return id;
  };
  const open = (pick: boolean) => {
    run(pick ? "open-in" : "open", async () => {
      let dir: string | null = null;
      if (pick) {
        dir = await terminalDirPick(draft.dir ?? view.projects);
        if (dir === null) return;
      }
      const id = await persist();
      await terminalLaunch(id, dir);
    });
  };
  const chooseDir = () => {
    run("dir", async () => {
      const dir = await terminalDirPick(draft.dir ?? view.projects);
      if (dir !== null) {
        setDraft((d) => ({ ...d, dir }));
      }
    });
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<Codicon name="terminal" size={48} className="text-vsc-desc" />}
        title={draft.name.trim() || t("Nuovo terminale")}
        titleId="terminal-title"
        meta={[
          <span key="tools">{t("{0} strumenti", count)}</span>,
          <span key="dir" className="truncate font-mono text-[12px]" title={draft.dir ?? view.projects}>
            {draft.dir ?? view.projects}
          </span>,
        ]}
        description={
          readOnly
            ? t(
                "Tutti gli strumenti installati, ognuno alla versione più recente. Non si modifica: duplicalo per crearne uno tuo.",
              )
            : undefined
        }
        actions={
          <>
            <ActionButton
              icon={codicon("terminal")}
              label={dirty ? t("Salva e apri") : t("Apri")}
              primary
              busy={busy === "open"}
              disabled={busy !== null || draft.name.trim() === ""}
              onClick={() => {
                open(false);
              }}
            />
            <ActionButton
              icon={codicon("folder-opened")}
              label={t("Apri in…")}
              busy={busy === "open-in"}
              disabled={busy !== null || draft.name.trim() === ""}
              onClick={() => {
                open(true);
              }}
            />
            {!isNew && (
              <ActionButton
                icon={codicon("copy")}
                label={t("Duplica")}
                disabled={busy !== null}
                onClick={() => {
                  onDuplicate({
                    ...plain(draft),
                    id: "",
                    name: t("{0} (copia)", draft.name),
                    builtin: false,
                    missing: [],
                    replaced: [],
                  });
                }}
              />
            )}
            {!readOnly && !isNew && (
              <ActionButton
                icon={codicon("trash")}
                label={t("Elimina")}
                disabled={busy !== null}
                onClick={() => {
                  setConfirmRemove(true);
                }}
              />
            )}
          </>
        }
      />
      <EditorBody>
        {confirmRemove && (
          <InlineConfirm
            label={t("Elimina il terminale")}
            onCancel={() => {
              setConfirmRemove(false);
            }}
            actions={
              <>
                <ActionButton
                  icon={codicon("trash")}
                  label={t("Elimina")}
                  primary
                  busy={busy === "remove"}
                  onClick={() => {
                    run("remove", async () => {
                      await terminalRemove(saved.id);
                      onRemoved();
                    });
                  }}
                />
                <ActionButton
                  icon={codicon("refresh")}
                  label={t("Annulla")}
                  onClick={() => {
                    setConfirmRemove(false);
                  }}
                />
              </>
            }
          >
            <Notice tone="warning">
              {t("Eliminare il terminale «{0}»? Strumenti, versioni e cartelle restano.", saved.name)}
            </Notice>
          </InlineConfirm>
        )}
        <ErrorLine error={error} />
        {!readOnly && (
          <EditorSection id="terminal-general" title={t("Generale")}>
            <div className="max-w-xl space-y-3">
              <div>
                <label htmlFor="terminal-name" className="mb-1 block font-semibold">
                  {t("Nome")}
                </label>
                <TextField
                  id="terminal-name"
                  value={draft.name}
                  autoFocus={isNew}
                  onChange={(name) => {
                    setDraft((d) => ({ ...d, name }));
                  }}
                  placeholder="PHP 7.4 legacy"
                />
              </div>
              <div>
                <label htmlFor="terminal-dir" className="mb-1 block font-semibold">
                  {t("Cartella di partenza")}
                </label>
                <div className="flex items-center gap-1.5">
                  <TextField
                    id="terminal-dir"
                    value={draft.dir ?? ""}
                    onChange={(v) => {
                      setDraft((d) => ({ ...d, dir: v.trim() === "" ? null : v }));
                    }}
                    mono
                    placeholder={view.projects}
                    className="min-w-0 flex-1"
                  />
                  <Button small variant="secondary" busy={busy === "dir"} onClick={chooseDir}>
                    {t("Scegli…")}
                  </Button>
                  {draft.dir !== null && (
                    <Button
                      small
                      variant="secondary"
                      onClick={() => {
                        setDraft((d) => ({ ...d, dir: null }));
                      }}
                    >
                      {t("Usa projects")}
                    </Button>
                  )}
                </div>
                <p className="mt-1 text-[12px] text-vsc-desc">
                  {t("Qualsiasi cartella del PC. «Apri in…» la sceglie solo per una volta.")}
                </p>
              </div>
            </div>
          </EditorSection>
        )}
        <EditorSection
          id="terminal-tools"
          title={t("Strumenti e versioni")}
          actions={
            readOnly ? undefined : (
              <span className="flex gap-2 text-[12px]">
                <Link
                  onClick={() => {
                    setAll(true);
                  }}
                >
                  {t("Tutti")}
                </Link>
                <Link
                  onClick={() => {
                    setAll(false);
                  }}
                >
                  {t("Nessuno")}
                </Link>
              </span>
            )
          }
        >
          <p className="text-vsc-desc">
            {t(
              "Gli strumenti non scelti restano fuori dal PATH. Sempre presenti: PowerShell 7, GitHub CLI, i comandi del kit e quelli di Windows.",
            )}
          </p>
          {saved.missing.length > 0 && (
            <Notice tone="warning">
              {t(
                "Non più installati, lasciati fuori: {0}.",
                saved.missing.map((id) => toolText(byId.get(id), id, saved.tools[id] ?? null)).join(", "),
              )}
            </Notice>
          )}
          {saved.replaced.length > 0 && (
            <Notice tone="warning">
              {t(
                "Versioni non più installate, al loro posto la più recente: {0}.",
                saved.replaced.map((r) => `${toolText(byId.get(r.id), r.id, r.saved)} → ${r.used}`).join(", "),
              )}
            </Notice>
          )}
          {view.tools.length === 0 ? (
            <Notice tone="info">{t("Nessuno strumento installato.")}</Notice>
          ) : (
            <div className="space-y-4">
              {GROUPS.map((g) => {
                const tools = view.tools.filter((x) => x.group === g.id);
                if (tools.length === 0) return null;
                return (
                  <div key={g.id} role="group" aria-label={g.label()} className="space-y-1.5">
                    <h4 className="text-[12px] font-semibold text-vsc-desc uppercase">{g.label()}</h4>
                    {tools.map((tool) => (
                      <ToolRow
                        key={tool.id}
                        tool={tool}
                        included={tool.id in draft.tools}
                        line={draft.tools[tool.id] ?? null}
                        readOnly={readOnly}
                        onChange={(included, line) => {
                          setTool(tool.id, included, line);
                        }}
                      />
                    ))}
                  </div>
                );
              })}
            </div>
          )}
          {onTools && (
            <p className="text-[12px] text-vsc-desc">
              {t("Altri strumenti o versioni:")} <Link onClick={onTools}>{t("Strumenti")}</Link>
            </p>
          )}
        </EditorSection>
      </EditorBody>
      {dirty && !isNew && (
        <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-vsc-border px-5 py-2">
          <span role="status" className="min-w-0 flex-1 text-vsc-desc">
            {t("Modifiche non salvate")}
          </span>
          <ActionButton
            icon={codicon("save")}
            label={t("Salva")}
            primary
            busy={busy === "save"}
            disabled={busy !== null || draft.name.trim() === ""}
            onClick={() => {
              run("save", async () => {
                await persist();
              });
            }}
          />
          <ActionButton
            icon={codicon("refresh")}
            label={t("Annulla")}
            disabled={busy !== null}
            onClick={() => {
              setDraft(plain(saved));
            }}
          />
        </div>
      )}
      {isNew && (
        <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-vsc-border px-5 py-2">
          <span className="min-w-0 flex-1" />
          <ActionButton
            icon={codicon("save")}
            label={t("Crea terminale")}
            primary
            busy={busy === "save"}
            disabled={busy !== null || draft.name.trim() === ""}
            onClick={() => {
              run("save", async () => {
                await persist();
              });
            }}
          />
        </div>
      )}
    </div>
  );
}

export function TerminalsPage({
  onTools,
  onGuide,
}: { onTools?: () => void; onGuide?: (topic: "terminals") => void } = {}) {
  const {
    data: view,
    error: loadError,
    reload,
  } = useLoad<TerminalsView | null>(terminalsList, null, {
    onError: "show",
  });
  const [chosen, setChosen] = useState<string | null>(null);
  /** A terminal being created, not saved yet. */
  const [draft, setDraft] = useState<TerminalInfo | null>(null);
  const [listOpen, setListOpen] = useState(true);
  const { busy, error, run } = useAction();

  const terminals = view?.terminals ?? [];
  const current = draft ?? terminals.find((x) => x.id === chosen) ?? terminals[0] ?? null;
  const toolsById = new Map((view?.tools ?? []).map((x) => [x.id, x]));

  const startNew = useCallback(() => {
    setDraft({ id: "", name: "", dir: null, tools: {}, builtin: false, missing: [], replaced: [] });
  }, []);

  const summary = (x: TerminalInfo) => {
    if (x.builtin) return t("Ogni strumento alla versione più recente");
    const parts = Object.entries(x.tools)
      .filter(([id]) => toolsById.has(id))
      .map(([id, line]) => toolText(toolsById.get(id), id, line));
    return parts.length === 0 ? t("Solo PowerShell e i comandi del kit") : parts.join(", ");
  };

  const side = (
    <>
      <SideBarTitle
        title={t("Terminali")}
        actions={
          <>
            <IconButton icon={codicon("add")} label={t("Nuovo terminale")} onClick={startNew} />
            <IconButton icon={codicon("refresh")} label={t("Aggiorna")} onClick={reload} />
            {onGuide && (
              <IconButton
                icon={codicon("book")}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("terminals");
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
          title={t("Terminali")}
          expanded={listOpen}
          onToggle={() => {
            setListOpen(!listOpen);
          }}
        >
          {view === null ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
          ) : (
            <List label={t("Terminali")}>
              {terminals.map((x) => (
                <ListItem
                  key={x.id}
                  icon={<Codicon name="terminal" size={28} className="text-vsc-desc" />}
                  name={x.name}
                  meta={x.builtin ? t("predefinito") : undefined}
                  description={summary(x)}
                  title={summary(x)}
                  selected={draft === null && current?.id === x.id}
                  onSelect={() => {
                    setDraft(null);
                    setChosen(x.id);
                  }}
                  actions={
                    <Button
                      small
                      variant="secondary"
                      busy={busy === x.id}
                      onClick={() => {
                        run(x.id, () => terminalLaunch(x.id));
                      }}
                    >
                      {t("Apri")}
                    </Button>
                  }
                />
              ))}
            </List>
          )}
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="terminals" side={side}>
      <div className="px-5 pt-3 empty:hidden">
        <ErrorLine error={loadError ?? error} />
      </div>
      {view && current ? (
        <TerminalEditor
          key={draft ? `new:${draft.name}` : current.id}
          view={view}
          saved={current}
          onSaved={(id) => {
            setDraft(null);
            setChosen(id);
            reload();
          }}
          onRemoved={() => {
            setChosen(null);
            reload();
          }}
          onDuplicate={setDraft}
          onTools={onTools}
        />
      ) : (
        <div className="flex-1 p-5 text-vsc-desc">{t("Caricamento…")}</div>
      )}
    </ViewContainer>
  );
}
