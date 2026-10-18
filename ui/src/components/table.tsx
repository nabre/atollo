// Every tool in one table, the default tab of the Tools page: filters by
// group and state, sortable columns, the choice of each row (or of all
// those shown) applied with "Apply" as on the tool's own page, and the
// actions done at once: install (at a chosen version), add a version, update.
import { useMemo, useState } from "react";
import { VscArrowDown, VscArrowUp, VscCheck, VscCloudDownload, VscSync, VscWarning } from "react-icons/vsc";
import {
  TARGETS,
  componentsInstall,
  setupStart,
  versionsAdd,
  versionsInstall,
  type ComponentInfo,
  type SetupSnapshot,
} from "../api";
import { t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { ComponentLogo } from "../logos";
import { ItemStatus, itemState } from "../setup";
import { Tag } from "../tags";
import { AddLine, composerOf, RefreshLines, splitVersion, type KitVersions } from "../versions";
import { Checkbox, DataTable, IconButton, Link, SearchBox, Select } from "../workbench";
import { groups, linesOf, matches, sizeText } from "./model";
import type { ComponentsState } from "./state";

type Sort = "name" | "group" | "version" | "size" | "state";
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
  if (c.noArm64 || c.unavailableReason !== null || c.needsWindows11) return null;
  if (!ready) {
    if (!c.available) return null;
    return (
      <span className="flex items-center gap-1">
        <IconButton
          icon={VscCloudDownload}
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
        icon={VscSync}
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

/** The order of the states in the "State" column. */
function stateRank(c: ComponentInfo, on: boolean): number {
  if (on !== (c.chosen || c.locked)) return 0;
  if (c.installed || c.bootstrapStaged) return 1;
  return on ? 2 : 3;
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
  const { chosen, choose, neededBy, target, setTarget, checks, checking, verifyTarget } = state;
  const activeTarget = target ?? setup?.target ?? "win32-x64";
  const running = setup?.running ?? false;
  const all = groups();
  const [search, setSearch] = useState("");
  const [group, setGroup] = useState("all");
  const [show, setShow] = useState<Show>("all");
  const [sort, setSort] = useState<{ by: Sort; up: boolean }>({ by: "group", up: true });

  const groupOf = (c: ComponentInfo) => all.findIndex((g) => g.has(c));
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
          return !ready;
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
        case "group":
          return groupOf(c);
        case "version":
          return c.version ?? "";
        case "size":
          return c.mb;
        case "state":
          return stateRank(c, chosen.has(c.id));
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
  }, [list, chosen, search, group, show, sort, neededBy]);

  const free = rows.filter((c) => !c.locked && (c.available || chosen.has(c.id)));
  const allOn = free.length > 0 && free.every((c) => chosen.has(c.id));
  const someOn = free.some((c) => chosen.has(c.id));
  const header = (by: Sort, label: string, right = false) => (
    <th scope="col" aria-sort={sort.by === by ? (sort.up ? "ascending" : "descending") : undefined} className="p-0">
      <button
        type="button"
        onClick={() => {
          setSort((s) => ({ by, up: s.by === by ? !s.up : true }));
        }}
        className={`flex w-full items-center gap-1 px-2 py-[5px] leading-5 font-bold whitespace-nowrap hover:bg-vsc-list-hover ${
          right ? "justify-end" : ""
        }`}
      >
        {label}
        {sort.by === by &&
          (sort.up ? <VscArrowUp size={12} aria-hidden="true" /> : <VscArrowDown size={12} aria-hidden="true" />)}
      </button>
    </th>
  );

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
          options={TARGETS.map((id) => ({ value: id, label: id }))}
        />
        <Select
          label={t("Gruppo")}
          value={group}
          onChange={setGroup}
          options={[{ value: "all", label: t("Tutti i gruppi") }, ...all.map((g) => ({ value: g.id, label: g.title }))]}
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
          icon={VscCheck}
          label={t("Verifica")}
          busy={checking}
          disabled={running || checking}
          onClick={verifyTarget}
        />
        <span className="ml-auto text-[12px] text-vsc-desc" role="status">
          {t("{0} di {1}", rows.length, list.length)}
        </span>
      </div>
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
              {header("group", t("Gruppo"))}
              {header("version", t("Versione"))}
              <th scope="col" className="px-2 py-[5px] text-left font-bold whitespace-nowrap">
                {t("Versioni su disco")}
              </th>
              {header("size", t("Dimensione"), true)}
              {header("state", t("Stato"))}
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
          {rows.map((c) => {
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
                    disabled={c.locked || running || c.noArm64 || c.unavailableReason !== null || (!c.available && !on)}
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
                <td className={`${cell} whitespace-nowrap text-vsc-desc`}>{all[groupOf(c)]?.title ?? c.group}</td>
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
                <td className={`${cell} whitespace-nowrap`}>
                  <span className="flex items-center gap-1.5">
                    {item ? (
                      <ItemStatus state={item} />
                    ) : pending ? (
                      <span className="text-vsc-settings-modified">{on ? t("da installare") : t("da rimuovere")}</span>
                    ) : c.installed ? (
                      <span className="text-vsc-success">{t("installato")}</span>
                    ) : c.bootstrapStaged ? (
                      <span
                        className="text-vsc-success"
                        title={t("Bootstrap pre-scaricato; l'installazione termina al primo avvio su quel sistema")}
                      >
                        {t("pre-scaricato")}
                      </span>
                    ) : c.noArm64 ? (
                      <span className="text-vsc-desc">{t("non disponibile su PC ARM64")}</span>
                    ) : c.unavailableReason !== null ? (
                      <span className="text-vsc-desc">{c.unavailableReason}</span>
                    ) : (
                      <span className="text-vsc-desc">{t("non installato")}</span>
                    )}
                    {check && (
                      <Tag tone={check.ok ? "focus" : "warning"}>
                        {check.ok ? "√" : "x"} {check.answer}
                      </Tag>
                    )}
                  </span>
                </td>
                <td className={`${cell} max-w-[16rem] truncate`} title={needed.join(", ") || undefined}>
                  {needed.length > 0 && !on && (
                    <VscWarning
                      size={12}
                      className="mr-1 inline text-vsc-warning"
                      aria-label={t("Serve a progetti e non è scelto")}
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
          })}
        </DataTable>
        {rows.length === 0 && <p className="py-3 text-vsc-desc">{t("Nessuno strumento corrisponde ai filtri.")}</p>}
      </div>
    </div>
  );
}
