// "Linguaggi e strumenti": which agents, languages and tools the kit has.
// Same rules as the page of nabre/atollo: required components always on,
// dependencies follow both ways (Apache with PHP, Kotlin with Java), a
// component turned off is removed at the next installation. Also used as the
// first-start page, before anything optional is installed.
import { Fragment, useCallback, useEffect, useMemo, useState } from "react";
import { BrandSymbol } from "./brand";
import {
  VscCheck,
  VscCircleSlash,
  VscCloudDownload,
  VscExtensions,
  VscFolderLibrary,
  VscReferences,
  VscRefresh,
  VscTrash,
  VscTypeHierarchySub,
  VscWarning,
} from "react-icons/vsc";
import {
  componentsApply,
  componentsList,
  setupCancel,
  projectsList,
  type ComponentInfo,
  type Group,
  type ProjectInfo,
  type SetupSnapshot,
  type VersionedTool,
} from "./api";
import { locale, rich, t } from "./i18n";
import { ActionButton, ErrorLine } from "./pages";
import { Checkbox, SettingsEditor, SettingsGroup, SingleEditor } from "./workbench";

import { ItemBar, ItemStatus, itemState, setupStatusText, type ItemState } from "./setup";
import { ExtensionsMatrixSection } from "./extensions";
import { composerOf, splitVersion, ToolVersions, useKitVersions } from "./versions";
import { ComponentLogo } from "./logos";
import { LinkProblems, outsideOf, RelationsSection, RuntimeRows, useRelations } from "./relations";
import { Badge, Tag } from "./tags";

/** The sections of the page. The catalog's "languages" are split: those
 * with one folder per minor version, side by side, and the tools that are
 * always at their latest version. */
function groups(): { id: string; title: string; hint: string; has: (c: ComponentInfo) => boolean }[] {
  const of = (group: Group) => (c: ComponentInfo) => c.group === group;
  return [
    {
      id: "required",
      title: t("Obbligatori"),
      hint: t("Sempre installati: l'editor, la shell del kit e GitHub CLI."),
      // The editor (part of the app) is listed with the required ones.
      has: (c) => c.group === "required" || c.group === "app",
    },
    {
      id: "agents",
      title: t("Agenti"),
      hint: t("Facoltativi: il kit funziona anche senza. Sempre all'ultima versione."),
      has: of("agents"),
    },
    {
      id: "languages",
      title: t("Linguaggi"),
      hint: t(
        "Più versioni una accanto all'altra, una per versione minore: «Aggiorna tutto» porta ognuna all'ultima patch, una versione minore nuova si aggiunge senza togliere le altre. Il kit usa la più recente; un progetto tiene quella che c'era quando è stato creato (o quella che chiede il suo codice) e può cambiarla nella sua pagina.",
      ),
      has: (c) => c.group === "languages" && c.versioned,
    },
    {
      id: "tools",
      title: t("Strumenti"),
      hint: t(
        "Una sola versione, sempre l'ultima: «Aggiorna tutto» la sostituisce. Quelli che servono a un agente o a un progetto si installano da soli.",
      ),
      has: (c) => c.group === "languages" && !c.versioned,
    },
    {
      id: "databases",
      title: t("Database"),
      hint: t("Server avviati dalla pagina Database, solo su questo PC (127.0.0.1); i dati restano in data\\db."),
      has: of("databases"),
    },
    {
      id: "os",
      title: t("Legati a un sistema operativo"),
      hint: t("Funzionano solo sul sistema per cui sono fatti, qui Windows."),
      has: of("os"),
    },
  ];
}

/** Turns `id` on or off following the dependencies both ways. */
export function toggle(list: ComponentInfo[], chosen: Set<string>, id: string, on: boolean): Set<string> {
  const next = new Set(chosen);
  const visit = (cid: string, value: boolean) => {
    const c = list.find((x) => x.id === cid);
    if (!c || c.locked) return;
    if (value === next.has(cid)) return;
    if (value) {
      next.add(cid);
      c.needs.forEach((n) => {
        visit(n, true);
      });
    } else {
      next.delete(cid);
      list
        .filter((x) => x.needs.includes(cid))
        .forEach((x) => {
          visit(x.id, false);
        });
    }
  };
  visit(id, on);
  return next;
}

