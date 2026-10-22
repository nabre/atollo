// The app's widgets of the Guide that show the app's own data or filter
// it: the kit's card, the table of sources with the state of each
// component on this PC, the problems with their search box. The site draws
// the same widgets without the data (site/guide.mjs).
import { useMemo, useState, type ReactNode } from "react";
import { urlOpen } from "../api";
import { ComponentLogo } from "../shared/logos";
import { Tag } from "../shared/tags";
import { Button, Codicon, Properties, SearchBox } from "../workbench";
import { guideLabels } from "@guide/core/labels";
import { rich } from "@guide/core/lang";
import type { SourceCategory } from "@guide/core/types";
import type { GuideWidgetProps as W } from "@guide/core/widgets";
import { Code, P, QA, Section, See, Table } from "./widgets";

/** A short label: the app's tag. */
export function Badge({ tone, children }: W["Badge"]) {
  return <Tag {...(tone ? { tone } : {})}>{children}</Tag>;
}

/** The kit's card, or its folder, when the app knows the kit. */
export function KitInfo({ info, show, text }: W["KitInfo"]) {
  if (!info) return null;
  if (show === "root") {
    const root = <Code>{info.root}</Code>;
    return text ? <P>{rich(text, root)}</P> : root;
  }
  const l = guideLabels().kit;
  const rows: [string, ReactNode][] = [
    [l.version, <Tag key="v" tone="focus">{`v${info.version}`}</Tag>],
    [l.root, <Code key="r">{info.root}</Code>],
    [l.projects, <Code key="p">{info.projects}</Code>],
  ];
  if (info.diskFs) rows.push([l.diskFs, <Tag key="fs">{info.diskFs}</Tag>]);
  return (
    <Section title={l.title}>
      <Properties rows={rows} />
    </Section>
  );
}

/** The external sources of every component, with a filter and a search. */
export function SourcesTable({ rows: all, categories, components }: W["SourcesTable"]) {
  const l = guideLabels().sources;
  const [query, setQuery] = useState("");
  const [cat, setCat] = useState<"all" | SourceCategory>("all");
  const byId = useMemo(() => new Map((components ?? []).map((c) => [c.id, c])), [components]);
  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return all.filter(
      (r) =>
        (cat === "all" || r.group === cat) &&
        (!q ||
          [r.id, r.name, r.home, r.metaSource, r.binarySource, r.verifyKind, r.targetPath].some((s) =>
            s.toLowerCase().includes(q),
          )),
    );
  }, [all, query, cat]);
  const cats: { id: "all" | SourceCategory; label: string }[] = [
    { id: "all", label: `${l.all} (${String(all.length)})` },
    ...categories,
  ];
  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-1.5">
          {cats.map((c) => (
            <Button
              key={c.id}
              small
              variant={cat === c.id ? "primary" : "secondary"}
              onClick={() => {
                setCat(c.id);
              }}
            >
              {c.label}
            </Button>
          ))}
        </div>
        <div className="w-64">
          <SearchBox value={query} onChange={setQuery} placeholder={l.filter} />
        </div>
      </div>
      <Table
        columns={l.columns}
        rows={rows.map((r) => {
          const live = byId.get(r.id);
          const state: ReactNode = live?.installed ? (
            <Tag tone="focus">{live.version ?? l.installed}</Tag>
          ) : live && (!live.available || live.noArm64 || live.unavailableReason !== null) ? (
            <Tag title={live.unavailableReason ?? undefined}>{l.notHere}</Tag>
          ) : (
            <Tag>{live ? l.notInstalled : l.withComponent}</Tag>
          );
          return [
            <div key="n" className="space-y-1">
              <div className="flex items-center gap-2">
                <ComponentLogo id={r.logoId} size={16} />
                <span className="font-semibold text-vsc-settings-header">{r.name}</span>
              </div>
              <div className="flex flex-wrap items-center gap-1.5 pl-6">
                {state}
                <button
                  type="button"
                  onClick={() => {
                    void urlOpen(r.home);
                  }}
                  title={r.home}
                  className="inline-flex items-center gap-0.5 text-[11px] text-vsc-link hover:underline"
                >
                  {l.website}
                  <Codicon name="link-external" size={10} />
                </button>
              </div>
            </div>,
            <div key="m" className="font-mono text-[11px] break-all select-text">
              {r.metaSource}
            </div>,
            <div key="b" className="space-y-1">
              <div className="font-mono text-[11px] break-all text-vsc-desc select-text">{r.binarySource}</div>
              {r.unix && <div className="text-[11px] select-text">{r.unix}</div>}
            </div>,
            <div key="v" className="space-y-0.5">
              <Tag tone="focus">{r.verifyKind}</Tag>
              <div className="text-[11px] text-vsc-desc">{r.verifyDetail}</div>
            </div>,
            <code key="p" className="font-mono text-[11px] break-all select-text">
              {r.targetPath}
            </code>,
          ];
        })}
      />
    </>
  );
}

/** The problems with a search box, by area. */
export function FilterList({ groups: all, placeholder }: W["FilterList"]) {
  const [q, setQ] = useState("");
  const groups = useMemo(() => {
    const s = q.trim().toLowerCase();
    return all
      .map((g) => ({ ...g, items: g.items.filter((p) => !s || `${p.q} ${p.a}`.toLowerCase().includes(s)) }))
      .filter((g) => g.items.length > 0);
  }, [all, q]);
  return (
    <>
      <div className="w-full max-w-md">
        <SearchBox value={q} onChange={setQ} placeholder={placeholder} />
      </div>
      {groups.map((g) => (
        <Section key={g.title} title={g.title}>
          {g.items.map((p) => (
            <QA key={p.q} q={p.q}>
              <P>{p.a}</P>
              {p.see && (
                <P>
                  {guideLabels().learnMore} <See id={p.see} />
                </P>
              )}
            </QA>
          ))}
        </Section>
      ))}
    </>
  );
}
