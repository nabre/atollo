// Every tool in one table, the default tab of the Tools page, in the
// site's kinds (system and editor, AI agents, languages, development tools,
// web servers, databases: model.ts), each under its title with what to know
// about it, its count and a box that chooses all of it, folded by a click.
// Filters by kind and state, sortable columns (inside each kind), the
// choice of each row applied with "Apply" as on the tool's own page, and
// the actions done at once: install (at a chosen version), add a version,
// update.
import { Fragment, useMemo, useState } from "react";
import {
  TARGETS,
  componentsInstall,
  setupStart,
  versionsAdd,
  versionsInstall,
  type ComponentInfo,
  type SetupSnapshot,
  type TargetId,
} from "../ipc";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../shared/common";
import { ComponentLogo } from "../shared/logos";
import { itemState } from "./setup";
import { Tag } from "../shared/tags";
import { AddLine, composerOf, RefreshLines, splitVersion, type KitVersions } from "../versions";
import { Checkbox, Codicon, codicon, DataTable, IconButton, Link, SearchBox, Select } from "../workbench";
import { groups, linesOf, matches, sizeText } from "./model";
import type { ComponentsState } from "./state";
import { IntegrityReport, useIntegrity } from "./integrity";
import { TOOL_STATES, ToolStateIcon, otherStatus, shownStatus, toolStateLabel } from "./status";
import { targetLabel, targetName } from "./targets";

type Sort = "name" | "version" | "size" | "state";
type Show = "all" | "installed" | "missing" | "pending" | "needed";

/** What a row can do now, without "Apply": install it (at its latest
 * version or at one chosen), add a version next to those on disk, update
 * it. Removing stays a choice applied with "Apply". */
function RowActions({ c, kit, setup }: { c: ComponentInfo; kit: KitVersions; setup: SetupSnapshot | null }) {
  const { busy, run, offers, target } = kit;
  const running = setup?.running ?? false;
  const tool = kit.byId.get(c.id);
  const lines = tool?.downloadable ? tool : null;
  const doing = running || busy !== null;
  const ready = c.installed || Boolean(c.bootstrapStaged);
  if (c.unavailableReason !== null || c.notNeeded !== null) return null;
  if (!ready) {
    if (!c.available) return null;
    return (
      <span className="flex items-center gap-1">
        <IconButton
          icon={codicon("cloud-download")}
          label={t("Installa {0}", c.name)}
          title={t("Installa {0} all'ultima versione, subito", c.name)}
          busy={busy === `install:${c.id}`}
          disabled={doing}
          className="text-vsc-link"
          onClick={() => {
            run(c.id, `install:${c.id}`, () => componentsInstall(c.id, target));
          }}
        />
        {lines && (
          <>
            <AddLine
              tool={lines}
              setup={setup}
              busy={doing}
              label={t("Installa una versione di {0}", c.name)}
              onAdd={(line) => {
                run(c.id, `install:${c.id}`, () => versionsInstall(c.id, line, target));
              }}
            />
            {lines.available.length === 0 && (
              <RefreshLines
                tool={lines}
                checking={offers.checking.has(c.id)}
                disabled={busy !== null}
                onRefresh={() => {
                  offers.refresh([c.id]);
                }}
              />
            )}
          </>
        )}
      </span>
    );
  }
  return (
    <span className="flex items-center gap-1">
      <IconButton
        icon={codicon("sync")}
        label={t("Aggiorna {0}", c.name)}
        title={t("Scarica la versione nuova di {0}, se c'è", c.name)}
        busy={busy === `update:${c.id}`}
        disabled={doing}
        onClick={() => {
          run(c.id, `update:${c.id}`, () => setupStart([c.id], true, target));
        }}
      />
      {lines && (
        <AddLine
          tool={lines}
          setup={setup}
          busy={doing}
          onAdd={(line) => {
            run(c.id, `add:${c.id}`, () => versionsAdd(c.id, line, target));
          }}
        />
      )}
    </span>
  );
}

