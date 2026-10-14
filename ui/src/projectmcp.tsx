// A project's own MCP servers (DECISIONS.md D-78): in its .mcp.json for
// Claude Code, copied to .codex\config.toml for Codex; Antigravity has only
// the kit's servers.
import { useCallback, useEffect, useState, type SyntheticEvent } from "react";
import { VscAdd, VscClose, VscSave, VscTrash } from "react-icons/vsc";
import { projectMcp, projectMcpSet, type McpServer, type McpView } from "./api";
import { t } from "./i18n";
import { ActionButton, ErrorLine, useAction } from "./pages";

function describe(s: McpServer): string {
  return s.kind === "http" ? (s.url ?? "") : [s.command ?? "", ...s.args].join(" ");
}

function AddServer({ onAdd, onCancel }: { onAdd: (s: McpServer) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<McpServer["kind"]>("stdio");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  const field = "w-full input";
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    onAdd({
      name: name.trim(),
      kind,
      command: kind === "stdio" ? command.trim() : null,
      args:
        kind === "stdio"
          ? args
              .split(/\r?\n/)
              .map((a) => a.trim())
              .filter(Boolean)
          : [],
      url: kind === "http" ? url.trim() : null,
      env: {},
    });
  };
  return (
    <form
      onSubmit={submit}
      aria-label={t("Nuovo server MCP")}
      className="space-y-2 border-t border-vsc-border px-3 py-2"
    >
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Nome")}</span>
          <input
            className={`${field} font-mono`}
            required
            pattern="[A-Za-z0-9_\-]+"
            value={name}
            onChange={(e) => {
              setName(e.target.value);
            }}
          />
        </label>
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Tipo")}</span>
          <select
            className={field}
            value={kind}
            onChange={(e) => {
              setKind(e.target.value as McpServer["kind"]);
            }}
          >
            <option value="stdio">{t("programma")}</option>
            <option value="http">{t("indirizzo (http)")}</option>
          </select>
        </label>
      </div>
      {kind === "stdio" ? (
        <div className="grid gap-2 sm:grid-cols-2">
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Programma")}</span>
            <input
              className={`${field} font-mono`}
              required
              placeholder="npx"
              value={command}
              onChange={(e) => {
                setCommand(e.target.value);
              }}
            />
          </label>
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Argomenti, uno per riga")}</span>
            <textarea
              className={`${field} font-mono`}
              rows={2}
              spellCheck={false}
              value={args}
              onChange={(e) => {
                setArgs(e.target.value);
              }}
            />
          </label>
        </div>
      ) : (
        <label className="block space-y-1">
          <span className="text-vsc-desc">{t("Indirizzo")}</span>
          <input
            className={`${field} font-mono`}
            required
            placeholder="https://"
            value={url}
            onChange={(e) => {
              setUrl(e.target.value);
            }}
          />
        </label>
      )}
      <div className="flex gap-2">
        <ActionButton icon={VscSave} label={t("Aggiungi")} primary submit onClick={() => undefined} />
        <ActionButton icon={VscClose} label={t("Annulla")} onClick={onCancel} />
      </div>
    </form>
  );
}

export function ProjectMcpBox({ project }: { project: string }) {
  const [view, setView] = useState<McpView | null>(null);
  const [adding, setAdding] = useState(false);
  const load = useCallback(() => {
    projectMcp(project).then(setView, () => undefined);
  }, [project]);
  useEffect(load, [load]);
  const { busy, error, run } = useAction(load);
  if (!view) return null;
  const save = (servers: McpServer[]) => {
    run("save", () => projectMcpSet(project, servers));
  };
  return (
    <section aria-labelledby="project-mcp" className="border-t border-vsc-border">
      <h2
        id="project-mcp"
        className="flex h-8 items-center gap-2 px-3 text-[12px] font-semibold text-vsc-sidebar-title"
      >
        {t("Server MCP del progetto")}
      </h2>
      {view.readOnly ? (
        <p className="px-3 py-2 text-vsc-desc">
          {t(".mcp.json non è JSON semplice: NEXKORA non lo riscrive. Modificalo nell'editor.")}
        </p>
      ) : (
        <>
          <ul className="divide-y divide-vsc-border">
            {view.servers.length === 0 && (
              <li className="px-3 py-1.5 text-vsc-desc">{t("Nessuno: solo quelli del kit.")}</li>
            )}
            {view.servers.map((s) => (
              <li key={s.name} className="flex flex-wrap items-center gap-2 px-3 py-1.5">
                <span className="font-semibold">{s.name}</span>
                <span className="min-w-0 flex-1 truncate font-mono text-[12px] text-vsc-desc">{describe(s)}</span>
                <ActionButton
                  icon={VscTrash}
                  label={t("Togli")}
                  busy={busy === "save"}
                  onClick={() => {
                    save(view.servers.filter((x) => x.name !== s.name));
                  }}
                />
              </li>
            ))}
          </ul>
          {adding ? (
            <AddServer
              onAdd={(s) => {
                setAdding(false);
                save([...view.servers.filter((x) => x.name !== s.name), s]);
              }}
              onCancel={() => {
                setAdding(false);
              }}
            />
          ) : (
            <div className="px-3 py-2">
              <ActionButton
                icon={VscAdd}
                label={t("Aggiungi un server")}
                onClick={() => {
                  setAdding(true);
                }}
              />
            </div>
          )}
          <p className="px-4 pb-3 text-[12px] text-vsc-desc">
            {t(
              "Solo in questo progetto, oltre a quelli del kit. Claude Code li legge da .mcp.json (chiede conferma al primo uso), Codex da .codex\\config.toml (NEXKORA lo tiene allineato); Antigravity usa solo i server del kit.",
            )}
          </p>
        </>
      )}
      <ErrorLine error={error} />
    </section>
  );
}
