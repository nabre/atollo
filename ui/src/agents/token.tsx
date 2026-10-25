// "Collega con un token": Claude Code's sign-in on macOS kept in the kit
// (D-115). On macOS Claude Code keeps its login in the
// Keychain, which stays on that Mac: `claude setup-token` runs in the app's
// terminal and the token it prints is saved in data/agents/claude/oauth-token
// (Rust: tauri/src/agents/claude_token.rs). If it is not recognized, it can
// be pasted here. "Esci" on the page deletes it.
import { useState } from "react";
import { act } from "../ipc/core";
import type { AgentInfo } from "../ipc";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction } from "../shared/hooks";
import { Button, codicon, Notice, TextField } from "../workbench";

/** Offered for Claude Code on macOS (Rust `AgentInfo::token_link`). */
export function tokenOffered(agent: AgentInfo): boolean {
  return agent.tokenLink;
}

/** Runs `claude setup-token` in a session of the app's terminal. */
export function agentTokenLink(): Promise<void> {
  return act("agent_token_link");
}

/** Saves a token pasted by hand (`sk-ant-oat…`). */
export function agentTokenSave(token: string): Promise<void> {
  return act("agent_token_save", { token });
}

const PREFIX = "sk-ant-oat";

/** The token link of an agent not signed in (only where it is offered). */
export function TokenLink({ agent, onDone }: { agent: AgentInfo; onDone: () => void }) {
  const { busy, error, run } = useAction(onDone);
  const [token, setToken] = useState("");
  if (!tokenOffered(agent) || agent.signedIn) return null;
  const pasted = token.trim();
  return (
    <div className="flex flex-col gap-2">
      <Notice tone="info" className="text-[12px] text-vsc-desc">
        {t(
          "Su macOS Claude Code salva l'accesso nel Portachiavi, che resta su questo Mac. Con un token (valido un anno) l'accesso resta nel kit e funziona su ogni Mac.",
        )}
      </Notice>
      <div className="flex flex-wrap items-center gap-1.5">
        <ActionButton
          icon={codicon("key")}
          label={t("Collega con un token")}
          primary
          busy={busy === "link"}
          disabled={agent.running}
          title={t("Apre claude setup-token nel terminale dell'app: completa l'accesso nel browser")}
          onClick={() => {
            run("link", agentTokenLink);
          }}
        />
      </div>
      <p className="text-[12px] text-vsc-desc">
        {t("Al termine il token viene salvato nel kit. Se non viene riconosciuto, incollalo qui:")}
      </p>
      <div className="flex flex-wrap items-center gap-1.5">
        <TextField
          value={token}
          onChange={setToken}
          label={t("Token di Claude Code")}
          type="password"
          autoComplete="off"
          mono
          placeholder={`${PREFIX}01-…`}
          className="min-w-0 flex-1"
        />
        <Button
          busy={busy === "save"}
          disabled={!pasted.startsWith(PREFIX)}
          onClick={() => {
            run("save", async () => {
              await agentTokenSave(pasted);
              setToken("");
            });
          }}
        >
          {t("Salva il token")}
        </Button>
      </div>
      <ErrorLine error={error} />
    </div>
  );
}
