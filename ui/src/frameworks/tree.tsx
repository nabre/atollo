// The tree of the frameworks in the side bar of Settings.
import { useState } from "react";
import type { FrameworkPiece } from "../api";
import { t } from "../i18n";
import { VscBeaker } from "react-icons/vsc";
import { List, ListRow } from "../workbench";
import { bases, cleaned, pieceTab, slotLabel, textOf, TRY_TAB, type Frameworks } from "./model";
import { PieceLogo } from "./tags";

/** The side bar's tree of the frameworks: bases, additions by slot, the
 * trial of a combination. */
export function FrameworksTree({
  fw,
  selected,
  onSelect,
}: {
  fw: Frameworks;
  selected: string | null;
  onSelect: (tab: string) => void;
}) {
  const { view, pieces } = fw;
  const [closed, setClosed] = useState<Set<string>>(() => new Set());
  const modified = (p: FrameworkPiece) => {
    const official = view?.official.find((o) => o.id === p.id);
    return official ? cleaned(official) !== cleaned(p) : true;
  };
  const row = (p: FrameworkPiece) => (
    <ListRow
      key={p.id}
      depth={1}
      icon={<PieceLogo piece={p} />}
      label={p.name || p.id}
      title={textOf(view, p.desc)}
      selected={selected === pieceTab(p.id)}
      current="page"
      onSelect={() => {
        onSelect(pieceTab(p.id));
      }}
      decorations={
        modified(p) ? (
          <span title={t("Modificata")} className="text-vsc-settings-modified">
            M
          </span>
        ) : undefined
      }
    />
  );
  const branch = (key: string, label: string, list: FrameworkPiece[]) => {
    if (list.length === 0) return null;
    const open = !closed.has(key);
    return (
      <ListRow
        key={key}
        twistie={open ? "open" : "closed"}
        label={label}
        description={String(list.length)}
        onSelect={() => {
          setClosed((c) => {
            const next = new Set(c);
            if (open) next.add(key);
            else next.delete(key);
            return next;
          });
        }}
      >
        <List label={label}>{list.map(row)}</List>
      </ListRow>
    );
  };
  const addons = pieces.filter((p) => p.kind === "addon");
  const slots = [...new Set(addons.map((p) => p.slot ?? ""))];
  return (
    <List label={t("Framework")}>
      {branch("bases", t("Basi"), bases(pieces))}
      {slots.map((s) =>
        branch(
          `slot:${s}`,
          t("Aggiunte: {0}", slotLabel(s)),
          addons.filter((p) => (p.slot ?? "") === s),
        ),
      )}
      <ListRow
        icon={<VscBeaker size={16} aria-hidden="true" />}
        label={t("Prova una combinazione")}
        selected={selected === TRY_TAB}
        current="page"
        onSelect={() => {
          onSelect(TRY_TAB);
        }}
      />
    </List>
  );
}
