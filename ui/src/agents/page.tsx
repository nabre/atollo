// The agents of the AI page (D-165) and the agent buttons of a project:
// Claude Code, Codex, Antigravity, Pi, as VSCodium's Extensions view: the
// installed agents in a pane of the AI side bar, the chosen one's page in
// the editor with its state, usage and sign-in. Launch happens only in a
// project. Instructions, skills and MCP servers are shared by all of them
// (data\agents, aligned by the app): their own row, under the agents.
import { useState } from "react";
import {
  agentLaunch,
  agentSignOut,
  agentsStatus,
  folderOpen,
  type AgentId,
  type AgentInfo,
  type AgentStats,
  type Level,
} from "../api";
import { rich, t } from "../i18n";
import { ActionButton, ErrorLine } from "../shared/common";
import { ComponentLogo, componentIcon } from "../shared/logos";
import { AgentUsage, type useStats } from "./stats";
import { TokenLink } from "./token";
import { GeminiKey } from "./gemini";
import { KitMcpBox } from "../projects/mcp";
import { useAction, useLoad } from "../shared/hooks";
import {
  Button,
  Codicon,
  codicon,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  InlineConfirm,
  List,
  ListItem,
  Notice,
  ViewPane,
  WelcomeView,
  type DotTone,
} from "../workbench";

/** The agents' state, refreshed every few seconds (running, signed in). A
 * poll that fails keeps what the last one read: an empty list would show
 * the welcome page and hide the project's agents until the next poll. */
export function useAgents(): { agents: AgentInfo[] | null; reload: () => void } {
  const { data: agents, reload } = useLoad<AgentInfo[] | null>(agentsStatus, null, {
    onError: "keep",
    every: 5000,
  });
  return { agents, reload };
}

type Stats = ReturnType<typeof useStats>;

/** The row of the shared instructions, skills and MCP servers. */
export const SHARED = "shared";

function signInText(a: AgentInfo): string {
  if (!a.signedIn) return t("accesso non fatto: si fa al primo avvio dell'agente");
  if (a.signInKind === "api key") return t("accesso con chiave API");
  if (a.signInKind === "local") return t("modelli locali del kit, senza accesso");
  return a.signInKind === "token" ? t("accesso con un token") : t("accesso fatto");
}

const TONE: Record<Level, DotTone> = { none: "off", ok: "ok", warning: "warn", critical: "err" };

/** The footer of an agent's row: running, and its most used limit. */
function AgentFooter({ agent, stats }: { agent: AgentInfo; stats: AgentStats | undefined }) {
  const peak = stats?.peak ?? null;
  if (!agent.running && peak === null) return null;
  return (
    <span className="flex items-center gap-2">
      {agent.running && (
        <span className="flex items-center gap-1">
          <Dot tone="ok" />
          {t("in esecuzione")}
        </span>
      )}
      {peak !== null && stats && (
        <span className="flex items-center gap-1 tabular-nums">
          <Dot tone={TONE[stats.level]} />
          {stats.peakLabel ? `${stats.peakLabel} ${String(peak)}%` : `${String(peak)}%`}
        </span>
      )}
    </span>
  );
}

/** The pane of the agents: the installed ones, then what they share. */
export function AgentsPane({
  agents,
  stats,
  selected,
  onSelect,
  expanded,
  onToggle,
  fit = false,
}: {
  agents: AgentInfo[] | null;
  stats: Stats;
  /** An agent, `SHARED`, or null: none of this pane's rows. */
  selected: string | null;
  onSelect: (id: AgentId | typeof SHARED) => void;
  expanded: boolean;
  onToggle: () => void;
  /** As tall as its rows, the rest to the panes under it. */
  fit?: boolean;
}) {
  const { error, run } = useAction();
  const installed = (agents ?? []).filter((a) => a.installed);
  return (
    <ViewPane
      first
      fill
      fit={fit}
      title={t("Agenti")}
      badge={installed.length > 0 ? installed.length : undefined}
      expanded={expanded}
      onToggle={onToggle}
      actions={
        <>
          <IconButton
            icon={codicon("refresh")}
            label={t("Aggiorna i consumi")}
            title={t("Legge di nuovo limiti e consumi degli agenti")}
            busy={stats.loading}
            disabled={installed.length === 0}
            onClick={stats.refresh}
          />
          <IconButton
            icon={codicon("folder-opened")}
            label={t("Apri la cartella condivisa")}
            onClick={() => {
              run("folder", () => folderOpen("agents"));
            }}
          />
        </>
      }
    >
      <ErrorLine error={error} />
      {agents === null ? (
        <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
      ) : (
        <>
          {installed.length === 0 && (
            <WelcomeView>
              <p>{t("Nessun agente installato: aggiungili da Strumenti.")}</p>
            </WelcomeView>
          )}
          <List label={t("Agenti")}>
            {installed.map((a) => (
              <ListItem
                key={a.id}
                icon={<ComponentLogo id={a.id} size={32} />}
                name={a.name}
                meta={a.version ?? undefined}
                description={signInText(a)}
                footer={<AgentFooter agent={a} stats={stats.report?.agents.find((x) => x.id === a.id)} />}
                selected={selected === a.id}
                onSelect={() => {
                  onSelect(a.id);
                }}
              />
            ))}
            <ListItem
              icon={<Codicon name="references" size={24} />}
              name={t("Condiviso fra gli agenti")}
              description={t("Istruzioni, skill e server MCP")}
              selected={selected === SHARED}
              onSelect={() => {
                onSelect(SHARED);
              }}
            />
          </List>
        </>
      )}
    </ViewPane>
  );
}

