// Agents page and the agent buttons of Home: Claude Code, Codex,
// Antigravity, with their state, launch and sign-out. Instructions, skills
// and MCP servers are shared by the three (data\agents, aligned by the app).
import { useCallback, useEffect, useState } from "react";
import { VscCheck, VscFolderOpened, VscPlay, VscSignOut, VscWarning } from "react-icons/vsc";
import { agentLaunch, agentSignOut, agentsStatus, folderOpen, type AgentId, type AgentInfo } from "./api";
import { ActionButton, useAction } from "./pages";

/** The agents' state, refreshed every few seconds (running, signed in). */
export function useAgents(): { agents: AgentInfo[] | null; reload: () => void } {
  const [agents, setAgents] = useState<AgentInfo[] | null>(null);
  const reload = useCallback(() => {
    agentsStatus().then(setAgents, () => {
      setAgents([]);
    });
  }, []);
  useEffect(() => {
    reload();
    const t = window.setInterval(reload, 5000);
    return () => {
      window.clearInterval(t);
    };
  }, [reload]);
  return { agents, reload };
}

/** Buttons that open each installed agent in `projects\` (Home). */
export function AgentButtons({ agents, project = null }: { agents: AgentInfo[] | null; project?: string | null }) {
  const { busy, error, run } = useAction();
  if (!agents || agents.length === 0) return null;
  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2">
        {agents.map((a) => (
          <ActionButton
            key={a.id}
            icon={VscPlay}
            label={a.name}
            busy={busy === a.id}
            disabled={!a.installed}
            title={
              a.installed
                ? `Apre ${a.name} in una console del kit${project ? ` nel progetto ${project}` : " nella cartella dei progetti"}`
                : `${a.name} non è installato: aggiungilo da Linguaggi e strumenti`
            }
            onClick={() => {
              run(a.id, () => agentLaunch(a.id, project));
            }}
          />
        ))}
      </div>
      {error && (
        <p role="alert" className="flex items-center gap-2 text-vsc-error">
          <VscWarning size={16} aria-hidden="true" />
          <span className="select-text">{error}</span>
        </p>
      )}
    </div>
  );
}

export function AgentsPage() {
  const { agents, reload } = useAgents();
  const { busy, error, run } = useAction(reload);
  const [confirm, setConfirm] = useState<AgentId | null>(null);

  if (!agents) return <p className="text-vsc-desc">Caricamento…</p>;
  return (
    <div className="max-w-4xl space-y-5">
      <p>
        Claude Code, Codex e Antigravity condividono istruzioni globali, skill e server MCP: il kit li allinea a ogni
        avvio e quando un agente termina. Le istruzioni comuni sono in{" "}
        <span className="font-mono">data\agents\AGENTS.md</span>, le skill in{" "}
        <span className="font-mono">data\agents\skills</span>, i server MCP in{" "}
        <span className="font-mono">data\agents\mcp.json</span>.
      </p>
      <ActionButton
        icon={VscFolderOpened}
        label="Apri la cartella condivisa"
        onClick={() => {
          run("folder", () => folderOpen("agents"));
        }}
      />
      {error && (
        <p role="alert" className="flex items-center gap-2 text-vsc-error">
          <VscWarning size={16} aria-hidden="true" />
          <span className="select-text">{error}</span>
        </p>
      )}
      <ul className="space-y-3">
        {agents.map((a) => (
          <li key={a.id} className="rounded border border-vsc-border bg-vsc-sidebar px-4 py-3">
            <div className="flex flex-wrap items-center gap-3">
              <h2 className="text-[14px] font-semibold">{a.name}</h2>
              <span className="text-[12px] text-vsc-desc">
                {a.installed ? (a.version ?? "installato") : "non installato"}
              </span>
              {a.running && <span className="rounded-sm bg-vsc-badge px-1.5 text-[11px]">in esecuzione</span>}
              <span className="ml-auto flex items-center gap-1 text-[12px]">
                {a.signedIn ? (
                  <>
                    <VscCheck className="text-vsc-success" size={14} aria-hidden="true" />
                    {a.signInKind === "api key" ? "accesso con chiave API" : "accesso fatto"}
                  </>
                ) : (
                  <span className="text-vsc-desc">accesso non fatto: si fa al primo avvio dell'agente</span>
                )}
              </span>
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              <ActionButton
                icon={VscPlay}
                label="Apri"
                primary
                busy={busy === `open:${a.id}`}
                disabled={!a.installed}
                title={a.installed ? `Apre ${a.name} nella cartella dei progetti` : "Non installato"}
                onClick={() => {
                  run(`open:${a.id}`, () => agentLaunch(a.id, null));
                }}
              />
              {a.signedIn &&
                a.signInKind !== "api key" &&
                (confirm === a.id ? (
                  <>
                    <span className="self-center">Togliere l'accesso salvato nel kit?</span>
                    <ActionButton
                      icon={VscSignOut}
                      label="Esci"
                      busy={busy === `out:${a.id}`}
                      onClick={() => {
                        setConfirm(null);
                        run(`out:${a.id}`, () => agentSignOut(a.id));
                      }}
                    />
                    <ActionButton
                      icon={VscCheck}
                      label="Annulla"
                      onClick={() => {
                        setConfirm(null);
                      }}
                    />
                  </>
                ) : (
                  <ActionButton
                    icon={VscSignOut}
                    label="Esci"
                    disabled={a.running}
                    title={a.running ? "Chiudi prima l'agente" : "Toglie l'accesso salvato nel kit"}
                    onClick={() => {
                      setConfirm(a.id);
                    }}
                  />
                ))}
            </div>
          </li>
        ))}
      </ul>
      <p className="text-[12px] text-vsc-desc">
        Uscire dal kit non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni
        del tuo account.
      </p>
    </div>
  );
}

/** Compact choice of an agent to open in one project (Projects page). */
export function AgentPicker({ agents, project }: { agents: AgentInfo[] | null; project: string }) {
  const installed = (agents ?? []).filter((a) => a.installed);
  const [chosen, setChosen] = useState<AgentId | "">("");
  const { busy, error, run } = useAction();
  if (installed.length === 0) return null;
  const agent = chosen || installed[0]?.id || "claude";
  return (
    <span className="flex items-center gap-1" title={error ?? undefined}>
      <select
        aria-label={`Agente per ${project}`}
        value={agent}
        onChange={(e) => {
          setChosen(e.target.value as AgentId);
        }}
        className="rounded-sm border border-vsc-input-border bg-vsc-input px-1 py-1 text-[12px]"
      >
        {installed.map((a) => (
          <option key={a.id} value={a.id}>
            {a.name}
          </option>
        ))}
      </select>
      <ActionButton
        icon={error ? VscWarning : VscPlay}
        label="Agente"
        busy={busy === "launch"}
        title={error ?? `Apre l'agente scelto nel progetto ${project}`}
        onClick={() => {
          run("launch", () => agentLaunch(agent, project));
        }}
      />
    </span>
  );
}
