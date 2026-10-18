// «Matrice», an overview of the Tools page (D-121): one row per component,
// one column per system (Rust `router::probe::probe_all_targets`), each
// cell with the state of the component there and its details on a click.
// The column of the system shown by «Piattaforma» is highlighted; the
// details can make another one the shown system.
import { useState } from "react";
import { VscVm } from "react-icons/vsc";
import type { ComponentInfo, TargetId, TargetProfile, TargetToolStatus } from "../api";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { ComponentLogo } from "../logos";
import { Tag } from "../tags";
import { DataTable, Dot, Link } from "../workbench";
import { groups } from "./model";
import type { ComponentsState } from "./state";
import {
  CELL_STATES,
  cellHint,
  cellLabel,
  cellReason,
  cellState,
  targetMark,
  targetName,
  type CellState,
} from "./targets";

const GLYPH: Record<CellState, { glyph: string; tone: string }> = {
  available: { glyph: "✓", tone: "text-vsc-success" },
  emulated: { glyph: "≈", tone: "text-vsc-link" },
  staged: { glyph: "↓", tone: "text-vsc-success" },
  firstStart: { glyph: "◷", tone: "text-vsc-warning" },
  unavailable: { glyph: "✕", tone: "text-vsc-desc" },
};

function Glyph({ state }: { state: CellState }) {
  const g = GLYPH[state];
  return (
    <span aria-hidden="true" className={`inline-block w-3 text-center font-mono font-bold ${g.tone}`}>
      {g.glyph}
    </span>
  );
}

/** The details of a cell: its state and why, what is on disk, and the
 * actions (open the component, show that system in the other tabs). */
function CellDetail({
  c,
  s,
  profile,
  shown,
  onShow,
  onOpen,
}: {
  c: ComponentInfo;
  s: TargetToolStatus;
  profile: TargetProfile;
  shown: boolean;
  onShow: () => void;
  onOpen: () => void;
}) {
  const state = cellState(s);
  const reason = cellReason(s);
  const mark = targetMark(profile);
  const yes = (on: boolean) => (on ? t("sì") : t("no"));
  return (
    <section
      aria-label={t("{0} su {1}", c.name, targetName(profile.id))}
      className="space-y-2 border-t border-vsc-border px-5 py-3"
    >
      <h3 className="flex flex-wrap items-center gap-2 text-[13px] font-semibold">
        <ComponentLogo id={c.id} size={16} />
        {t("{0} su {1}", c.name, targetName(profile.id))}
        {mark && <Tag>{mark}</Tag>}
      </h3>
      <p className="flex items-center gap-1.5">
        <Glyph state={state} />
        <span>{cellLabel(state)}</span>
        {reason && <span className="text-vsc-desc">— {reason}</span>}
      </p>
      <p className="text-[12px] text-vsc-desc">{cellHint(state)}</p>
      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-0.5 text-[12px]">
        <dt className="text-vsc-desc">{t("Versione")}</dt>
        <dd className="font-mono select-text">{s.version ?? "—"}</dd>
        <dt className="text-vsc-desc">{t("Installato")}</dt>
        <dd>{yes(s.installed)}</dd>
        <dt className="text-vsc-desc">{t("Scelto")}</dt>
        <dd>{yes(s.chosen)}</dd>
        {s.needsReinstall && (
          <>
            <dt className="text-vsc-desc">{t("Da reinstallare")}</dt>
            <dd>{t("sì")}</dd>
          </>
        )}
      </dl>
      <div className="flex flex-wrap items-center gap-3">
        <Link onClick={onOpen}>{t("Apri {0}", c.name)}</Link>
        {!shown && <ActionButton icon={VscVm} label={t("Mostra questo sistema")} onClick={onShow} />}
      </div>
    </section>
  );
}

