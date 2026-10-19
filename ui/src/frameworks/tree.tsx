// The tree of the frameworks in the side bar of Settings: the bases by
// group (web with PHP, CMS, desktop…), under each base the additions that
// go with it by slot, each one opening what it does with that base; then
// all the additions by slot and the trial of a combination.
import { useState, type ReactNode } from "react";
import type { FrameworkPiece } from "../api";
import { t } from "../i18n";
import { VscBeaker } from "react-icons/vsc";
import { List, ListRow } from "../workbench";
import {
  basesByCategory,
  cleaned,
  integrationsOf,
  integrationTab,
  parseIntegrationTab,
  pieceTab,
  slotLabel,
  textOf,
  TRY_TAB,
  type Frameworks,
} from "./model";
import { PieceLogo } from "./tags";

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
  /** Branches closed by hand (groups, slots: open at first). */
  const [closed, setClosed] = useState<Set<string>>(() => new Set());
  /** Bases opened by hand (closed at first), besides the one of the tab shown. */
  const [opened, setOpened] = useState<Record<string, boolean>>({});
  const shownBase = selected ? (parseIntegrationTab(selected)?.base ?? null) : null;
  const modified = (p: FrameworkPiece) => {
    const official = view?.official.find((o) => o.id === p.id);
    return official ? cleaned(official) !== cleaned(p) : true;
  };
  const mark = (p: FrameworkPiece) =>
    modified(p) ? (
      <span title={t("Modificata")} className="text-vsc-settings-modified">
        M
      </span>
    ) : undefined;
  const row = (p: FrameworkPiece, depth = 1) => (
    <ListRow
      key={p.id}
      depth={depth}
      icon={<PieceLogo piece={p} />}
      label={p.name || p.id}
      title={textOf(view, p.desc)}
      selected={selected === pieceTab(p.id)}
      current="page"
      onSelect={() => {
        onSelect(pieceTab(p.id));
      }}
      decorations={mark(p)}
    />
  );
  const flip = (key: string, open: boolean) => {
    setClosed((c) => {
      const next = new Set(c);
      if (open) next.add(key);
      else next.delete(key);
      return next;
    });
  };
  const branch = (key: string, label: string, count: number, depth: number, children: ReactNode) => {
    if (count === 0) return null;
    const open = !closed.has(key);
    return (
      <ListRow
        key={key}
        depth={depth}
        twistie={open ? "open" : "closed"}
        label={label}
        description={String(count)}
        onSelect={() => {
          flip(key, open);
        }}
      >
        <List label={label}>{children}</List>
      </ListRow>
    );
  };
  /** A base: its tab, and under it the additions that go with it. */
  const baseRow = (b: FrameworkPiece, depth: number) => {
    const slots = integrationsOf(pieces, b.id);
    if (slots.length === 0) return row(b, depth);
    const open = opened[b.id] ?? shownBase === b.id;
    const tab = pieceTab(b.id);
    return (
      <ListRow
        key={b.id}
        depth={depth}
        twistie={open ? "open" : "closed"}
        icon={<PieceLogo piece={b} />}
        label={b.name || b.id}
        title={textOf(view, b.desc)}
        selected={selected === tab}
        current="page"
        onSelect={() => {
          // Shown already: the row opens and closes; else its tab, opened.
          setOpened((all) => ({ ...all, [b.id]: selected === tab ? !open : true }));
          onSelect(tab);
        }}
        decorations={mark(b)}
      >
        <List label={t("Aggiunte con {0}", b.name || b.id)}>
          {slots.map((s) =>
            branch(
              `with:${b.id}:${s.slot}`,
              slotLabel(s.slot),
              s.addons.length,
              depth + 1,
              s.addons.map((a) => {
                const at = integrationTab(a.id, b.id);
                return (
                  <ListRow
                    key={a.id}
                    depth={depth + 2}
                    icon={<PieceLogo piece={a} />}
                    label={a.name || a.id}
                    title={t("{0} con {1}", a.name || a.id, b.name || b.id)}
                    selected={selected === at}
                    current="page"
                    onSelect={() => {
                      onSelect(at);
                    }}
                    decorations={mark(a)}
                  />
                );
              }),
            ),
          )}
        </List>
      </ListRow>
    );
  };
  const groups = basesByCategory(pieces);
  const addons = pieces.filter((p) => p.kind === "addon");
  const slots = [...new Set(addons.map((p) => p.slot ?? ""))];
  return (
    <List label={t("Framework")}>
      {branch(
        "bases",
        t("Basi"),
        groups.reduce((n, g) => n + g.bases.length, 0),
        0,
        groups.map((g) =>
          branch(
            `category:${g.id}`,
            g.label,
            g.bases.length,
            1,
            g.bases.map((b) => baseRow(b, 2)),
          ),
        ),
      )}
      {branch(
        "addons",
        t("Aggiunte"),
        addons.length,
        0,
        slots.map((s) => {
          const list = addons.filter((p) => (p.slot ?? "") === s);
          return branch(
            `slot:${s}`,
            slotLabel(s),
            list.length,
            1,
            list.map((p) => row(p, 2)),
          );
        }),
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
