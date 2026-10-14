// Agents page and the agent choice of a project: Claude Code, Codex,
// Antigravity, with their state, usage, launch (only in a project) and
// sign-out. Instructions, skills
// and MCP servers are shared by the three (data\agents, aligned by the app).
import { useCallback, useEffect, useState } from "react";
import { VscCheck, VscFolderOpened, VscHubot, VscPlay, VscSignOut, VscWarning } from "react-icons/vsc";
import { agentLaunch, agentSignOut, agentsStatus, folderOpen, type AgentId, type AgentInfo } from "./api";
import { rich, t } from "./i18n";
import { ActionButton, useAction } from "./pages";
import { ComponentLogo } from "./logos";
import { Badge } from "./tags";
import { AgentUsage, StatsRefresh, useStats } from "./stats";

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
    const timer = window.setInterval(reload, 5000);
    return () => {
      window.clearInterval(timer);
    };
  }, [reload]);
  return { agents, reload };
}

export function AgentsPage() {
  const { agents, reload } = useAgents();
  const { busy, error, run } = useAction(reload);
  const [confirm, setConfirm] = useState<AgentId | null>(null);
  const stats = useStats();

  if (!agents) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  return (
    <div className="max-w-4xl space-y-3">
      <p>
        {rich(
          t(
            "Claude Code, Codex e Antigravity condividono istruzioni globali, skill e server MCP: il kit li allinea a ogni avvio e quando un agente termina. Le istruzioni comuni sono in {0}, le skill in {1}, i server MCP in {2}.",
          ),
          <span className="font-mono">data\agents\AGENTS.md</span>,
          <span className="font-mono">data\agents\skills</span>,
          <span className="font-mono">data\agents\mcp.json</span>,
        )}
      </p>
      <div className="flex flex-wrap gap-2">
        <ActionButton
          icon={VscFolderOpened}
          label={t("Apri la cartella condivisa")}
          onClick={() => {
            run("folder", () => folderOpen("agents"));
          }}
        />
        {agents.some((a) => a.installed) && <StatsRefresh loading={stats.loading} onRefresh={stats.refresh} />}
      </div>
      {stats.error && (
        <p role="alert" className="flex items-center gap-2 text-vsc-error">
          <VscWarning size={16} aria-hidden="true" />
          <span className="select-text">{stats.error}</span>
        </p>
      )}
      {error && (
        <p role="alert" className="flex items-center gap-2 text-vsc-error">
          <VscWarning size={16} aria-hidden="true" />
          <span className="select-text">{error}</span>
        </p>
      )}
      {agents.every((a) => !a.installed) && (
        <p className="text-vsc-desc">{t("Nessun agente installato: aggiungili da Strumenti.")}</p>
      )}
      <p className="text-vsc-desc">
        {t("Gli agenti si aprono sempre nella cartella di un progetto, dalla pagina Progetti.")}
      </p>
      <ul className="space-y-2">
        {agents
          .filter((a) => a.installed)
          .map((a) => (
            <li key={a.id} className="border-t border-vsc-border px-3 py-2">
              <div className="flex flex-wrap items-center gap-2">
                <h2 className="flex items-center gap-2 text-[14px] font-semibold">
                  <ComponentLogo id={a.id} size={18} />
                  {a.name}
                </h2>
                <span className="text-[12px] text-vsc-desc">
                  {a.installed ? (a.version ?? t("installato")) : t("non installato")}
                </span>
                {a.running && <Badge>{t("in esecuzione")}</Badge>}
                <span className="ml-auto flex items-center gap-1 text-[12px]">
                  {a.signedIn ? (
                    <>
                      <VscCheck className="text-vsc-success" size={14} aria-hidden="true" />
                      {a.signInKind === "api key" ? t("accesso con chiave API") : t("accesso fatto")}
                    </>
                  ) : (
                    <span className="text-vsc-desc">{t("accesso non fatto: si fa al primo avvio dell'agente")}</span>
                  )}
                </span>
              </div>
              <div className="mt-1.5 flex flex-wrap gap-2">
                {a.signedIn &&
                  a.signInKind !== "api key" &&
                  (confirm === a.id ? (
                    <>
                      <span className="self-center">{t("Togliere l'accesso salvato nel kit?")}</span>
                      <ActionButton
                        icon={VscSignOut}
                        label={t("Esci")}
                        busy={busy === `out:${a.id}`}
                        onClick={() => {
                          setConfirm(null);
                          run(`out:${a.id}`, () => agentSignOut(a.id));
                        }}
                      />
                      <ActionButton
                        icon={VscCheck}
                        label={t("Annulla")}
                        onClick={() => {
                          setConfirm(null);
                        }}
                      />
                    </>
                  ) : (
                    <ActionButton
                      icon={VscSignOut}
                      label={t("Esci")}
                      disabled={a.running}
                      title={a.running ? t("Chiudi prima l'agente") : t("Toglie l'accesso salvato nel kit")}
                      onClick={() => {
                        setConfirm(a.id);
                      }}
                    />
                  ))}
              </div>
              {(() => {
                const st = stats.report?.agents.find((x) => x.id === a.id);
                return st ? <AgentUsage stats={st} /> : null;
              })()}
            </li>
          ))}
      </ul>
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Uscire dal kit non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account.",
        )}
      </p>
    </div>
  );
}

/** Compact choice of an agent to open in one project (Projects page). */
export function AgentPicker({
  agents,
  project,
  preferred,
  compact = false,
}: {
  agents: AgentInfo[] | null;
  project: string;
  /** Only the icon of the Agents page on the button (list of projects). */
  compact?: boolean;
  /** The agent last opened in the project (its dev-settings.json). */
  preferred?: string | undefined;
}) {
  const installed = (agents ?? []).filter((a) => a.installed);
  const [chosen, setChosen] = useState<AgentId | "">("");
  const { busy, error, run } = useAction();
  if (installed.length === 0) return null;
  const agent = chosen || installed.find((a) => a.id === preferred)?.id || installed[0]?.id || "claude";
  return (
    <span className="flex items-center gap-1" title={error ?? undefined}>
      <span role="radiogroup" aria-label={t("Agente per {0}", project)} className="flex items-center">
        {installed.map((a) => (
          <button
            key={a.id}
            type="button"
            role="radio"
            aria-checked={agent === a.id}
            aria-label={a.name}
            title={a.name}
            onClick={() => {
              setChosen(a.id);
            }}
            className={`rounded-sm p-1 ${
              agent === a.id ? "bg-vsc-list-active outline-1 outline-vsc-list-focus" : "opacity-60 hover:opacity-100"
            }`}
          >
            <ComponentLogo id={a.id} size={16} />
          </button>
        ))}
      </span>
      <ActionButton
        icon={error ? VscWarning : compact ? VscHubot : VscPlay}
        label={t("Agente")}
        compact={compact}
        busy={busy === "launch"}
        title={error ?? t("Apre l'agente scelto nel progetto {0}", project)}
        onClick={() => {
          run("launch", () => agentLaunch(agent, project));
        }}
      />
    </span>
  );
}