/** The projects that need `c`: blue when chosen, orange with a warning when not. */
function ProjectsChip({ names, on }: { names: string[]; on: boolean }) {
  if (names.length === 0) return null;
  const list = names.join(", ");
  return on ? (
    <Tag
      tone="focus"
      className="ml-2"
      title={t("Serve ai progetti: {0}.", list)}
      icon={<VscFolderLibrary size={11} aria-hidden="true" />}
    >
      <span className="sr-only">{t("serve ai progetti")} </span>
      {list}
    </Tag>
  ) : (
    <Tag
      tone="warning"
      className="ml-2"
      title={t("Serve ai progetti: {0}. Non è scelto: sceglilo perché funzionino.", list)}
      icon={<VscWarning size={11} aria-hidden="true" />}
    >
      <span className="sr-only">{t("attenzione: non scelto, serve ai progetti")} </span>
      {list}
    </Tag>
  );
}

/** What `c` needs (installed with it) and, when chosen, what needs `c`. */
function Dependencies({ c, list, chosen }: { c: ComponentInfo; list: ComponentInfo[]; chosen: Set<string> }) {
  const name = (id: string) => list.find((x) => x.id === id)?.name ?? id;
  const usedBy = list.filter((x) => chosen.has(x.id) && x.needs.includes(c.id)).map((x) => x.name);
  return (
    <>
      {c.needs.map((n) => (
        <Tag
          key={n}
          className="ml-2"
          title={t("Richiede {0}: si installa insieme.", name(n))}
          icon={<VscTypeHierarchySub size={11} aria-hidden="true" />}
        >
          <span className="sr-only">{t("richiede")} </span>
          {name(n)}
        </Tag>
      ))}
      {usedBy.length > 0 && (
        <Tag
          dashed
          className="ml-2"
          title={t("Serve a {0}: resta installato finché serve.", usedBy.join(", "))}
          icon={<VscReferences size={11} aria-hidden="true" />}
        >
          {t("per {0}", usedBy.join(", "))}
        </Tag>
      )}
    </>
  );
}

/** The versions on disk of a versioned tool: the default, then the saved. */
function linesOf(tool: VersionedTool | undefined): string[] {
  if (!tool) return [];
  return [...(tool.default ? [tool.default.line] : []), ...tool.saved.map((s) => s.line)];
}

/** Composer: a tool of its own with its version, but one per version of
 * PHP and compatible with it, installed and updated with it; its checkbox
 * follows PHP's. */
