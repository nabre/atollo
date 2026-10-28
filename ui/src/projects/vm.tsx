// «VM del progetto» (D-179, docs/app/LEVELS.md §8.1, §8.2, §8.5): each
// sandbox of the project (its own, then each agent's) with its state, the
// folders it mounts (rw/ro), the PC's ports it reaches, the tools found by
// «Verifica strumenti», and its four actions: «Riavvia», «Ricrea» (asked
// here first: what is lost, what stays), «Aggiorna strumenti»,
// «Verifica strumenti». The long work is a job (§8.6): the preparation in
// the page's «Preparazione della VM…» and the status bar.
import { useCallback, useState } from "react";
import {
  projectVmAction,
  projectVmCheck,
  projectVmReport,
  type SandboxReport,
  type ToolCheck,
  type VmAction,
  type VmReport,
} from "../ipc/levels";
import type { AgentId } from "../ipc/agents";
import { useAction, useLoad } from "../shared/hooks";
import { ActionButton, ErrorLine } from "../shared/common";
import { openVmNotice } from "../shared/levels";
import { SandboxPill } from "../shell/isolation";
import { t } from "../i18n";
import { codicon, ConfirmDialog, DataTable, EditorSection, IconButton, Link, Notice } from "../workbench";

const AGENT_NAMES: Record<AgentId, string> = {
  claude: "Claude Code",
  codex: "Codex",
  pi: "Pi",
  antigravity: "Antigravity",
};

/** Whose sandbox it is, as the panel titles it. */
function whose(s: SandboxReport): string {
  return s.agent ? t("Sandbox di {0}", AGENT_NAMES[s.agent]) : t("Sandbox del progetto");
}

export function ProjectVm({ project }: { project: string }) {
  const load = useCallback(() => projectVmReport(project), [project]);
  const { data, setData, error, reload } = useLoad<VmReport | null>(load, null, { onError: "show" });
  if (!data) return error ? <ErrorLine error={error} /> : <p className="text-vsc-desc">{t("Caricamento…")}</p>;
  const checked = (s: SandboxReport) => {
    setData({ ...data, sandboxes: data.sandboxes.map((x) => (x.name === s.name ? s : x)) });
  };
  return (
    <EditorSection
      id="project-vm"
      title={t("VM del progetto")}
      actions={<IconButton icon={codicon("refresh")} label={t("Aggiorna")} onClick={reload} />}
    >
      {data.vm === "off" && (
        <Notice tone="warning">
          <p>{data.reason ?? t("La VM non è attiva.")}</p>
          <Link onClick={openVmNotice}>{t("Apri l'avviso VM")}</Link>
        </Notice>
      )}
      <ul className="space-y-vsc-size160">
        {data.sandboxes.map((s) => (
          <li key={s.name}>
            <SandboxCard project={project} s={s} on={data.vm === "on"} onChanged={reload} onChecked={checked} />
          </li>
        ))}
      </ul>
      <p className="text-[12px] text-vsc-desc">{data.github ? t("GitHub nella VM: sì") : t("GitHub nella VM: no")}</p>
    </EditorSection>
  );
}

