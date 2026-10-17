// Agents page and the agent choice of a project: Claude Code, Codex,
// Antigravity, as VSCodium's Extensions view: the installed agents in the
// side bar, the chosen one's page in the editor with its state, usage and
// sign-out. Launch happens only in a project. Instructions, skills and MCP
// servers are shared by the three (data\agents, aligned by the app).
import { useState } from "react";
import { VscBook, VscFolderOpened, VscHubot, VscRefresh, VscSignOut, VscWarning } from "react-icons/vsc";
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
import type { GuideTopicId } from "../guide";
import { rich, t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { ComponentLogo } from "../logos";
import { AgentUsage, useStats } from "./stats";
import { useAction, useLoad } from "../hooks";
import {
  Button,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  InlineConfirm,
  List,
  ListItem,
  Notice,
  PaneStack,
  ProgressBar,
  SideBarTitle,
  ViewContainer,
  ViewPane,
  WelcomeView,
} from "../workbench";

/** The agents' state, refreshed every few seconds (running, signed in). */
export function useAgents(): { agents: AgentInfo[] | null; reload: () => void } {
  const { data: agents, reload } = useLoad<AgentInfo[] | null>(agentsStatus, null, {
    onError: { fallback: [] },
    every: 5000,
  });
  return { agents, reload };
}

function signInText(a: AgentInfo): string {
  if (!a.signedIn) return t("accesso non fatto: si fa al primo avvio dell'agente");
  return a.signInKind === "api key" ? t("accesso con chiave API") : t("accesso fatto");
}

const TONE: Record<Level, "ok" | "warn" | "err" | "off"> = { none: "off", ok: "ok", warning: "warn", critical: "err" };

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

export function AgentsPage({ onGuide }: { onGuide?: (topic: GuideTopicId) => void } = {}) {
  const { agents, reload } = useAgents();
  const { busy, error, run } = useAction(reload);
  const [confirm, setConfirm] = useState<AgentId | null>(null);
  const [chosen, setChosen] = useState<AgentId | null>(null);
  const [listOpen, setListOpen] = useState(true);
  const stats = useStats();

  const installed = (agents ?? []).filter((a) => a.installed);
  const agent = installed.find((a) => a.id === chosen) ?? installed[0] ?? null;
  const st = agent ? stats.report?.agents.find((x) => x.id === agent.id) : undefined;

  const side = (
    <>
      <SideBarTitle
        title={t("Agenti")}
        actions={
          <>
            <IconButton
              icon={VscRefresh}
              label={t("Aggiorna i consumi")}
              title={t("Legge di nuovo limiti e consumi degli agenti")}
              busy={stats.loading}
              disabled={installed.length === 0}
              onClick={stats.refresh}
            />
            <IconButton
              icon={VscFolderOpened}
              label={t("Apri la cartella condivisa")}
              onClick={() => {
                run("folder", () => folderOpen("agents"));
              }}
            />
            {onGuide && (
              <IconButton
                icon={VscBook}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("feat-agents");
                }}
              />
            )}
          </>
        }
      />
      <ProgressBar active={stats.loading && installed.length > 0} />
      <PaneStack>
        <ViewPane
          first
          fill
          title={t("Installati")}
          badge={installed.length > 0 ? installed.length : undefined}
          expanded={listOpen}
          onToggle={() => {
            setListOpen(!listOpen);
          }}
        >
          {agents === null ? (
            <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
          ) : installed.length === 0 ? (
            <WelcomeView>
              <p>{t("Nessun agente installato: aggiungili da Strumenti.")}</p>
            </WelcomeView>
          ) : (
            <List label={t("Agenti")}>
              {installed.map((a) => (
                <ListItem
                  key={a.id}
                  icon={<ComponentLogo id={a.id} size={32} />}
                  name={a.name}
                  meta={a.version ?? undefined}
                  description={signInText(a)}
                  footer={<AgentFooter agent={a} stats={stats.report?.agents.find((x) => x.id === a.id)} />}
                  selected={agent?.id === a.id}
                  onSelect={() => {
                    setChosen(a.id);
                    setConfirm(null);
                  }}
                />
              ))}
            </List>
          )}
        </ViewPane>
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="agents" side={side}>
      {agent ? (
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
          <EditorBody narrow>
            <ErrorLine error={stats.error ?? error} />
            <EditorSection id="agent-usage" title={t("Limiti e consumi")}>
              {st ? <AgentUsage stats={st} /> : <p className="text-vsc-desc">{t("Caricamento…")}</p>}
            </EditorSection>
            <EditorSection id="agent-account" title={t("Accesso")}>
              <p className="flex items-center gap-2">
                <Dot tone={agent.signedIn ? "ok" : "off"} />
                {signInText(agent)}
              </p>
              {agent.signedIn && agent.signInKind !== "api key" && (
                <div className="flex flex-wrap items-center gap-1.5">
                  {confirm === agent.id ? (
                    <InlineConfirm
                      row
                      question={t("Togliere l'accesso salvato nel kit?")}
                      onCancel={() => {
                        setConfirm(null);
                      }}
                      actions={
                        <>
                          <ActionButton
                            icon={VscSignOut}
                            label={t("Esci")}
                            primary
                            busy={busy === `out:${agent.id}`}
                            onClick={() => {
                              setConfirm(null);
                              run(`out:${agent.id}`, () => agentSignOut(agent.id));
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
                    <ActionButton
                      icon={VscSignOut}
                      label={t("Esci")}
                      disabled={agent.running}
                      title={agent.running ? t("Chiudi prima l'agente") : t("Toglie l'accesso salvato nel kit")}
                      onClick={() => {
                        setConfirm(agent.id);
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
            <EditorSection id="agent-shared" title={t("Condiviso fra gli agenti")}>
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
              <Button
                onClick={() => {
                  run("folder", () => folderOpen("agents"));
                }}
              >
                {t("Apri la cartella condivisa")}
              </Button>
            </EditorSection>
          </EditorBody>
        </div>
      ) : (
        <div className="p-5 text-vsc-desc">
          {agents === null ? t("Caricamento…") : t("Nessun agente installato: aggiungili da Strumenti.")}
        </div>
      )}
    </ViewContainer>
  );
}

/** The agent last used in a project, else the first installed. */
function pick(installed: AgentInfo[], chosen: AgentId | "", preferred: string | undefined): AgentId {
  return chosen || installed.find((a) => a.id === preferred)?.id || installed[0]?.id || "claude";
}

/** The choice of an agent to open in one project (the project's page). */
export function AgentPicker({
  agents,
  project,
  preferred,
}: {
  agents: AgentInfo[] | null;
  project: string;
  /** The agent last opened in the project (its dev-settings.json). */
  preferred?: string | undefined;
}) {
  const installed = (agents ?? []).filter((a) => a.installed);
  const [chosen, setChosen] = useState<AgentId | "">("");
  const { busy, error, run } = useAction();
  if (installed.length === 0) return null;
  const agent = pick(installed, chosen, preferred);
  return (
    <span className="flex items-center gap-1" title={error ?? undefined}>
      {/* As a split button: the agent's logos, then Agente. */}
      <span
        role="radiogroup"
        aria-label={t("Agente per {0}", project)}
        className="flex h-[26px] items-center rounded border border-vsc-input-border px-px"
      >
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
            // An input option (the find widget's toggles): its active colors
            // and border when chosen, the toolbar hover otherwise.
            className={`flex size-[22px] items-center justify-center rounded-[3px] ${
              agent === a.id
                ? "bg-vsc-option-active outline-1 -outline-offset-1 outline-vsc-option-active-border"
                : "opacity-60 hover:bg-vsc-toolbar-hover hover:opacity-100"
            }`}
          >
            <ComponentLogo id={a.id} size={16} />
          </button>
        ))}
      </span>
      <ActionButton
        icon={error ? VscWarning : VscHubot}
        label={t("Agente")}
        busy={busy === "launch"}
        title={error ?? t("Apre l'agente scelto nel progetto {0}", project)}
        onClick={() => {
          run("launch", () => agentLaunch(agent, project));
        }}
      />
    </span>
  );
}

/** The inline action of a project's row: opens the agent last used there. */
export function AgentButton({
  agents,
  project,
  preferred,
}: {
  agents: AgentInfo[] | null;
  project: string;
  preferred?: string | undefined;
}) {
  const installed = (agents ?? []).filter((a) => a.installed);
  const { busy, error, run } = useAction();
  if (installed.length === 0) return null;
  const agent = pick(installed, "", preferred);
  const name = installed.find((a) => a.id === agent)?.name ?? agent;
  return (
    <IconButton
      icon={error ? VscWarning : VscHubot}
      label={t("Agente")}
      busy={busy === "launch"}
      title={error ?? t("Apre {0} nel progetto {1}", name, project)}
      onClick={() => {
        run("launch", () => agentLaunch(agent, project));
      }}
    />
  );
}
