// The agents of the AI page (D-165) and the agent buttons of a project:
// Claude Code, Codex, Antigravity, Pi, as VSCodium's Extensions view: the
// installed agents in a pane of the AI side bar, the chosen one's page in
// the editor with its state, usage and its sign-in in the VM. They run
// only in the VM (D-179, LEVELS.md §7): without it every one is disabled
// with the reason, and what shows them is read again when the VM goes on
// or off (`levels-changed`). Launch happens only in a project. Instructions, skills and MCP servers are shared by all of them
// (data\agents, aligned by the app): their own row, under the agents.
import { useState } from "react";
import {
  agentLaunch,
  agentSignOut,
  folderOpen,
  onLevelsChanged,
  type AgentId,
  type AgentInfo,
  type AgentStats,
  type Level,
} from "../ipc";
import { rich, t } from "../i18n";
import { ActionButton, ErrorLine } from "../shared/common";
import { ComponentLogo, componentIcon } from "../shared/logos";
import { AgentUsage, type useStats } from "./stats";
import { GeminiKey } from "./gemini";
import { KitMcpBox } from "../projects/mcp";
import { useAction, useSubscription } from "../shared/hooks";
import { usePoll } from "../shared/poll";
import { agentsPoll } from "../shared/polls";
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

/** The agents' state, refreshed every few seconds and at once when the VM
 * goes on or off (`levels-changed`). A poll that fails keeps what the last one read: an empty list would show
 * the welcome page and hide the project's agents until the next poll. The
 * app and the AI page share one poll (shared/polls). */
export function useAgents(): { agents: AgentInfo[] | null; reload: () => void } {
  const { data: agents, reload } = usePoll<AgentInfo[], AgentInfo[] | null>(agentsPoll, null, {
    onError: "keep",
    every: 5000,
  });
  useSubscription(onLevelsChanged, reload);
  return { agents, reload };
}

type Stats = ReturnType<typeof useStats>;

/** The row of the shared instructions, skills and MCP servers. */
export const SHARED = "shared";

/** The sign-in in the VM, as far as the PC can tell (LEVELS.md §13 item
 * 12); without the VM, that it needs it. */
function signInText(a: AgentInfo): string {
  if (!a.available) return t("serve la VM");
  if (a.signInKind === "api key") return t("chiave API, tramite il proxy di Docker Sandboxes");
  if (a.signInKind === "local") return t("modelli locali del kit, senza accesso");
  return t("l'accesso si fa nella VM, al primo avvio dell'agente");
}

const TONE: Record<Level, DotTone> = { none: "off", ok: "ok", warning: "warn", critical: "err" };

/** The footer of an agent's row: its most used limit. */
function AgentFooter({ stats }: { stats: AgentStats | undefined }) {
  const peak = stats?.peak ?? null;
  if (peak === null) return null;
  return (
    <span className="flex items-center gap-2">
      {stats && (
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
                footer={<AgentFooter stats={stats.report?.agents.find((x) => x.id === a.id)} />}
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

/** An agent's page: its limits and usage, its sign-in in the VM; without
 * the VM only the notice, with the reason (LEVELS.md §7). */
export function AgentEditor({ agent, stats, onChanged }: { agent: AgentInfo; stats: Stats; onChanged: () => void }) {
  const { busy, error, run } = useAction(onChanged);
  const [confirm, setConfirm] = useState(false);
  const st = stats.report?.agents.find((x) => x.id === agent.id);
  const where = t("Gira nella VM del progetto, con la sua versione per Linux.");
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={agent.id} size={56} />}
        title={agent.name}
        meta={[
          <span key="v">{agent.version ?? t("installato")}</span>,
          <span key="in" className={agent.signedIn ? "" : "text-vsc-desc"}>
            {signInText(agent)}
          </span>,
        ]}
        description={`${where} ${t("Gli agenti si aprono sempre nella cartella di un progetto, dalla pagina Progetti.")}`}
      />
      <EditorBody>
        {!agent.available ? (
          <Notice tone="warning">{agent.unavailableReason ?? t("serve la VM")}</Notice>
        ) : (
          <>
            <ErrorLine error={stats.error ?? error} />
            <EditorSection id="agent-usage" title={t("Limiti e consumi")}>
              {st ? <AgentUsage stats={st} /> : <p className="text-vsc-desc">{t("Caricamento…")}</p>}
            </EditorSection>
            <EditorSection id="agent-account" title={t("Accesso")}>
              <p className="flex items-center gap-2">
                <Dot tone={agent.signedIn ? "ok" : "off"} />
                {signInText(agent)}
              </p>
              {agent.id === "antigravity" && <GeminiKey onDone={onChanged} />}
              {agent.kitLogin && (
                <div className="flex flex-col gap-1.5">
                  <Notice tone="info" className="text-[12px] text-vsc-desc">
                    {t(
                      "Un accesso fatto sul PC con una versione precedente di Atollo resta salvato nel kit: gli agenti nella VM non lo usano.",
                    )}
                  </Notice>
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
                        title={t("Toglie l'accesso salvato nel kit")}
                        onClick={() => {
                          setConfirm(true);
                        }}
                      />
                    )}
                  </div>
                  <Notice tone="info" className="text-[12px] text-vsc-desc">
                    {t(
                      "Uscire dal kit non revoca le sessioni sul sito del fornitore: se perdi la chiavetta, revocale dalle impostazioni del tuo account.",
                    )}
                  </Notice>
                </div>
              )}
            </EditorSection>
          </>
        )}
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

/** One button per agent the VM can run, with its logo: a click opens that
 * agent in the project's VM (D-146, D-178, D-179: Antigravity too, like
 * Pi). Without the VM each is shown disabled with the reason as its title
 * (LEVELS.md §7: not hidden, the user sees why). The project's page, its
 * row in the side bar and its card on the Home. Nothing when none is
 * there. */
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
            disabled={!a.available}
            title={
              !a.available
                ? (a.unavailableReason ?? t("serve la VM"))
                : failed
                  ? error
                  : t("Apre {0} nel progetto {1}", a.name, project)
            }
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
