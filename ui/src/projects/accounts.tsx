// «Account degli agenti» (D-185): the account each agent uses in the
// project's sandbox. An account is a sign-in kept in the kit
// (data/accounts/<agent>/<name>): made inside the sandbox at the agent's
// first session, shared by every project that picks it, kept by «Ricrea».
import { useCallback, useState } from "react";
import {
  agentAccountCreate,
  agentAccounts,
  agentAccountSignOut,
  projectAgentAccounts,
  projectAgentAccountSet,
  DEFAULT_ACCOUNT,
  type AgentAccounts,
  type AgentId,
} from "../ipc/agents";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { codicon, EditorSection, Select, TextField } from "../workbench";

const NAMES: Record<AgentId, string> = {
  claude: "Claude Code",
  codex: "Codex",
  pi: "Pi",
  antigravity: "Antigravity",
};

interface View {
  all: AgentAccounts[];
  chosen: Partial<Record<AgentId, string>>;
}

export function AgentAccountsSection({ project }: { project: string }) {
  const load = useCallback(
    () => Promise.all([agentAccounts(), projectAgentAccounts(project)]).then(([all, chosen]) => ({ all, chosen })),
    [project],
  );
  const { data, error, reload } = useLoad<View | null>(load, null, { onError: "show" });
  const { busy, error: actError, run } = useAction(reload);
  const [adding, setAdding] = useState<AgentId | null>(null);
  const [name, setName] = useState("");
  if (!data) return error ? <ErrorLine error={error} /> : <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  const create = (agent: AgentId) => {
    const wanted = name.trim();
    run(`new:${agent}`, () =>
      agentAccountCreate(agent, wanted).then(() => projectAgentAccountSet(project, agent, wanted)),
    );
    setAdding(null);
    setName("");
  };
  return (
    <EditorSection id="agent-accounts" title={t("Account degli agenti")}>
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Ogni agente usa nella sandbox del progetto l'account scelto qui. L'accesso si fa nella sandbox alla prima sessione e resta nel kit: vale per tutti i progetti che scelgono lo stesso account.",
        )}
      </p>
      <ul className="space-y-vsc-size80">
        {data.all.map((a) => {
          const current = data.chosen[a.agent] ?? DEFAULT_ACCOUNT;
          const account = a.accounts.find((x) => x.name === current);
          return (
            <li key={a.agent} className="flex flex-wrap items-center gap-vsc-size80" data-agent={a.agent}>
              <span className="w-28">{NAMES[a.agent]}</span>
              {a.supported ? (
                <>
                  <Select
                    label={t("Account di {0}", NAMES[a.agent])}
                    value={current}
                    options={a.accounts.map((x) => ({ value: x.name, label: x.name }))}
                    disabled={busy !== null}
                    onChange={(v) => {
                      run(`set:${a.agent}`, () => projectAgentAccountSet(project, a.agent, v));
                    }}
                  />
                  <span className="text-[12px] text-vsc-desc">
                    {account?.signedIn ? t("connesso") : t("non ancora connesso")}
                  </span>
                  {account?.signedIn && (
                    <ActionButton
                      icon={codicon("sign-out")}
                      label={t("Esci")}
                      small
                      busy={busy === `out:${a.agent}`}
                      onClick={() => {
                        run(`out:${a.agent}`, () => agentAccountSignOut(a.agent, current));
                      }}
                    />
                  )}
                  {adding === a.agent ? (
                    <>
                      <TextField
                        label={t("Nome del nuovo account")}
                        value={name}
                        onChange={setName}
                        autoFocus
                        placeholder={t("lavoro")}
                      />
                      <ActionButton
                        icon={codicon("check")}
                        label={t("Crea")}
                        small
                        disabled={name.trim() === ""}
                        onClick={() => {
                          create(a.agent);
                        }}
                      />
                      <ActionButton
                        icon={codicon("close")}
                        label={t("Annulla")}
                        small
                        onClick={() => {
                          setAdding(null);
                          setName("");
                        }}
                      />
                    </>
                  ) : (
                    <ActionButton
                      icon={codicon("add")}
                      label={t("Nuovo account…")}
                      small
                      onClick={() => {
                        setAdding(a.agent);
                      }}
                    />
                  )}
                </>
              ) : (
                <span className="text-[12px] text-vsc-desc">
                  {t("Nessun account separato: usa l'accesso fatto nella sandbox del progetto.")}
                </span>
              )}
            </li>
          );
        })}
      </ul>
      <p className="text-[12px] text-vsc-desc">
        {t(
          "Un cambio vale dalla prossima sessione dell'agente. L'account usato prima resta montato nella sandbox fino a «Ricrea».",
        )}
      </p>
      <ErrorLine error={actError} />
    </EditorSection>
  );
}
