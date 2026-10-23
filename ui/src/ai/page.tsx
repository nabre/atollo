// The AI page (D-165): all of the kit's AI under one item of the activity
// bar. The side bar has two panes, the agents (with what they share) and
// the local models (the llama.cpp server and the GGUF models); the editor
// shows the row chosen in either. What is installed comes from Tools, as
// for every component; agents open in a project, from Projects.
import { useState } from "react";
import type { AgentId } from "../api";
import { AgentEditor, AgentsPane, SHARED, SharedEditor, useAgents, useStats } from "../agents";
import type { GuideTopicId } from "../guide";
import { t } from "../i18n";
import { ModelsEditor, ModelsPane, useModels } from "../models";
import { codicon, IconButton, PaneStack, ProgressBar, SideBarTitle, ViewContainer } from "../workbench";

/** The two panes of the side bar. */
export type AiPane = "agents" | "models";

/** The row shown in the editor: an agent or what the agents share; the
 * server (null) or a model. */
type Choice = { pane: "agents"; id: AgentId | typeof SHARED } | { pane: "models"; id: string | null };

export function AiPage({ focus, onGuide }: { focus?: AiPane; onGuide?: (topic: GuideTopicId) => void } = {}) {
  const { agents, reload } = useAgents();
  const stats = useStats();
  const models = useModels();
  const [chosen, setChosen] = useState<Choice | null>(focus === "models" ? { pane: "models", id: null } : null);
  const [open, setOpen] = useState<Record<AiPane, boolean>>({ agents: true, models: true });

  const installed = (agents ?? []).filter((a) => a.installed);
  // Nothing chosen yet: the first agent, or the models when there is none.
  const choice: Choice | null =
    chosen ??
    (installed[0] ? { pane: "agents", id: installed[0].id } : agents === null ? null : { pane: "models", id: null });
  const agent = choice?.pane === "agents" ? installed.find((a) => a.id === choice.id) : undefined;
  const toggle = (pane: AiPane) => () => {
    setOpen((o) => ({ ...o, [pane]: !o[pane] }));
  };

  const side = (
    <>
      <SideBarTitle
        title={t("AI")}
        actions={
          onGuide && (
            <IconButton
              icon={codicon("book")}
              label={t("Apri la guida di questa pagina")}
              onClick={() => {
                onGuide(choice?.pane === "models" ? "models" : "agents");
              }}
            />
          )
        }
      />
      <ProgressBar active={stats.loading && installed.length > 0} />
      <PaneStack>
        <AgentsPane
          agents={agents}
          stats={stats}
          selected={choice?.pane === "agents" ? choice.id : null}
          onSelect={(id) => {
            setChosen({ pane: "agents", id });
          }}
          expanded={open.agents}
          onToggle={toggle("agents")}
          fit={open.models}
        />
        <ModelsPane
          models={models}
          selected={choice?.pane === "models" ? choice.id : undefined}
          onSelect={(id) => {
            setChosen({ pane: "models", id });
          }}
          expanded={open.models}
          onToggle={toggle("models")}
        />
      </PaneStack>
    </>
  );

  return (
    <ViewContainer id="ai" side={side}>
      {choice === null ? (
        <div className="p-5 text-vsc-desc">{t("Caricamento…")}</div>
      ) : choice.pane === "models" ? (
        <ModelsEditor models={models} />
      ) : choice.id === SHARED ? (
        <SharedEditor />
      ) : agent ? (
        <AgentEditor key={agent.id} agent={agent} stats={stats} onChanged={reload} />
      ) : (
        <SharedEditor />
      )}
    </ViewContainer>
  );
}