function SandboxCard({
  project,
  s,
  on,
  onChanged,
  onChecked,
}: {
  project: string;
  s: SandboxReport;
  on: boolean;
  onChanged: () => void;
  onChecked: (s: SandboxReport) => void;
}) {
  // An action reads the report again once done; a check keeps its answer.
  const actions = useAction(onChanged);
  const checks = useAction();
  const busy = actions.busy ?? checks.busy;
  const error = actions.error ?? checks.error;
  const fix = actions.fix ?? checks.fix;
  /** The action asked in the page before it runs: «Ricrea», or «Riavvia»
   * with sessions to stop. */
  const [asked, setAsked] = useState<VmAction | null>(null);
  const act = (action: VmAction) => {
    setAsked(null);
    actions.run(action, () => projectVmAction(project, s.agent, action).then(() => undefined));
  };
  const check = () => {
    checks.run("check", () => projectVmCheck(project, s.agent).then(onChecked));
  };
  const off = !on || busy !== null;
  return (
    <section aria-label={whose(s)} className="space-y-vsc-size40">
      <div className="flex flex-wrap items-center gap-vsc-size80">
        <span className="font-vsc-semibold text-vsc-fg">{whose(s)}</span>
        <SandboxPill state={s.state} busy={busy !== null && busy !== "check"} />
        <code className="min-w-0 truncate text-[12px] text-vsc-desc" title={s.name}>
          {s.name}
        </code>
      </div>
      {s.mounts.length > 0 && (
        <ul aria-label={t("Cartelle montate")} className="text-[12px]">
          {s.mounts.map((m) => (
            <li key={m.host} className="flex gap-vsc-size80">
              <span className={m.mode === "rw" ? "font-vsc-semibold text-vsc-warning" : "text-vsc-desc"}>
                {m.mode === "rw" ? t("lettura e scrittura") : t("sola lettura")}
              </span>
              <span className="min-w-0 truncate" title={m.host}>
                {m.guest}
              </span>
            </li>
          ))}
        </ul>
      )}
      {s.hostPorts.length > 0 && (
        <p className="text-[12px] text-vsc-desc">
          {t("Porte del PC raggiunte: {0}", s.hostPorts.map((p) => `${String(p.port)} (${p.what})`).join(", "))}
        </p>
      )}
      {s.sessions.length > 0 && (
        <p className="text-[12px] text-vsc-desc">{t("Sessioni in corso: {0}", s.sessions.join(", "))}</p>
      )}
      <div className="flex flex-wrap gap-vsc-size80">
        <ActionButton
          icon={codicon("debug-restart")}
          label={t("Riavvia")}
          small
          busy={busy === "restart"}
          disabled={off}
          onClick={() => {
            if (s.sessions.length > 0) setAsked("restart");
            else act("restart");
          }}
        />
        <ActionButton
          icon={codicon("trash")}
          label={t("Ricrea")}
          small
          busy={busy === "recreate"}
          disabled={off}
          onClick={() => {
            setAsked("recreate");
          }}
        />
        <ActionButton
          icon={codicon("cloud-download")}
          label={t("Aggiorna strumenti")}
          small
          busy={busy === "refresh"}
          disabled={off}
          onClick={() => {
            act("refresh");
          }}
        />
        <ActionButton
          icon={codicon("checklist")}
          label={t("Verifica strumenti")}
          small
          busy={busy === "check"}
          disabled={off}
          onClick={check}
        />
      </div>
      <ErrorLine error={error} fix={fix} />
      {s.note && <Notice tone="info">{s.note}</Notice>}
      {s.tools && <ToolsTable tools={s.tools} />}
      {asked && (
        <ConfirmDialog
          title={asked === "recreate" ? t("Ricreare {0}?", whose(s)) : t("Riavviare {0}?", whose(s))}
          cancelLabel={t("Annulla")}
          onCancel={() => {
            setAsked(null);
          }}
          actions={
            <ActionButton
              icon={codicon(asked === "recreate" ? "trash" : "debug-restart")}
              label={asked === "recreate" ? t("Ricrea") : t("Riavvia")}
              primary
              onClick={() => {
                act(asked);
              }}
            />
          }
        >
          {asked === "recreate" && (
            <>
              <p>{t("Si perde: la home della VM e le immagini che ha scaricato.")}</p>
              <p>
                {t("Resta: il progetto, i suoi database, gli strumenti del kit e gli accessi che il kit conserva.")}
              </p>
            </>
          )}
          {s.sessions.length > 0 && <p>{t("Si fermano: {0}", s.sessions.join(", "))}</p>}
        </ConfirmDialog>
      )}
    </section>
  );
}

/** The verdict of a tool as the table says it: never the colour alone. */
function verdictText(c: ToolCheck): string {
  switch (c.verdict) {
    case "ok":
      return t("✓ come scelto");
    case "otherLine":
      return t("✗ atteso {0}, trovato {1}", c.chosen ?? "?", c.found ?? "?");
    case "missing":
      return t("✗ manca");
    case "notFromKit":
      return t("✗ non è del kit: {0}", c.path ?? "?");
    case "notInVm":
      return t("non gira nella VM");
  }
}

const VERDICT_CLASS: Record<ToolCheck["verdict"], string> = {
  ok: "text-vsc-success",
  otherLine: "text-vsc-error",
  missing: "text-vsc-error",
  notFromKit: "text-vsc-error",
  notInVm: "text-vsc-desc",
};

function ToolsTable({ tools }: { tools: ToolCheck[] }) {
  return (
    <DataTable
      caption={t("Strumenti nella VM")}
      columns={[t("Strumento"), t("Scelto"), t("Trovato"), t("Esito")]}
      className="w-full text-[12px]"
    >
      {tools.map((c) => (
        <tr key={c.id} title={c.command ? `${c.command}\n${c.output}` : undefined}>
          <th scope="row" className="px-vsc-size80 text-left font-normal">
            {c.name}
          </th>
          <td className="px-vsc-size80">{c.chosen ?? "—"}</td>
          <td className="px-vsc-size80">{c.found ?? "—"}</td>
          <td className={`px-vsc-size80 ${VERDICT_CLASS[c.verdict]}`}>{verdictText(c)}</td>
        </tr>
      ))}
    </DataTable>
  );
}
