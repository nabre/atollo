// The versions of a tool in its row of the Tools page: the kit's line, the
// lines on disk with who uses them, the menu to download another.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { VscAdd, VscCloudDownload, VscLock, VscTrash } from "react-icons/vsc";
import { versionsAdd, versionsList, versionsRemove, type SetupSnapshot, type VersionedTool } from "../api";
import { t } from "../i18n";
import { useAction, useLoad } from "../hooks";
import { ActionButton, ErrorLine } from "../common";
import { ItemBar, ItemStatus, itemState } from "../setup";
import { ByHandBox } from "../byhand";
import { Tag } from "../tags";
import { isPresent, lineText, offered, splitVersion, useOffers, useReloadOnSteps } from "./model";
import { LineState, RefreshLines } from "./lines";

/** The lines on disk of a tool, with who uses them. Each can go while
 * another stays (the newest left becomes the default); the last goes with
 * the tool, by its checkbox. */
function DiskLines({
  tool,
  setup,
  busy,
  running,
  outside,
  onRemove,
}: {
  tool: VersionedTool;
  setup: SetupSnapshot | null;
  outside: Record<string, string[]>;
  busy: string | null;
  running: boolean;
  onRemove: (line: string) => void;
}) {
  const rows = [...(tool.default ? [tool.default] : []), ...tool.saved];
  // Lines the preparation is downloading or will download, not on disk yet.
  const prefix = `${tool.name} `;
  const coming = [...(setup?.running ? setup.queue : []), ...(setup?.steps ?? [])]
    .filter((s) => s.id === tool.id && s.name.startsWith(prefix))
    .map((s) => s.name.slice(prefix.length))
    .filter((line, i, all) => all.indexOf(line) === i && !isPresent(tool, line))
    .filter((line) => {
      const state = itemState(setup, tool.id, prefix + line);
      // A failed download stays, to be done in the browser.
      return state !== null && (state.kind !== "problem" || Boolean(state.byHand));
    });
  if (rows.length === 0 && coming.length === 0) return null;
  return (
    <ul className="mt-1 space-y-0.5" aria-label={t("Versioni di {0}", tool.name)}>
      {rows.map((r) => {
        const users = tool.projects[r.line] ?? [];
        const state = itemState(setup, tool.id, prefix + r.line);
        return (
          <li key={r.line}>
            <div className="flex min-h-[22px] items-center gap-2">
              <span className="min-w-0 truncate font-mono text-[12px]">{lineText(r.line, r.version)}</span>
              {/* What comes with this version (Maven, Gradle with Java); Composer has its row. */}
              {splitVersion(r.version)
                .parts.filter((p) => !p.startsWith("Composer "))
                .map((p) => (
                  <Tag key={p}>{p}</Tag>
                ))}
              {/* The newest line its source offers. */}
              {tool.available[0] === r.line && <Tag tone="focus">latest</Tag>}
              {(outside[r.line] ?? []).length > 0 && (
                <Tag tone="warning">{t("non per {0}", (outside[r.line] ?? []).join(", "))}</Tag>
              )}
              <span className="flex-1" />
              {state && (
                <span className="text-[12px]">
                  <ItemStatus state={state} />
                </span>
              )}
              {/* Locked by the projects that use it; otherwise it can go. */}
              {users.length > 0 ? (
                <Locked projects={users} label={t("{0} {1}: progetti che la usano", tool.name, r.line)} />
              ) : (
                rows.length > 1 && (
                  <ActionButton
                    icon={VscTrash}
                    label={t("Rimuovi {0} {1}", tool.name, r.line)}
                    compact
                    busy={busy === `remove:${tool.id}:${r.line}`}
                    disabled={running || busy !== null}
                    onClick={() => {
                      onRemove(r.line);
                    }}
                  />
                )
              )}
            </div>
            {state && <ItemBar state={state} label={prefix + r.line} />}
            {state?.kind === "problem" && <p className="text-[12px] text-vsc-error select-text">{state.text}</p>}
            {state?.kind === "problem" && state.byHand && (
              <ByHandBox
                id={tool.id}
                name={prefix + r.line}
                byHand={state.byHand}
                disabled={running}
                onRetry={() => versionsAdd(tool.id, r.line)}
              />
            )}
          </li>
        );
      })}
      {coming.map((line) => {
        const state = itemState(setup, tool.id, prefix + line);
        return (
          <li key={`coming-${line}`}>
            <div className="flex min-h-[22px] items-center gap-2">
              <span className="min-w-0 truncate font-mono text-[12px] text-vsc-desc">{line}</span>
              <span className="flex-1" />
              {state && (
                <span className="text-[12px]">
                  <ItemStatus state={state} />
                </span>
              )}
            </div>
            {state && <ItemBar state={state} label={prefix + line} />}
            {state?.kind === "problem" && state.byHand && (
              <>
                <p className="text-[12px] text-vsc-error select-text">{state.text}</p>
                <ByHandBox
                  id={tool.id}
                  name={prefix + line}
                  byHand={state.byHand}
                  disabled={running}
                  onRetry={() => versionsAdd(tool.id, line)}
                />
              </>
            )}
          </li>
        );
      })}
    </ul>
  );
}