export function TargetMatrix({ state, onOpen }: { state: ComponentsState; onOpen: (id: string) => void }) {
  const { grid, gridError, list, target, host, setTarget } = state;
  const [picked, setPicked] = useState<{ id: string; target: TargetId } | null>(null);
  if (gridError && !grid) {
    return (
      <div className="px-5 py-4">
        <ErrorLine error={gridError} />
      </div>
    );
  }
  if (!grid || !list) return <p className="px-5 py-4 text-vsc-desc">{t("Caricamento…")}</p>;

  const shown = target ?? host;
  const status = (id: string, of: TargetId) =>
    grid.targets.find((s) => s.target === of)?.tools.find((x) => x.id === id);
  const cell = "border-b border-l border-vsc-border px-2 text-center";
  const pickedC = picked && list.find((c) => c.id === picked.id);
  const pickedS = picked && status(picked.id, picked.target);
  const pickedP = picked && grid.profiles.find((p) => p.id === picked.target);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="shrink-0 space-y-2 px-5 pt-4 pb-2">
        <p className="text-[12px] text-vsc-desc">
          {t(
            "Lo stato di ogni componente sui sei sistemi che il kit supporta: lo stesso kit su una chiavetta si apre su ciascuno. Fai clic su una casella per il dettaglio.",
          )}
        </p>
        <ul aria-label={t("Legenda")} className="flex flex-wrap gap-x-4 gap-y-1 text-[12px]">
          {CELL_STATES.map((s) => (
            <li key={s} className="flex items-center gap-1.5" title={cellHint(s)}>
              <Glyph state={s} />
              {cellLabel(s)}
            </li>
          ))}
          <li className="flex items-center gap-1.5">
            <Dot tone="ok" />
            {t("installato")}
          </li>
        </ul>
      </div>
      <div className="min-h-0 flex-1 overflow-auto px-5 pb-4">
        <DataTable
          caption={t("Matrice componente-sistema")}
          stickyHead
          className="border-separate border-spacing-0 text-[12px]"
          head={
            <>
              <th scope="col" className="px-2 py-[5px] text-left align-bottom font-bold">
                {t("Componente")}
              </th>
              {grid.profiles.map((p) => {
                const summary = grid.targets.find((s) => s.target === p.id);
                const mark = targetMark(p);
                return (
                  <th
                    key={p.id}
                    scope="col"
                    className={`border-l border-vsc-border px-2 py-[5px] text-center align-bottom font-bold ${
                      p.id === shown ? "bg-vsc-focus/15" : ""
                    }`}
                  >
                    <span className="block whitespace-nowrap">{targetName(p.id)}</span>
                    <span className="block font-mono text-[11px] font-normal text-vsc-desc">{p.id}</span>
                    {mark && <Tag className="mt-0.5">{mark}</Tag>}
                    {summary && (
                      <span
                        className="block font-normal text-vsc-desc tabular-nums"
                        title={t("installati su disponibili")}
                      >
                        {summary.installedCount}/{summary.availableCount}
                      </span>
                    )}
                  </th>
                );
              })}
            </>
          }
        >
          {groups().map((g) => {
            const items = list.filter((c) => g.has(c));
            if (items.length === 0) return null;
            return [
              <tr key={`group-${g.id}`}>
                <th
                  scope="colgroup"
                  colSpan={grid.profiles.length + 1}
                  className="border-b border-vsc-border bg-vsc-list-hover px-2 py-1 text-left text-[11px] font-semibold tracking-wide text-vsc-desc uppercase"
                >
                  {g.title}
                </th>
              </tr>,
              ...items.map((c) => (
                <tr key={c.id} className="hover:bg-vsc-list-hover">
                  <th scope="row" className="h-[26px] border-b border-vsc-border px-2 text-left font-normal">
                    <span className="flex items-center gap-2 whitespace-nowrap">
                      <ComponentLogo id={c.id} size={16} />
                      {c.name}
                    </span>
                  </th>
                  {grid.profiles.map((p) => {
                    const s = status(c.id, p.id);
                    if (!s) return <td key={p.id} className={cell} />;
                    const st = cellState(s);
                    const reason = cellReason(s);
                    const label = `${c.name}, ${targetName(p.id)}: ${cellLabel(st)}${reason ? ` (${reason})` : ""}`;
                    const on = picked?.id === c.id && picked.target === p.id;
                    return (
                      <td key={p.id} className={`${cell} ${p.id === shown ? "bg-vsc-focus/10" : ""}`}>
                        <button
                          type="button"
                          aria-label={label}
                          aria-pressed={on}
                          title={label}
                          onClick={() => {
                            setPicked(on ? null : { id: c.id, target: p.id });
                          }}
                          className={`inline-flex h-5 w-10 items-center justify-center gap-1 rounded-sm hover:bg-vsc-list-hover ${
                            on ? "outline outline-1 outline-vsc-focus" : ""
                          }`}
                        >
                          <Glyph state={st} />
                          {s.installed && <Dot tone="ok" />}
                        </button>
                      </td>
                    );
                  })}
                </tr>
              )),
            ];
          })}
        </DataTable>
      </div>
      {pickedC && pickedS && pickedP && (
        <CellDetail
          c={pickedC}
          s={pickedS}
          profile={pickedP}
          shown={pickedP.id === shown}
          onShow={() => {
            setTarget(pickedP.id);
          }}
          onOpen={() => {
            onOpen(pickedC.id);
          }}
        />
      )}
    </div>
  );
}