/** An agent's page: its limits and usage, its sign-in. */
export function AgentEditor({ agent, stats, onChanged }: { agent: AgentInfo; stats: Stats; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  const [confirm, setConfirm] = useState(false);
  const st = stats.report?.agents.find((x) => x.id === agent.id);
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={agent.id} size={56} />}
        title={agent.name}
        meta={[
          <span key="v">{agent.version ?? t("installato")}</span>,
          agent.running ? (
            <span key="run" className="flex items-center gap-1">
              <Dot tone="ok" />
              {t("in esecuzione")}
            </span>
          ) : null,
          <span key="in" className={agent.signedIn ? "" : "text-vsc-desc"}>
            {signInText(agent)}
          </span>,
        ]}
        description={t("Gli agenti si aprono sempre nella cartella di un progetto, dalla pagina Progetti.")}
      />
      <EditorBody>
        <ErrorLine error={stats.error ?? error} />
        <EditorSection id="agent-usage" title={t("Limiti e consumi")}>
          {st ? <AgentUsage stats={st} /> : <p className="text-vsc-desc">{t("Caricamento…")}</p>}
        </EditorSection>
        <EditorSection id="agent-account" title={t("Accesso")}>
          <p className="flex items-center gap-2">
            <Dot tone={agent.signedIn ? "ok" : "off"} />
            {signInText(agent)}
          </p>
          <TokenLink agent={agent} onDone={onChanged} />
          {agent.id === "antigravity" && <GeminiKey onDone={onChanged} />}
          {agent.signedIn && agent.signInKind !== "api key" && agent.signInKind !== "local" && (
            <div className="flex flex-wrap items-center gap-1.5">
              {confirm ? (
                <InlineConfirm
                  row
                  question={t("Togliere l'accesso salvato nel kit?")}
                  onCancel={() => {
                    setConfirm(false);
                  }}
                  actions={
                    <>
                      <ActionButton
                        icon={codicon("sign-out")}
                        label={t("Esci")}
                        primary
                        busy={busy === `out:${agent.id}`}
                        onClick={() => {
                          setConfirm(false);
                          run(`out:${agent.id}`, () => agentSignOut(agent.id));
                        }}
                      />
                      <Button
                        onClick={() => {
                          setConfirm(false);
                        }}
                      >
                        {t("Annulla")}
                      </Button>
                    </>
                  }
                />
              ) : (
                <ActionButton
                  icon={codicon("sign-out")}
                  label={t("Esci")}
                  disabled={agent.running}
                  title={agent.running ? t("Chiudi prima l'agente") : t("Toglie l'accesso salvato nel kit")}
                  onClick={() => {
                    setConfirm(true);
                  }}
                />
              )}
              {agent.running && <span className="text-[12px] text-vsc-desc">{t("Chiudi prima l'agente")}</span>}
            </div>
          )}
          <Notice tone="info" className="text-[12px] text-vsc-desc">
            {t(
              "Uscire dal kit non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account.",
            )}
          </Notice>
        </EditorSection>
      </EditorBody>
    </div>
  );
}

/** What the agents share: the instructions, the skills, the kit's MCP
 * servers. */
export function SharedEditor() {
  const { error, run } = useAction();
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<Codicon name="references" size={48} />}
        title={t("Condiviso fra gli agenti")}
        description={t("Istruzioni, skill e server MCP valgono per tutti gli agenti del kit.")}
      />
      <EditorBody>
        <ErrorLine error={error} />
        <EditorSection id="agent-shared" title={t("Istruzioni e skill")}>
          <p>
            {rich(
              t(
                "Claude Code, Codex, Antigravity e Pi condividono istruzioni globali, skill e server MCP: il kit li allinea a ogni avvio e quando un agente termina. Le istruzioni comuni sono in {0}, le skill in {1}, i server MCP in {2}.",
              ),
              <span className="font-mono">data\agents\AGENTS.md</span>,
              <span className="font-mono">data\agents\skills</span>,
              <span className="font-mono">data\agents\mcp.json</span>,
            )}
          </p>
          <Button
            onClick={() => {
              run("folder", () => folderOpen("agents"));
            }}
          >
            {t("Apri la cartella condivisa")}
          </Button>
        </EditorSection>
        <KitMcpBox />
      </EditorBody>
    </div>
  );
}

/** One button per installed agent, with its logo: a click opens that agent
 * in the project (D-146). The project's page, its row in the side bar and
 * its card on the Home. Nothing when no agent is installed. */
export function AgentButtons({ agents, project }: { agents: AgentInfo[] | null; project: string }) {
  const installed = (agents ?? []).filter((a) => a.installed);
  const { busy, error, run } = useAction();
  // The agent of the last launch: its button shows the error, if any.
  const [last, setLast] = useState<AgentId | null>(null);
  if (installed.length === 0) return null;
  return (
    <>
      {installed.map((a) => {
        const failed = error !== null && last === a.id;
        return (
          <IconButton
            key={a.id}
            icon={failed ? codicon("warning") : componentIcon(a.id)}
            label={t("Apri {0}", a.name)}
            busy={busy === `agent:${a.id}`}
            title={failed ? error : t("Apre {0} nel progetto {1}", a.name, project)}
            onClick={() => {
              setLast(a.id);
              run(`agent:${a.id}`, () => agentLaunch(a.id, project));
            }}
          />
        );
      })}
    </>
  );
}