/** The look of vscode-context-menu (5px corners, menu colors, the widget
 * shadow) and of its items (2em, 3px corners, the selection colors). The
 * menus stay native: the component has no menu roles, and its items hold
 * only a label and a keybinding, not the groups and tags these show. */
const MENU =
  "absolute top-full right-0 z-20 mt-1 rounded-[5px] border border-vsc-menu-border bg-vsc-menu py-1 text-vsc-menu-fg shadow-[0_2px_8px_var(--vsc-shadow)]";
const MENU_ITEM = "mx-1 flex h-[2em] items-center gap-2 rounded-[3px] px-2 text-[12px] whitespace-nowrap";

/** "locked": a version some projects use, so it cannot go; its small menu
 * lists them. The counterpart of the bin. A native button, drawn as a
 * toolbar button: it opens a menu (aria-haspopup, aria-expanded), which
 * Button can't say. */
function Locked({ projects, label }: { projects: string[]; label: string }) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !box.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  return (
    <div ref={box} className="relative">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={label}
        onClick={() => {
          setOpen((o) => !o);
        }}
        className="inline-flex h-[22px] items-center gap-1 rounded-[5px] px-1 text-[11px] text-vsc-desc hover:bg-vsc-toolbar-hover hover:text-vsc-fg"
      >
        <VscLock size={12} aria-hidden="true" />
        locked
      </button>
      {open && (
        <ul role="menu" aria-label={label} className={`${MENU} min-w-36`}>
          <li role="none" className="px-3 pb-0.5 text-[11px] text-vsc-desc">
            {t("Usata da")}
          </li>
          {projects.map((p) => (
            <li key={p} role="menuitem" className={MENU_ITEM}>
              {p}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** "+": the lines its source offers that are not on disk (nor being
 * downloaded), by major version; one chosen is downloaded without becoming
 * the kit's (for the projects). */
function AddLine({
  tool,
  setup,
  busy,
  onAdd,
}: {
  tool: VersionedTool;
  setup: SetupSnapshot | null;
  busy: boolean;
  onAdd: (line: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const groups = offered(tool)
    .map((g) => ({
      ...g,
      lines: g.lines.filter((l) => {
        const state = itemState(setup, tool.id, `${tool.name} ${l}`);
        return state === null || state.kind === "problem";
      }),
    }))
    .filter((g) => g.lines.length > 0);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent ? e.key === "Escape" : !box.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  if (groups.length === 0) return null;
  return (
    <div ref={box} className="relative">
      <ActionButton
        icon={VscAdd}
        label={t("Aggiungi una versione di {0}", tool.name)}
        compact
        busy={busy}
        onClick={() => {
          setOpen((o) => !o);
        }}
      />
      {open && (
        <ul role="menu" aria-label={t("Versioni da scaricare")} className={`${MENU} max-h-72 min-w-44 overflow-y-auto`}>
          {groups.map((g) => (
            <li key={g.major} role="none">
              <div className="px-3 pt-1 pb-0.5 text-[11px] text-vsc-desc">{`${g.major}.x`}</div>
              <ul role="none">
                {g.lines.map((l) => (
                  <li key={l} role="none">
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setOpen(false);
                        onAdd(l);
                      }}
                      className={`${MENU_ITEM} w-[calc(100%-8px)] text-left font-mono hover:bg-vsc-menu-selection hover:text-vsc-menu-selection-fg focus:bg-vsc-menu-selection focus:text-vsc-menu-selection-fg focus:outline-none`}
                    >
                      <VscCloudDownload size={12} aria-hidden="true" />
                      {tool.name} {l}
                      {tool.available[0] === l && (
                        <Tag tone="focus" className="ml-auto">
                          latest
                        </Tag>
                      )}
                    </button>
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** The kit's versions for the Tools page, where each versioned tool shows
 * them in its own row (ToolVersions). `enabled` false (first start) asks
 * nothing. */
export function useKitVersions(setup: SetupSnapshot | null, enabled: boolean) {
  const {
    data: tools,
    setData: setTools,
    reload: load,
  } = useLoad<VersionedTool[] | null>(versionsList, null, { onError: { fallback: [] }, enabled });
  const { busy, error, run: runAction } = useAction(load);
  // The tool of the last action, the one that shows its error.
  const [last, setLast] = useState<string | null>(null);
  const offers = useOffers(tools, load);
  useReloadOnSteps(setup, load);
  const run = useCallback(
    (id: string, key: string, task: () => Promise<void>) => {
      setLast(id);
      runAction(key, task);
    },
    [runAction],
  );
  const byId = useMemo(() => new Map((tools ?? []).map((x) => [x.id, x])), [tools]);
  return { byId, setTools, busy, error, last, run, offers };
}

export type KitVersions = ReturnType<typeof useKitVersions>;

/** The versions of an installed tool in its row of the Tools page: the
 * kit's line, the lines on disk with who uses them, one to download. */
export function ToolVersions({
  tool,
  kit,
  setup,
  outside = {},
}: {
  tool: VersionedTool;
  kit: KitVersions;
  setup: SetupSnapshot | null;
  /** Line → the components that cannot run on it (relations matrix). */
  outside?: Record<string, string[]>;
}) {
  const { busy, run, offers, setTools } = kit;
  const running = setup?.running ?? false;
  const add = (line: string) => {
    run(tool.id, `add:${tool.id}`, () => versionsAdd(tool.id, line));
  };
  return (
    <div className="mt-1.5 max-w-[620px]">
      <div className="flex items-center gap-1">
        <span className="flex-1 text-[12px] text-vsc-desc">
          {t("Versioni: il kit usa la più recente; un progetto nuovo tiene quella che c'è alla sua creazione")}
        </span>
        <div className="flex items-center gap-1">
          <RefreshLines
            tool={tool}
            checking={offers.checking.has(tool.id)}
            disabled={busy !== null}
            onRefresh={() => {
              offers.refresh([tool.id]);
            }}
          />
          {tool.downloadable && (
            <AddLine tool={tool} setup={setup} busy={running || busy === `add:${tool.id}`} onAdd={add} />
          )}
        </div>
      </div>
      <LineState
        tool={tool}
        line={tool.preferred}
        fallback={tool.default?.line ?? ""}
        setup={setup}
        error={offers.failed[tool.id]}
        onDownload={add}
        inList
      />
      <DiskLines
        tool={tool}
        setup={setup}
        outside={outside}
        busy={busy}
        running={running}
        onRemove={(line) => {
          run(tool.id, `remove:${tool.id}:${line}`, () => versionsRemove(tool.id, line).then(setTools));
        }}
      />
      {kit.last === tool.id && <ErrorLine error={kit.error} />}
    </div>
  );
}