export function ToolsTable({
  state,
  kit,
  setup,
  onOpen,
}: {
  state: ComponentsState;
  kit: KitVersions;
  setup: SetupSnapshot | null;
  onOpen: (id: string) => void;
}) {
  const list = state.list ?? [];
  const { chosen, choose, neededBy, target, setTarget, checks, checking, verifyTarget, grid, host } = state;
  const activeTarget = (target ?? host ?? setup?.target ?? "win32-x64") as TargetId;
  const running = setup?.running ?? false;
  const integrity = useIntegrity(target);
  // The state is a column per system, as in the «Matrice» overview: the
  // system shown, and with "Expand" each of the others.
  const [expanded, setExpanded] = useState(false);
  const allProfiles = grid?.profiles ?? [];
  const columns: TargetId[] = expanded && allProfiles.length > 0 ? allProfiles.map((p) => p.id) : [activeTarget];
  const status = (id: string, of: TargetId) =>
    grid?.targets.find((s) => s.target === of)?.tools.find((x) => x.id === id);
  const all = groups();
  const [search, setSearch] = useState("");
  const [group, setGroup] = useState("all");
  const [show, setShow] = useState<Show>("all");
  const [sort, setSort] = useState<{ by: Sort; up: boolean }>({ by: "name", up: true });
  const [folded, setFolded] = useState<Set<string>>(new Set());

  const groupOf = (c: ComponentInfo) => all.findIndex((g) => g.has(c));
  const fold = (id: string) => {
    setFolded((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const rows = useMemo(() => {
    const shown = list.filter((c) => {
      const on = chosen.has(c.id);
      const ready = c.installed || Boolean(c.bootstrapStaged);
      if (!matches(c, search)) return false;
      if (group !== "all" && all[groupOf(c)]?.id !== group) return false;
      switch (show) {
        case "installed":
          return ready;
        case "missing":
          // Not needed here (D-177) is not missing.
          return !ready && c.notNeeded === null;
        case "pending":
          return on !== (c.chosen || c.locked);
        case "needed":
          return (neededBy.get(c.id) ?? []).length > 0;
        default:
          return true;
      }
    });
    const key = (c: ComponentInfo): string | number => {
      switch (sort.by) {
        case "name":
          return c.name.toLowerCase();
        case "version":
          return c.version ?? "";
        case "size":
          return c.mb;
        case "state":
          return TOOL_STATES.indexOf(
            shownStatus(c, status(c.id, activeTarget), chosen.has(c.id), itemState(setup, c.id, c.name), activeTarget)
              .state,
          );
      }
    };
    return shown.sort((a, b) => {
      const x = key(a);
      const y = key(b);
      const d = typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y));
      return (sort.up ? d : -d) || a.name.localeCompare(b.name);
    });
    // groupOf and all only read the catalog's groups, fixed for the page.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [list, chosen, search, group, show, sort, neededBy, grid, activeTarget, setup]);

  /** The rows a box can choose: not required, installable or chosen. */
  const freeOf = (of: ComponentInfo[]) => of.filter((c) => !c.locked && (c.available || chosen.has(c.id)));
  const free = freeOf(rows);
  const allOn = free.length > 0 && free.every((c) => chosen.has(c.id));
  const someOn = free.some((c) => chosen.has(c.id));
  const header = (by: Sort, label: string, align: "left" | "right" | "center" = "left", extra = "", title?: string) => (
    <th
      scope="col"
      aria-sort={sort.by === by ? (sort.up ? "ascending" : "descending") : undefined}
      title={title}
      className={`p-0 ${extra}`}
    >
      <button
        type="button"
        onClick={() => {
          setSort((s) => ({ by, up: s.by === by ? !s.up : true }));
        }}
        className={`flex w-full items-center gap-1 px-2 py-[5px] leading-5 font-bold whitespace-nowrap hover:bg-vsc-list-hover ${
          { left: "", right: "justify-end", center: "justify-center" }[align]
        }`}
      >
        {label}
        {sort.by === by && (sort.up ? <Codicon name="arrow-up" size={12} /> : <Codicon name="arrow-down" size={12} />)}
      </button>
    </th>
  );

  const row = (c: ComponentInfo) => {
    const on = chosen.has(c.id);
    const pending = on !== (c.chosen || c.locked);
    const item = itemState(setup, c.id, c.name);
    const needed = neededBy.get(c.id) ?? [];
    const lines = linesOf(kit.byId.get(c.id));
    const version = composerOf(c.version) ? splitVersion(c.version).main : c.version;
    const check = checks.get(c.id);
    const cell = "h-[26px] border-b border-vsc-border px-2";
    return (
      <tr key={c.id} className={`hover:bg-vsc-list-hover ${pending ? "bg-vsc-settings-modified/5" : ""}`}>
        <td className={cell}>
          <Checkbox
            label={t("Scegli {0}", c.name)}
            checked={on}
            disabled={
              c.locked || running || c.unavailableReason !== null || c.notNeeded !== null || (!c.available && !on)
            }
            onChange={(v) => {
              choose(c.id, v);
            }}
          />
        </td>
        <td className={cell}>
          <span className="flex items-center gap-2 whitespace-nowrap">
            <ComponentLogo id={c.id} size={16} />
            <Link
              onClick={() => {
                onOpen(c.id);
              }}
            >
              {c.name}
            </Link>
            {c.locked && <Tag>{t("obbligatorio")}</Tag>}
          </span>
        </td>
        <td className={`${cell} font-mono whitespace-nowrap select-text`}>{version ?? "—"}</td>
        <td className={`${cell} font-mono whitespace-nowrap`}>
          {lines.length > 0 ? (
            lines.join(", ")
          ) : c.versioned ? (
            "—"
          ) : (
            <span className="font-sans text-vsc-desc">{t("sempre l'ultima")}</span>
          )}
        </td>
        <td className={`${cell} text-right whitespace-nowrap tabular-nums`}>{sizeText(c.mb)}</td>
        {columns.map((id) => {
          const here = id === activeTarget;
          const s = status(c.id, id);
          const st = here ? shownStatus(c, s, on, item, id) : s ? otherStatus(s, id) : null;
          if (!st) return <td key={id} className={cell} />;
          const parts = [toolStateLabel(st.state), st.detail];
          if (here && check) parts.push(`${t("Verifica")}: ${check.answer}`);
          const label = `${c.name}, ${targetName(id)}: ${parts.filter(Boolean).join(" — ")}`;
          return (
            <td key={id} className={`${cell} text-center whitespace-nowrap ${here ? "bg-vsc-focus/10" : ""}`}>
              <span role="img" aria-label={label} title={label} className="inline-flex items-center gap-1">
                <ToolStateIcon state={st.state} />
                {here &&
                  check &&
                  (check.ok ? (
                    <Codicon name="check" size={12} className="text-vsc-focus" />
                  ) : (
                    <Codicon name="warning" size={12} className="text-vsc-warning" />
                  ))}
              </span>
            </td>
          );
        })}
        <td className={`${cell} max-w-[16rem] truncate`} title={needed.join(", ") || undefined}>
          {needed.length > 0 && !on && (
            <Codicon
              name="warning"
              size={12}
              className="mr-1 text-vsc-warning"
              label={t("Serve a progetti e non è scelto")}
            />
          )}
          {needed.join(", ")}
        </td>
        <td className={`${cell} whitespace-nowrap text-vsc-desc`}>
          {c.needs.map((n) => list.find((x) => x.id === n)?.name ?? n).join(", ")}
        </td>
        <td className={cell}>
          <RowActions c={c} kit={kit} setup={setup} />
        </td>
      </tr>
    );
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex shrink-0 flex-wrap items-center gap-2 px-5 pt-4 pb-2">
        <div className="w-64">
          <SearchBox value={search} onChange={setSearch} placeholder={t("Cerca negli strumenti")} />
        </div>
        <Select
          label={t("Piattaforma")}
          value={activeTarget}
          onChange={setTarget}
          options={TARGETS.map((id) => ({ value: id, label: targetLabel(id, grid?.profiles ?? []) }))}
        />
        <Select
          label={t("Tipo")}
          value={group}
          onChange={setGroup}
          options={[{ value: "all", label: t("Tutti i tipi") }, ...all.map((g) => ({ value: g.id, label: g.title }))]}
        />
        <Select<Show>
          label={t("Stato")}
          value={show}
          onChange={setShow}
          options={[
            { value: "all", label: t("Tutti") },
            { value: "installed", label: t("Installati") },
            { value: "missing", label: t("Non installati") },
            { value: "pending", label: t("Da applicare") },
            { value: "needed", label: t("Usati dai progetti") },
          ]}
        />
        <ActionButton
          icon={expanded ? codicon("fold") : codicon("unfold")}
          label={expanded ? t("Riduci") : t("Espandi")}
          title={
            expanded
              ? t("Mostra solo lo stato sulla piattaforma scelta")
              : t("Mostra lo stato anche sulle altre piattaforme")
          }
          disabled={allProfiles.length < 2}
          onClick={() => {
            setExpanded((v) => !v);
          }}
        />
        <ActionButton
          icon={codicon("check")}
          label={t("Verifica")}
          busy={checking}
          disabled={running || checking}
          onClick={verifyTarget}
        />
        <ActionButton
          icon={codicon("file-binary")}
          label={t("Verifica i file")}
          title={t("Confronta i file dei programmi con quelli dell'installazione: trova quelli mancanti o danneggiati")}
          busy={integrity.busy === "check"}
          disabled={running || integrity.busy !== null}
          onClick={integrity.check}
        />
        <span className="ml-auto text-[12px] text-vsc-desc" role="status">
          {t("{0} di {1}", rows.length, list.length)}
        </span>
      </div>
      {(integrity.report !== null || integrity.error) && (
        <div className="px-5 pb-2">
          <IntegrityReport state={integrity} running={running} />
        </div>
      )}
      {kit.error && (
        <div className="px-5 pb-2">
          <ErrorLine
            error={kit.last ? `${list.find((c) => c.id === kit.last)?.name ?? kit.last}: ${kit.error}` : kit.error}
          />
        </div>
      )}
      <div className="min-h-0 flex-1 overflow-auto px-5 pb-4">
        <DataTable
          caption={t("Strumenti")}
          groups
          stickyHead
          className="w-full border-separate border-spacing-0 text-[12px]"
          head={
            <>
              <th scope="col" className="w-8 px-2">
                <Checkbox
                  label={t("Scegli tutti quelli mostrati")}
                  checked={allOn}
                  disabled={running || free.length === 0}
                  className={someOn && !allOn ? "opacity-60" : ""}
                  onChange={(v) => {
                    for (const c of free) choose(c.id, v);
                  }}
                />
              </th>
              {header("name", t("Nome"))}
              {header("version", t("Versione"))}
              <th scope="col" className="px-2 py-[5px] text-left font-bold whitespace-nowrap">
                {t("Versioni su disco")}
              </th>
              {header("size", t("Dimensione"), "right")}
              {columns.map((id) =>
                id === activeTarget ? (
                  <Fragment key={id}>
                    {header("state", targetName(id), "center", "bg-vsc-focus/15", targetLabel(id, allProfiles))}
                  </Fragment>
                ) : (
                  <th
                    key={id}
                    scope="col"
                    title={targetLabel(id, allProfiles)}
                    className="px-2 py-[5px] text-center font-bold whitespace-nowrap"
                  >
                    {targetName(id)}
                  </th>
                ),
              )}
              <th scope="col" className="px-2 py-[5px] text-left font-bold whitespace-nowrap">
                {t("Progetti")}
              </th>
              <th scope="col" className="px-2 py-[5px] text-left font-bold whitespace-nowrap">
                {t("Richiede")}
              </th>
              <th scope="col" className="px-2 py-[5px] text-left font-bold whitespace-nowrap">
                {t("Azioni")}
              </th>
            </>
          }
        >
          {all.map((g) => {
            const items = rows.filter((c) => g.has(c));
            if (items.length === 0) return null;
            const total = list.filter((c) => g.has(c)).length;
            const open = !folded.has(g.id);
            const choosable = freeOf(items);
            const groupOn = choosable.length > 0 && choosable.every((c) => chosen.has(c.id));
            const Chevron = open ? codicon("chevron-down") : codicon("chevron-right");
            return (
              <tbody key={g.id} aria-label={g.title}>
                <tr>
                  <th scope="col" className="border-b border-vsc-border bg-vsc-sidebar px-2 pt-3 pb-1 align-top">
                    {choosable.length > 0 && (
                      <Checkbox
                        label={t("Scegli tutti: {0}", g.title)}
                        checked={groupOn}
                        disabled={running}
                        className={!groupOn && choosable.some((c) => chosen.has(c.id)) ? "opacity-60" : ""}
                        onChange={(v) => {
                          for (const c of choosable) choose(c.id, v);
                        }}
                      />
                    )}
                  </th>
                  <th
                    scope="colgroup"
                    colSpan={8 + columns.length - 1}
                    className="border-b border-vsc-border bg-vsc-sidebar px-2 pt-2 pb-1.5 text-left font-normal"
                  >
                    <button
                      type="button"
                      aria-expanded={open}
                      onClick={() => {
                        fold(g.id);
                      }}
                      className="flex items-center gap-1.5 text-[13px] font-semibold text-vsc-fg"
                    >
                      <Chevron size={16} aria-hidden="true" />
                      {g.title}
                      <span className="font-normal text-vsc-desc tabular-nums">
                        {items.length === total ? total : t("{0} di {1}", items.length, total)}
                      </span>
                    </button>
                    {open && <p className="mt-0.5 max-w-[60rem] pl-[22px] text-[12px] text-vsc-desc">{g.hint}</p>}
                  </th>
                </tr>
                {open && items.map(row)}
              </tbody>
            );
          })}
        </DataTable>
        {rows.length === 0 && <p className="py-3 text-vsc-desc">{t("Nessuno strumento corrisponde ai filtri.")}</p>}
      </div>
    </div>
  );
}
