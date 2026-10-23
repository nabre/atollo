// MCP servers: a project's own (DECISIONS.md D-78), in its .mcp.json for
// Claude Code, copied to .codex\config.toml for Codex; and the kit's, in
// data\agents\mcp.json, that the alignment carries to every agent. Both
// lists offer the ready servers of their kind (N-7, D-156).
import { useCallback, useState, type SyntheticEvent } from "react";
import {
  kitMcp,
  kitMcpSet,
  mcpPresets,
  projectMcp,
  projectMcpSet,
  type McpPreset,
  type McpServer,
  type McpView,
} from "../api";
import { t } from "../i18n";
import { ActionButton, ErrorLine, LoadError } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { codicon, Select, TextArea, TextField } from "../workbench";

function describe(s: McpServer): string {
  return s.kind === "http" ? (s.url ?? "") : [s.command ?? "", ...s.args].join(" ");
}

/** What a ready server does, in the user's language. */
function presetText(id: McpPreset["id"]): string {
  switch (id) {
    case "github":
      return t("Issue, pull request e repository su GitHub, con l'account GitHub del kit.");
    case "playwright":
      return t("Un browser che l'agente apre e usa: pagine, clic, moduli, schermate.");
    case "fetch":
      return t("Legge una pagina web e la dà all'agente come testo.");
    case "filesystem":
      return t("Legge e scrive i file, solo nella cartella del progetto.");
    case "mysql":
      return t("Interroga il database MySQL del progetto (DBHub).");
    case "mongodb":
      return t("Interroga il database MongoDB del progetto.");
  }
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
      headers: {},
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
          icon={codicon("save")}
          label={t("Aggiungi")}
          primary
          submit
          disabled={!complete}
          onClick={() => undefined}
        />
        <ActionButton icon={codicon("close")} label={t("Annulla")} onClick={onCancel} />
      </div>
    </form>
  );
}

function McpBox({
  id,
  title,
  read,
  write,
  presets,
  empty,
  readOnly,
  note,
}: {
  id: string;
  title: string;
  read: () => Promise<McpView>;
  write: (servers: McpServer[]) => Promise<void>;
  /** The ready servers this list may take. */
  presets: McpPreset[];
  empty: string;
  readOnly: string;
  note: string;
}) {
  const { data: view, error: loadErr, reload: load, retry } = useLoad<McpView | null>(read, null, { onError: "show" });
  const [adding, setAdding] = useState(false);
  const { busy, error, run } = useAction(load);
  const loadError = <LoadError error={loadErr} onRetry={retry} className="px-3 py-2" />;
  if (!view) return loadError;
  const save = (key: string, servers: McpServer[]) => {
    run(key, () => write(servers));
  };
  // A ready server already in the list (by name) is not offered again.
  const ready = presets.filter((p) => !view.servers.some((s) => s.name === p.server.name));
  return (
    <section aria-labelledby={id} className="border-t border-vsc-border">
      <h2 id={id} className="flex h-8 items-center gap-2 px-3 text-[12px] font-semibold text-vsc-sidebar-title">
        {title}
      </h2>
      {view.readOnly ? (
        <p className="px-3 py-2 text-vsc-desc">{readOnly}</p>
      ) : (
        <>
          <ul className="divide-y divide-vsc-border">
            {view.servers.length === 0 && <li className="px-3 py-1.5 text-vsc-desc">{empty}</li>}
            {view.servers.map((s) => (
              <li key={s.name} className="flex flex-wrap items-center gap-2 px-3 py-1.5">
                <span className="font-semibold">{s.name}</span>
                <span className="min-w-0 flex-1 truncate font-mono text-[12px] text-vsc-desc">{describe(s)}</span>
                <ActionButton
                  icon={codicon("trash")}
                  label={t("Togli")}
                  busy={busy === `remove-${s.name}`}
                  disabled={busy !== null}
                  onClick={() => {
                    save(
                      `remove-${s.name}`,
                      view.servers.filter((x) => x.name !== s.name),
                    );
                  }}
                />
              </li>
            ))}
          </ul>
          {ready.length > 0 && (
            <div className="border-t border-vsc-border">
              <h3 className="px-3 pt-2 text-[12px] font-semibold text-vsc-desc">{t("Server pronti")}</h3>
              <ul className="divide-y divide-vsc-border">
                {ready.map((p) => (
                  <li key={p.id} className="flex flex-wrap items-center gap-2 px-3 py-1.5">
                    <span className="font-semibold">{p.server.name}</span>
                    <span className="min-w-0 flex-1 text-[12px] text-vsc-desc">{presetText(p.id)}</span>
                    <ActionButton
                      icon={codicon("add")}
                      label={t("Aggiungi")}
                      small
                      busy={busy === `add-${p.id}`}
                      disabled={busy !== null}
                      onClick={() => {
                        save(`add-${p.id}`, [...view.servers, p.server]);
                      }}
                    />
                  </li>
                ))}
              </ul>
            </div>
          )}
          {adding ? (
            <AddServer
              onAdd={(s) => {
                setAdding(false);
                save("add", [...view.servers.filter((x) => x.name !== s.name), s]);
              }}
              onCancel={() => {
                setAdding(false);
              }}
            />
          ) : (
            <div className="px-3 py-2">
              <ActionButton
                icon={codicon("add")}
                label={t("Aggiungi un server")}
                onClick={() => {
                  setAdding(true);
                }}
              />
            </div>
          )}
          <p className="px-4 pb-3 text-[12px] text-vsc-desc">{note}</p>
        </>
      )}
      {loadError}
      <ErrorLine error={error} />
    </section>
  );
}

