// The plan of a combination: the additions to choose and what it does;
// tried in Settings, shown by the wizard.
import type { ReactNode } from "react";
import type { FrameworkPiece, FrameworkPlan, FrameworksView } from "../api";
import { ErrorLine } from "../shared/common";
import { t } from "../i18n";
import { CheckboxLabel, Notice, Select, SettingItem, SettingsGroup } from "../workbench";
import { bases, slotLabel, textOf, useCombination, useComponents, usePlan, type Frameworks } from "./model";
import { PieceLogo, ToolTag } from "./tags";

/** The trial of a combination as a group of settings. */
export function TryGroup({ fw }: { fw: Frameworks }) {
  return (
    <SettingsGroup
      id="s-fw-try"
      title={t("Prova una combinazione")}
      description={t("Che cosa farebbe una combinazione, con le istruzioni come sono qui (anche se non salvate).")}
    >
      <SettingItem title={t("Prova una combinazione")} keywords="wizard framework">
        <TryPlan view={fw.view} pieces={fw.pieces} />
      </SettingItem>
    </SettingsGroup>
  );
}

/** The additions of a base, grouped by slot, as checkboxes. */
export function AddonChoice({
  view,
  base,
  available,
  addons,
  toggle,
  disabled = false,
}: {
  view: FrameworksView | null;
  base: string;
  available: FrameworkPiece[];
  addons: string[];
  toggle: (id: string, on: boolean) => void;
  disabled?: boolean;
}) {
  if (available.length === 0) return <p className="text-vsc-desc">{t("Nessuna aggiunta per questa base.")}</p>;
  const slots = [...new Set(available.map((p) => p.slot ?? ""))];
  return (
    <div className="space-y-3">
      {slots.map((slot) => (
        <div key={slot}>
          <div className="mb-1 text-[11px] font-semibold tracking-wide text-vsc-desc uppercase">{slotLabel(slot)}</div>
          <div className="space-y-1.5">
            {available
              .filter((p) => (p.slot ?? "") === slot)
              .map((p) => {
                const note = textOf(view, p.on?.[base]?.note);
                return (
                  <CheckboxLabel
                    key={p.id}
                    checked={addons.includes(p.id)}
                    disabled={disabled}
                    onChange={(on) => {
                      toggle(p.id, on);
                    }}
                  >
                    <span className="inline-flex translate-y-0.5 pr-1">
                      <PieceLogo piece={p} size={14} />
                    </span>
                    <span className="font-semibold">{p.name}</span>
                    <span className="text-vsc-desc">
                      {" "}
                      - {textOf(view, p.desc)}
                      {note && ` ${note}`}
                    </span>
                  </CheckboxLabel>
                );
              })}
          </div>
        </div>
      ))}
    </div>
  );
}

function Commands({ title, lines: list }: { title: string; lines: string[] }) {
  if (list.length === 0) return null;
  return (
    <div>
      <div className="mb-1 text-vsc-desc">{title}</div>
      <ol className="m-0 list-decimal space-y-0.5 pl-6 font-mono text-[12px] break-all select-text">
        {list.map((l, i) => (
          <li key={String(i) + l}>{l}</li>
        ))}
      </ol>
    </div>
  );
}

/** What a combination does: tools, database, .env, commands, actions, notes. */
export function PlanView({
  view,
  plan,
  error,
}: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
}): ReactNode {
  return <PlanDetails view={view} plan={plan} error={error} showEnv />;
}

function PlanDetails({
  view,
  plan,
  error,
  showEnv,
}: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
  showEnv: boolean;
}): ReactNode {
  const components = useComponents();
  if (error) return <ErrorLine error={error} />;
  if (!plan) return null;
  return (
    <div className="space-y-2.5">
      <div className="flex flex-wrap items-center gap-1">
        <span className="pr-1 text-vsc-desc">{t("Strumenti del kit")}:</span>
        {plan.needs.length === 0 ? "-" : plan.needs.map((id) => <ToolTag key={id} id={id} components={components} />)}
      </div>
      {plan.database && (
        <div className="flex flex-wrap items-center gap-1">
          <span className="pr-1 text-vsc-desc">{t("Database del progetto")}:</span>
          <ToolTag id={plan.database} components={components} />
        </div>
      )}
      {showEnv && plan.env.length > 0 && (
        <div>
          <div className="mb-1 text-vsc-desc">{t("File .env")}</div>
          <ul className="m-0 list-none space-y-0.5 p-0 font-mono text-[12px] select-text">
            {plan.env.map((e) => (
              <li key={e.key}>
                {e.key}={e.value}
              </li>
            ))}
          </ul>
        </div>
      )}
      <Commands title={t("Nella cartella projects")} lines={plan.create} />
      <Commands title={t("Nella cartella del progetto")} lines={plan.steps} />
      {plan.actions.length > 0 && (
        <div>
          <div className="mb-1 text-vsc-desc">{t("Azioni del progetto")}</div>
          <ul className="m-0 list-none space-y-0.5 p-0 text-[12px]">
            {plan.actions.map((a) => (
              <li key={a.name}>
                <span className="font-semibold">{a.name}</span>{" "}
                <span className="font-mono text-vsc-desc">{a.command}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {plan.notes.map((n) => (
        <Notice key={n} tone="info">
          {textOf(view, n)}
        </Notice>
      ))}
    </div>
  );
}

/** The plan without the .env, which the wizard shows as fields of its own. */
export function PlanSummary(props: {
  view: FrameworksView | null;
  plan: FrameworkPlan | null;
  error: string | null;
}): ReactNode {
  return <PlanDetails {...props} showEnv={false} />;
}

function TryPlan({ view, pieces }: { view: FrameworksView | null; pieces: FrameworkPiece[] }) {
  const { base, setBase, addons, toggle, available } = useCombination(pieces);
  const { plan, error } = usePlan(pieces, base, addons);
  return (
    <div className="space-y-3">
      <Select
        label={t("Base")}
        value={base}
        onChange={setBase}
        options={bases(pieces).map((b) => ({ value: b.id, label: b.name }))}
        className="w-full"
      />
      <AddonChoice view={view} base={base} available={available} addons={addons} toggle={toggle} />
      <PlanView view={view} plan={plan} error={error} />
    </div>
  );
}