function ComposerRow({
  php,
  tool,
  on,
  state,
}: {
  php: ComponentInfo;
  tool: VersionedTool | undefined;
  on: boolean;
  state: ItemState | null;
}) {
  const kitLine = tool?.preferred ?? tool?.default?.line;
  const all = [...(tool?.default ? [tool.default] : []), ...(tool?.saved ?? [])]
    .map((l) => ({ line: l.line, composer: composerOf(l.version) }))
    .filter((l): l is { line: string; composer: string } => l.composer !== null);
  const kit = all.find((l) => l.line === kitLine) ?? all[0];
  return (
    <div
      data-setting
      className="max-w-[880px] border-l-2 border-l-transparent py-2 pr-[14px] pl-3 hover:bg-vsc-list-hover/50"
    >
      <div className="flex items-center gap-3">
        <Checkbox id="comp-composer" checked={on} disabled onChange={() => undefined} />
        <span className="flex w-5 shrink-0 justify-center">
          <ComponentLogo id="composer" size={20} />
        </span>
        <label
          htmlFor="comp-composer"
          className="min-w-0 flex-1 leading-[18px]"
          title={t("Si installa, si aggiorna e si toglie con PHP.")}
        >
          <span className="font-semibold text-vsc-settings-header">Composer</span>
          <span className="ml-2 text-vsc-desc">
            {t("pacchetti PHP, uno per ogni versione di PHP e compatibile con quella")}
          </span>
          <Tag dashed className="ml-2" icon={<VscReferences size={11} aria-hidden="true" />}>
            {t("con {0}", php.name)}
          </Tag>
        </label>
        <span className="w-16 shrink-0" />
        <span className="w-40 shrink-0 truncate text-right text-[12px]">
          {state ? (
            <ItemStatus state={state} />
          ) : all.length > 1 ? null /* its versions are in the list below */ : kit ? (
            <span className="text-vsc-success">
              <VscCheck className="mr-1 inline" size={12} aria-hidden="true" />
              {kit.composer}
            </span>
          ) : (
            <span className="text-vsc-desc">{t("non installato")}</span>
          )}
        </span>
      </div>
      {all.length > 1 && (
        <ul className="mt-1.5 space-y-0.5 pl-[62px]" aria-label={t("Versioni di Composer")}>
          {all.map((l) => (
            <li key={l.line} className="flex min-h-[22px] items-center gap-2 font-mono text-[12px]">
              <span className="text-vsc-desc">{t("con {0} {1}", php.name, l.line)}</span>
              <span>{l.composer}</span>
              {l.line === kitLine && <Badge>{t("in uso")}</Badge>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function sizeText(mb: number): string {
  if (mb < 1000) return `${String(mb)} MB`;
  const gb = new Intl.NumberFormat(locale(), { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(mb / 1000);
  return `${gb} GB`;
}

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
  useEffect(load, [load]);
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
        {all.map((g) => {
          const items = list.filter((c) => g.has(c) && visible(c));
          if (items.length === 0 && words.length > 0) return null;
          return (
            <SettingsGroup key={g.id} id={`group-${g.id}`} title={g.title} description={g.hint}>
              {items.map((c) => {
                const required = c.locked;
                const on = chosen.has(c.id);
                const inputId = `comp-${c.id}`;
                const forProjects = neededBy.get(c.id) ?? [];
                const unchosen = forProjects.length > 0 && !on;
                // Versions only of what is installed and stays.
                const versioned = kit.byId.get(c.id);
                const versions = versioned?.installed && on ? versioned : null;
                // Its step in the preparation: waiting, downloading, failed.
                const state = itemState(setup, c.id, c.name);
                // A left bar marks what a project needs: emerald, orange when not chosen.
                const mark = unchosen
                  ? "border-l-vsc-orange bg-vsc-orange/10"
                  : forProjects.length > 0
                    ? "border-l-vsc-focus"
                    : on !== (c.chosen || c.locked)
                      ? "border-l-vsc-settings-modified"
                      : "border-l-transparent";
                // PHP's version without its Composer, which has its row.
                const shown = composerOf(c.version) ? splitVersion(c.version).main : c.version;
                return (
                  <Fragment key={c.id}>
                    <div
                      data-setting
                      className={`max-w-[880px] border-l-2 py-2 pr-[14px] pl-3 hover:bg-vsc-list-hover/50 ${mark}`}
                    >
                      <div className="flex items-center gap-3">
                        <Checkbox
                          id={inputId}
                          checked={on}
                          disabled={required || running || c.noArm64 || (!c.available && !on)}
                          onChange={(v) => {
                            setChosen(toggle(list, chosen, c.id, v));
                          }}
                          className={unchosen ? "rounded-sm outline-2 outline-offset-1 outline-vsc-orange" : ""}
                        />
                        <span className="flex w-5 shrink-0 justify-center">
                          <ComponentLogo id={c.id} size={20} />
                        </span>
                        <label htmlFor={inputId} className="min-w-0 flex-1 cursor-pointer leading-[18px]">
                          <span
                            className={
                              unchosen ? "font-semibold text-vsc-orange" : "font-semibold text-vsc-settings-header"
                            }
                          >
                            {c.name}
                          </span>
                          <span className="ml-2 text-vsc-desc">{c.desc}</span>
                          <ProjectsChip names={forProjects} on={on} />
                          <Dependencies c={c} list={list} chosen={chosen} />
                          <LinkProblems id={c.id} links={links} />
                        </label>
                        {c.x64Only && (
                          <span className="text-[12px] text-vsc-desc" title={t("Su PC ARM64 gira in emulazione x64")}>
                            x64
                          </span>
                        )}
                        <span className="w-16 shrink-0 text-right text-[12px] text-vsc-desc">{sizeText(c.mb)}</span>
                        <span
                          className="w-40 shrink-0 truncate text-right text-[12px]"
                          title={versions ? undefined : (c.version ?? undefined)}
                        >
                          {state ? (
                            <ItemStatus state={state} />
                          ) : versions ? null /* its versions are in the list below */ : c.installed ? (
                            <span className="text-vsc-success">
                              <VscCheck className="mr-1 inline" size={12} aria-hidden="true" />
                              {shown ?? t("installato")}
                            </span>
                          ) : c.noArm64 ? (
                            <span className="text-vsc-desc">{t("non disponibile su PC ARM64")}</span>
                          ) : (
                            <span className="text-vsc-desc">{t("non installato")}</span>
                          )}
                        </span>
                      </div>
                      {state && <ItemBar state={state} label={c.name} className="mt-1.5 ml-[62px]" />}
                      {state?.kind === "problem" && (
                        <p className="mt-1 ml-[62px] text-[12px] text-vsc-error select-text">{state.text}</p>
                      )}
                      {versions && (
                        <div className="pl-[62px]">
                          <ToolVersions tool={versions} kit={kit} setup={setup} outside={outsideOf(links, c.id)} />
                        </div>
                      )}
                    </div>
                    {c.id === "php" && <ComposerRow php={c} tool={versioned} on={on} state={state} />}
                  </Fragment>
                );
              })}
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
