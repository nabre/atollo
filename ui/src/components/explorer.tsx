// The Tools page, as the Projects page: the side bar shows the components
// as a tree, one branch per group (required, agents, languages, tools,
// databases, system), with their state, and the overviews of the kit; the editor opens each one in
// its own tab (./tool.tsx). Choosing and removing wait for "Apply", in the
// bar under the editor (./applybar.tsx).
import { useState, type ReactNode } from "react";
import {
  VscAdd,
  VscBook,
  VscClose,
  VscExtensions,
  VscReferences,
  VscRefresh,
  VscServerProcess,
  VscTable,
  VscWarning,
} from "react-icons/vsc";
import type { ComponentInfo, SetupSnapshot } from "../api";
import type { GuideTopicId } from "../guide";
import { t } from "../i18n";
import { ErrorLine } from "../common";
import { ComponentLogo } from "../logos";
import { ExtensionsMatrixSection } from "../extensions";
import { RelationsSection, RuntimeRows, useRelations } from "../relations";
import { ItemStatus, itemState } from "../setup";
import { composerOf, splitVersion, useKitVersions } from "../versions";
import {
  Dot,
  EditorBody,
  EditorTabs,
  IconButton,
  List,
  ListRow,
  Notice,
  PaneStack,
  SearchBox,
  SideBarTitle,
  ViewContainer,
  ViewPane,
  type useTabs,
} from "../workbench";
import { groups, matches } from "./model";
import { useComponents } from "./state";
import { ApplyBar } from "./applybar";
import { ToolPage } from "./tool";
import { ToolsTable } from "./table";

/** The overviews of the kit, opened as tabs like the tools. */
const OVERVIEWS = ["overview:table", "overview:extensions", "overview:relations", "overview:system"] as const;
type Overview = (typeof OVERVIEWS)[number];
const TABLE: Overview = "overview:table";

function overviewOf(id: string): Overview | null {
  return (OVERVIEWS as readonly string[]).includes(id) ? (id as Overview) : null;
}

const OVERVIEW_ICON: Record<Overview, ReactNode> = {
  "overview:table": <VscTable size={16} aria-hidden="true" />,
  "overview:extensions": <VscExtensions size={16} aria-hidden="true" />,
  "overview:relations": <VscReferences size={16} aria-hidden="true" />,
  "overview:system": <VscServerProcess size={16} aria-hidden="true" />,
};

function overviewTitle(id: Overview): string {
  return {
    "overview:table": t("Tutti gli strumenti"),
    "overview:extensions": t("Estensioni dell'editor"),
    "overview:relations": t("Relazioni"),
    "overview:system": t("Requisiti del PC"),
  }[id];
}