/** The ready servers of `scope`; none while they load or if they fail
 * (the list itself still works). */
function usePresets(scope: McpPreset["scope"]): McpPreset[] {
  const { data } = useLoad<McpPreset[] | null>(mcpPresets, null, { onError: "keep" });
  return (data ?? []).filter((p) => p.scope === scope);
}

/** `engines`: the databases created for the project, whose ready servers
 * it is offered. */
export function ProjectMcpBox({ project, engines }: { project: string; engines: string[] }) {
  const read = useCallback(() => projectMcp(project), [project]);
  const write = useCallback((servers: McpServer[]) => projectMcpSet(project, servers), [project]);
  const presets = usePresets("project").filter((p) => p.engine === null || engines.includes(p.engine));
  return (
    <McpBox
      id="project-mcp"
      title={t("Server MCP del progetto")}
      read={read}
      write={write}
      presets={presets}
      empty={t("Nessuno: solo quelli del kit.")}
      readOnly={t(".mcp.json non è JSON semplice: Atollo non lo riscrive. Modificalo nell'editor.")}
      note={t(
        "Solo in questo progetto, oltre a quelli del kit. Claude Code li legge da .mcp.json (chiede conferma al primo uso), Codex da .codex\\config.toml (Atollo lo tiene allineato); Antigravity e Pi usano solo i server del kit.",
      )}
    />
  );
}

/** The kit's servers, on the shared row of the AI page. */
export function KitMcpBox() {
  const presets = usePresets("kit");
  return (
    <McpBox
      id="kit-mcp"
      title={t("Server MCP del kit")}
      read={kitMcp}
      write={kitMcpSet}
      presets={presets}
      empty={t("Nessuno.")}
      readOnly={t("data\\agents\\mcp.json non è JSON valido: Atollo non lo riscrive. Correggilo nell'editor.")}
      note={t(
        "Per tutti gli agenti in tutti i progetti: Atollo li porta a Claude Code, Codex, Antigravity e Pi. I programmi che servono (Node per npx, uv per uvx) si installano al primo avvio dell'agente; il token di GitHub e i database del progetto arrivano all'agente quando parte, mai scritti nei file.",
      )}
    />
  );
}
