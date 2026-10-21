// The first-start page: which agents, languages and tools the kit gets,
// before anything optional is installed. Same rules as the Tools page
// (./explorer.tsx): required components always on, dependencies follow both
// ways (Apache with PHP, Kotlin with Java). First the choices for the whole
// kit (language, updates, network: FirstRunSettings, D-137), since nothing
// is downloaded before «Installa».
import { useState } from "react";
import { BrandSymbol } from "../brand";
import type { SetupSnapshot } from "../api";
import { rich, t } from "../i18n";
import { ErrorLine } from "../common";
import { Codicon, SettingsEditor, SettingsGroup, SingleEditor } from "../workbench";
import { useKitVersions } from "../versions";
import { RuntimeRows, useRelations } from "../relations";
import { groups, matches } from "./model";
import { ComponentRow } from "./rows";
import { useComponents } from "./state";
import { ApplyBar } from "./applybar";
import { FirstRunSettings } from "../settings/firstrun";

export function ComponentsPage({
  setup,
  onDone,
  platform,
  onSettings,
}: {
  setup: SetupSnapshot | null;
  onDone?: () => void;
  /** This PC's system (AppInfo), for the examples of the network fields. */
  platform?: string | undefined;
  /** A first-start choice was saved (the language changes the page). */
  onSettings?: () => void;
}) {
  const state = useComponents(setup, true, onDone);
  const { list, chosen, choose, error, neededBy } = state;
  const [search, setSearch] = useState("");
  const kit = useKitVersions(setup, false);
  const relations = useRelations(setup, list, true);
  const links = relations.matrix?.links ?? [];

  const title = t("Benvenuto");
  const icon = <Codicon name="extensions" size={16} />;
  if (error) {
    return (
      <SingleEditor title={title} icon={icon}>
        <div className="p-5">
          <ErrorLine error={error} />
        </div>
      </SingleEditor>
    );
  }
  if (!list) {
    return (
      <SingleEditor title={title} icon={icon}>
        <p className="p-5 text-vsc-desc">{t("Caricamento…")}</p>
      </SingleEditor>
    );
  }

  const searching = search.trim() !== "";
  const all = groups();

  return (
    <SingleEditor title={title} icon={icon}>
      <div className="bg-hero shrink-0 space-y-1 border-b border-vsc-border px-5 py-4 pr-[30%]">
        <BrandSymbol size={40} />
        <h2 className="font-brand text-[18px] font-semibold tracking-tight">{t("Benvenuto in Atollo")}</h2>
        <p>
          {rich(
            t(
              "Scegli agenti e linguaggi: verranno scaricati dalle fonti ufficiali, verificati e messi nella cartella {0} del kit. Niente installazione, niente diritti di amministratore.",
            ),
            <span className="font-mono">data</span>,
          )}
        </p>
        <p className="text-vsc-desc">
          {rich(
            t("Le cartelle in {0} sono attendibili per Claude e Codex: apri lì solo codice di cui ti fidi."),
            <span className="font-mono">projects</span>,
          )}
        </p>
      </div>
      <SettingsEditor
        toc={[
          ...(searching
            ? []
            : [
                { id: "group-start", label: t("Prima di iniziare") },
                { id: "s-network", label: t("Rete") },
              ]),
          ...all
            .filter((g) => list.some((c) => g.has(c) && matches(c, search)))
            .map((g) => ({ id: `group-${g.id}`, label: g.title })),
        ]}
        search={search}
        onSearch={setSearch}
        placeholder={t("Cerca negli strumenti")}
      >
        {!searching && <FirstRunSettings platform={platform} onSaved={onSettings} />}
        {all.map((g) => {
          const items = list.filter((c) => g.has(c) && matches(c, search));
          if (items.length === 0 && searching) return null;
          return (
            <SettingsGroup key={g.id} id={`group-${g.id}`} title={g.title} description={g.hint}>
              {items.map((c) => (
                <ComponentRow
                  key={c.id}
                  c={c}
                  list={list}
                  chosen={chosen}
                  forProjects={neededBy.get(c.id) ?? []}
                  kit={kit}
                  setup={setup}
                  links={links}
                  onChange={(v) => {
                    choose(c.id, v);
                  }}
                />
              ))}
              {/* What the PC must provide, with the required components. */}
              {g.id === "system" && <RuntimeRows runtimes={relations.matrix?.runtimes ?? []} />}
            </SettingsGroup>
          );
        })}
      </SettingsEditor>
      <ApplyBar state={state} kit={kit} setup={setup} firstRun />
    </SingleEditor>
  );
}