export function ToolsPage({
  setup,
  tabs,
  onGuide,
}: {
  setup: SetupSnapshot | null;
  tabs: ReturnType<typeof useTabs>;
  onGuide?: (topic: GuideTopicId) => void;
}) {
  const state = useComponents(setup, false);
  const { list, chosen, choose, error, synced, neededBy, projects, load } = state;
  const kit = useKitVersions(setup, true, state.target);
  const relations = useRelations(setup, list, true);
  const links = relations.matrix?.links ?? [];
  const [search, setSearch] = useState("");
  const all = groups();
  const [closed, setClosed] = useState<Set<string>>(new Set());
  const running = setup?.running ?? false;

  const toggle = (id: string) => {
    setClosed((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const byId = (id: string) => list?.find((c) => c.id === id);
  // The table is the first tab, always open: shown when no other is.
  const active = tabs.active ?? TABLE;
  const page = byId(active);
  const overview = overviewOf(active);
  const open = (id: string) => {
    if (id === TABLE) tabs.setActive(TABLE);
    else tabs.show(id);
  };

  const shown = list?.filter((c) => matches(c, search)) ?? [];
  const installedAll = shown.filter((c) => c.installed || Boolean(c.bootstrapStaged)).length;
  // A tool's row, under its group's branch of the tree.
  const row = (c: ComponentInfo) => {
    const on = chosen.has(c.id);
    const ready = c.installed || Boolean(c.bootstrapStaged);
    const pending = on !== (c.chosen || c.locked);
    const item = itemState(setup, c.id, c.name);
    const missing = (neededBy.get(c.id) ?? []).length > 0 && !on;
    const version = composerOf(c.version) ? splitVersion(c.version).main : c.version;
    return (
      <ListRow
        key={c.id}
        depth={1}
        icon={<ComponentLogo id={c.id} size={16} />}
        label={c.name}
        description={ready ? (version ?? undefined) : undefined}
        title={c.desc}
        selected={active === c.id}
        onSelect={() => {
          tabs.show(c.id);
        }}
        actions={
          c.locked ? undefined : on ? (
            <IconButton
              icon={VscClose}
              label={ready ? t("Rimuovi {0}", c.name) : t("Non installare {0}", c.name)}
              disabled={running}
              onClick={() => {
                choose(c.id, false);
              }}
            />
          ) : (
            <IconButton
              icon={VscAdd}
              label={t("Installa {0}", c.name)}
              disabled={running || !c.available}
              onClick={() => {
                choose(c.id, true);
              }}
            />
          )
        }
        decorations={
          item ? (
            <ItemStatus state={item} />
          ) : (
            <>
              {missing && (
                <VscWarning size={14} className="text-vsc-warning" aria-label={t("Serve a progetti e non è scelto")} />
              )}
              {pending ? (
                <Dot tone="warn" label={on ? t("da installare") : t("da rimuovere")} />
              ) : c.installed ? (
                <Dot tone="ok" label={t("installato")} />
              ) : c.bootstrapStaged ? (
                <Dot tone="ok" label={t("pre-scaricato")} />
              ) : null}
            </>
          )
        }
      />
    );
  };

  const side = (
    <>
      <SideBarTitle
        title={t("Strumenti")}
        actions={
          <>
            <IconButton
              icon={VscRefresh}
              label={t("Aggiorna")}
              onClick={() => {
                load();
                relations.load();
              }}
            />
            {onGuide && (
              <IconButton
                icon={VscBook}
                label={t("Apri la guida di questa pagina")}
                onClick={() => {
                  onGuide("sources-table");
                }}
              />
            )}
          </>
        }
      />
      <div className="shrink-0 px-3 pt-1 pb-1.5">
        <SearchBox value={search} onChange={setSearch} placeholder={t("Cerca negli strumenti")} />
      </div>
      <div className="px-3 empty:hidden">
        <ErrorLine error={error} />
      </div>
      {list === null ? (
        <p className="px-5 py-1 text-vsc-desc">{t("Caricamento…")}</p>
      ) : (
        <PaneStack>
          <ViewPane
            first
            fill
            title={t("Componenti")}
            badge={`${String(installedAll)}/${String(shown.length)}`}
            expanded={!closed.has("components")}
            onToggle={() => {
              toggle("components");
            }}
          >
            <List label={t("Componenti")}>
              {all.map((g) => {
                const items = shown.filter((c) => g.has(c));
                if (items.length === 0) return null;
                const installed = items.filter((c) => c.installed || Boolean(c.bootstrapStaged)).length;
                const open = !closed.has(g.id) || search !== "";
                return (
                  <ListRow
                    key={g.id}
                    twistie={open ? "open" : "closed"}
                    label={g.title}
                    title={g.hint}
                    description={`${String(installed)}/${String(items.length)}`}
                    onSelect={() => {
                      toggle(g.id);
                    }}
                  >
                    <List label={g.title}>{items.map(row)}</List>
                  </ListRow>
                );
              })}
            </List>
          </ViewPane>
          <ViewPane
            title={t("Panoramica")}
            expanded={!closed.has("overview")}
            onToggle={() => {
              toggle("overview");
            }}
          >
            <List label={t("Panoramica")}>
              {OVERVIEWS.map((id) => (
                <ListRow
                  key={id}
                  icon={OVERVIEW_ICON[id]}
                  label={overviewTitle(id)}
                  selected={active === id}
                  onSelect={() => {
                    open(id);
                  }}
                />
              ))}
            </List>
          </ViewPane>
        </PaneStack>
      )}
    </>
  );

  return (
    <ViewContainer id="components" side={side}>
      <div className="flex min-h-0 flex-1 flex-col">
        <EditorTabs
          tabs={[
            { id: TABLE, label: overviewTitle(TABLE), icon: OVERVIEW_ICON[TABLE], pinned: true },
            ...tabs.open
              .filter((id) => id !== TABLE)
              .map((id) => {
                const o = overviewOf(id);
                return o
                  ? { id, label: overviewTitle(o), icon: OVERVIEW_ICON[o] }
                  : { id, label: byId(id)?.name ?? id, icon: <ComponentLogo id={id} size={16} /> };
              }),
          ]}
          active={active}
          onSelect={open}
          onClose={tabs.close}
        />
        {synced.length > 0 && (
          <div className="px-5 pt-3">
            <Notice tone="info">
              {t("Non più nella cartella del kit, tolti dalla scelta: {0}.", synced.join(", "))}
            </Notice>
          </div>
        )}
        {page ? (
          <ToolPage
            key={page.id}
            c={page}
            state={state}
            kit={kit}
            setup={setup}
            links={links}
            onOpen={(id) => {
              tabs.show(id);
            }}
          />
        ) : overview === "overview:extensions" ? (
          <EditorBody narrow>
            <ExtensionsMatrixSection projects={projects} components={list ?? []} />
          </EditorBody>
        ) : overview === "overview:relations" ? (
          <EditorBody narrow>
            {relations.matrix ? (
              <RelationsSection matrix={relations.matrix} setup={setup} onReload={relations.load} />
            ) : (
              <p className="text-vsc-desc">{t("Caricamento…")}</p>
            )}
          </EditorBody>
        ) : overview === "overview:system" ? (
          <EditorBody narrow>
            <div data-setting>
              <RuntimeRows runtimes={relations.matrix?.runtimes ?? []} />
            </div>
          </EditorBody>
        ) : (
          // The table of every tool: its own tab, and the page when no tab is open.
          list && (
            <ToolsTable
              state={state}
              kit={kit}
              setup={setup}
              onOpen={(id) => {
                tabs.show(id);
              }}
            />
          )
        )}
        <ApplyBar state={state} kit={kit} setup={setup} />
      </div>
    </ViewContainer>
  );
}
