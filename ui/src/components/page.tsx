// "Linguaggi e strumenti": which agents, languages and tools the kit has.
// Same rules as the page of nabre/dev-kit: required components always on,
// dependencies follow both ways (Apache with PHP, Kotlin with Java), a
// component turned off is removed at the next installation. Also used as the
// first-start page, before anything optional is installed.
import { useCallback, useEffect, useMemo, useState } from "react";
import { BrandSymbol } from "../brand";
import { VscCheck, VscCircleSlash, VscCloudDownload, VscExtensions, VscRefresh, VscTrash } from "react-icons/vsc";
import {
  componentsApply,
  componentsList,
  componentsSync,
  setupCancel,
  projectsList,
  type ComponentInfo,
  type ProjectInfo,
  type SetupSnapshot,
} from "../api";
import { rich, t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { Notice, SettingsEditor, SettingsGroup, SingleEditor } from "../workbench";
import { setupStatusText } from "../setup";
import { ExtensionsMatrixSection } from "../extensions";
import { useKitVersions } from "../versions";
import { RelationsSection, RuntimeRows, useRelations } from "../relations";
import { groups, linesOf, sizeText, toggle } from "./model";
import { ComponentRow } from "./rows";

export function ComponentsPage({
  setup,
  firstRun = false,
  onDone,
}: {
  setup: SetupSnapshot | null;
  firstRun?: boolean;
  onDone?: () => void;
}) {
  const [list, setList] = useState<ComponentInfo[] | null>(null);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [search, setSearch] = useState("");
  const [cancelError, setCancelError] = useState<string | null>(null);
  const running = setup?.running ?? false;
  const kit = useKitVersions(setup, !firstRun);
  const relations = useRelations(setup, list, true);
  const links = relations.matrix?.links ?? [];

  const load = useCallback(() => {
    // Only for the highlight: the page works without the projects.
    projectsList().then(setProjects, () => {
      setProjects([]);
    });
    componentsList().then(
      (l) => {
        setList(l);
        setChosen(new Set(l.filter((c) => c.chosen || c.locked).map((c) => c.id)));
      },
      (e: unknown) => {
        setError(String(e));
      },
    );
  }, []);
  // Opening the page: first what was deleted by hand from the kit's
  // folder, then the list as it is on disk.
  const [synced, setSynced] = useState<string[]>([]);
  useEffect(() => {
    if (firstRun) {
      load();
      return;
    }
    componentsSync().then(
      (off) => {
        setSynced(off);
        load();
      },
      () => {
        load();
      },
    );
  }, [firstRun, load]);
  // Reload as each component of an installation ends, and when it ends:
  // versions and "installed" change.
  const finished = setup?.steps.filter((s) => s.outcome !== null).length ?? 0;
  const ended = setup !== null && !setup.running && setup.steps.length > 0;
  useEffect(() => {
    if (finished > 0 || ended) load();
  }, [finished, ended, load]);

  const plan = useMemo(() => {
    const items = list ?? [];
    const install = items.filter((c) => chosen.has(c.id) && !c.installed && c.available);
    const remove = items.filter((c) => !chosen.has(c.id) && c.present);
    return { install, remove, mb: install.reduce((n, c) => n + c.mb, 0) };
  }, [list, chosen]);

  /** Component id → names of the projects that need it. */
  const neededBy = useMemo(() => {
    const map = new Map<string, string[]>();
    for (const p of projects) {
      for (const id of p.requires) map.set(id, [...(map.get(id) ?? []), p.name]);
    }
    return map;
  }, [projects]);

  const title = firstRun ? t("Benvenuto") : t("Strumenti");
  const icon = <VscExtensions size={16} aria-hidden="true" />;
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

  const apply = (updateAll: boolean) => {
    setConfirm(false);
    componentsApply([...chosen], updateAll).then(
      () => onDone?.(),
      (e: unknown) => {
        setError(String(e));
      },
    );
  };
  const changed = plan.install.length > 0 || plan.remove.length > 0;
  const words = search.toLowerCase().split(/\s+/).filter(Boolean);
  const visible = (c: ComponentInfo) =>
    words.length === 0 || words.every((w) => `${c.name} ${c.desc} ${c.id}`.toLowerCase().includes(w));
  const all = groups();

  return (
    <SingleEditor title={title} icon={icon}>
      {firstRun && (
        <div className="bg-hero shrink-0 space-y-1 border-b border-vsc-border px-5 py-4 pr-[30%]">
          <BrandSymbol size={40} />
          <h2 className="font-brand text-[18px] font-semibold tracking-tight">{t("Benvenuto in NEXKORA")}</h2>
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
      )}
      <SettingsEditor
        toc={[
          ...all
            .filter((g) => list.some((c) => g.has(c) && visible(c)))
            .map((g) => ({ id: `group-${g.id}`, label: g.title })),

          ...(firstRun
            ? []
            : [
                { id: "ext-matrix", label: t("Estensioni dell'editor") },
                { id: "relations", label: t("Relazioni") },
              ]),
        ]}
        search={search}
        onSearch={setSearch}
        placeholder={t("Cerca negli strumenti")}
      >
        {synced.length > 0 && (
          <div className="max-w-[880px] px-[14px] pt-3">
            <Notice tone="info">
              {t("Non più nella cartella del kit, tolti dalla scelta: {0}.", synced.join(", "))}
            </Notice>
          </div>
        )}
        {all.map((g) => {
          const items = list.filter((c) => g.has(c) && visible(c));
          if (items.length === 0 && words.length > 0) return null;
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
                    setChosen(toggle(list, chosen, c.id, v));
                  }}
                />
              ))}
              {/* What the PC must provide, with the required components. */}
              {g.id === "required" && <RuntimeRows runtimes={relations.matrix?.runtimes ?? []} />}
            </SettingsGroup>
          );
        })}
        {!firstRun && words.length === 0 && (
          <div className="max-w-[880px] px-[14px]" data-setting>
            <ExtensionsMatrixSection projects={projects} components={list} />
            {relations.matrix && <RelationsSection matrix={relations.matrix} setup={setup} onReload={relations.load} />}
          </div>
        )}
      </SettingsEditor>
      {/* As the editor's notification of a pending change: what Apply does. */}
      <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-vsc-border px-5 py-2">
        {confirm ? (
          <>
            <span role="status" className="min-w-0 flex-1">
              {plan.install.length > 0 &&
                t("Da installare: {0} (circa {1}).", plan.install.map((c) => c.name).join(", "), sizeText(plan.mb)) +
                  " "}
              {plan.remove.length > 0 &&
                t(
                  "Da rimuovere: {0}.",
                  plan.remove
                    .map((c) => {
                      const n = linesOf(kit.byId.get(c.id)).length;
                      return n > 1 ? `${c.name} (${t("{0} versioni", n)})` : c.name;
                    })
                    .join(", "),
                )}
            </span>
            <ActionButton
              icon={VscCheck}
              label={t("Conferma")}
              primary
              onClick={() => {
                apply(false);
              }}
            />
            <ActionButton
              icon={VscRefresh}
              label={t("Indietro")}
              onClick={() => {
                setConfirm(false);
              }}
            />
          </>
        ) : (
          <>
            <span
              role="status"
              className={`min-w-0 flex-1 ${setup?.error || cancelError ? "text-vsc-error" : "text-vsc-desc"}`}
            >
              {running
                ? setupStatusText(setup)
                : (setup?.error ?? cancelError)
                  ? (setup?.error ?? cancelError)
                  : changed
                    ? t(
                        "{0} da installare ({1}), {2} da rimuovere",
                        plan.install.length,
                        sizeText(plan.mb),
                        plan.remove.length,
                      )
                    : t("Nessuna modifica")}
            </span>
            {running && (
              <ActionButton
                icon={VscCircleSlash}
                label={setup?.cancelling ? t("Annullamento…") : t("Annulla")}
                disabled={setup?.cancelling ?? false}
                onClick={() => {
                  setCancelError(null);
                  setupCancel().catch((e: unknown) => {
                    setCancelError(String(e));
                  });
                }}
              />
            )}
            {!firstRun && (
              <ActionButton
                icon={VscRefresh}
                label={t("Aggiorna tutto")}
                disabled={running}
                title={
                  running
                    ? t("Un'installazione è già in corso")
                    : t("Scarica le versioni nuove di tutto ciò che è installato")
                }
                onClick={() => {
                  apply(true);
                }}
              />
            )}
            <ActionButton
              icon={
                firstRun ? VscCloudDownload : plan.remove.length > 0 && plan.install.length === 0 ? VscTrash : VscCheck
              }
              label={firstRun ? t("Installa") : t("Applica")}
              primary
              disabled={running || (!changed && !firstRun)}
              title={running ? t("Un'installazione è già in corso") : undefined}
              onClick={() => {
                if (firstRun) apply(false);
                else setConfirm(true);
              }}
            />
          </>
        )}
      </div>
    </SingleEditor>
  );
}
