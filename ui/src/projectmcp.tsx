// A project's own MCP servers (DECISIONS.md D-78): in its .mcp.json for
// Claude Code, copied to .codex\config.toml for Codex; Antigravity has only
// the kit's servers.
import { useCallback, useState, type SyntheticEvent } from "react";
import { VscAdd, VscClose, VscSave, VscTrash } from "react-icons/vsc";
import { projectMcp, projectMcpSet, type McpServer, type McpView } from "./api";
import { t } from "./i18n";
import { ActionButton, ErrorLine, LoadError } from "./common";
import { useAction, useLoad } from "./hooks";
import { Select, TextArea, TextField } from "./workbench";

function describe(s: McpServer): string {
  return s.kind === "http" ? (s.url ?? "") : [s.command ?? "", ...s.args].join(" ");
}

function AddServer({ onAdd, onCancel }: { onAdd: (s: McpServer) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<McpServer["kind"]>("stdio");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [url, setUrl] = useState("");
  // Every text field is needed: Add waits for them rather than marking the
  // untouched ones `required`, which the components paint as errors at once.
  const complete = name.trim() !== "" && (kind === "stdio" ? command : url).trim() !== "";
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    if (!complete) return;
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
          <TextField mono pattern="[A-Za-z0-9_\-]+" value={name} onChange={setName} className="w-full" />
        </label>
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Tipo")}</span>
          <Select<McpServer["kind"]>
            value={kind}
            onChange={setKind}
            options={[
              { value: "stdio", label: t("programma") },
              { value: "http", label: t("indirizzo (http)") },
            ]}
            className="w-full"
          />
        </label>
      </div>
      {kind === "stdio" ? (
        <div className="grid gap-2 sm:grid-cols-2">
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Programma")}</span>
            <TextField mono placeholder="npx" value={command} onChange={setCommand} className="w-full" />
          </label>
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Argomenti, uno per riga")}</span>
            <TextArea mono rows={2} spellCheck={false} value={args} onChange={setArgs} className="w-full" />
          </label>
        </div>
      ) : (
        <label className="block space-y-1">
          <span className="text-vsc-desc">{t("Indirizzo")}</span>
          <TextField mono placeholder="https://" value={url} onChange={setUrl} className="w-full" />
        </label>
      )}
      <div className="flex gap-2">
        <ActionButton
          icon={VscSave}
          label={t("Aggiungi")}
          primary
          submit
          disabled={!complete}
          onClick={() => undefined}
        />
        <ActionButton icon={VscClose} label={t("Annulla")} onClick={onCancel} />
      </div>
    </form>
  );
}

export function ProjectMcpBox({ project }: { project: string }) {
  const read = useCallback(() => projectMcp(project), [project]);
  const { data: view, error: loadErr, reload: load } = useLoad<McpView | null>(read, null, { onError: "show" });
  const [adding, setAdding] = useState(false);
  const { busy, error, run } = useAction(load);
  const loadError = <LoadError error={loadErr} onRetry={load} className="px-3 py-2" />;
  if (!view) return loadError;
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
          {t(".mcp.json non è JSON semplice: Atollo non lo riscrive. Modificalo nell'editor.")}
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
              "Solo in questo progetto, oltre a quelli del kit. Claude Code li legge da .mcp.json (chiede conferma al primo uso), Codex da .codex\\config.toml (Atollo lo tiene allineato); Antigravity usa solo i server del kit.",
            )}
          </p>
        </>
      )}
      {loadError}
      <ErrorLine error={error} />
    </section>
  );
}
