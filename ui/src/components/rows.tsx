// The rows of the Tools page: a component with its checkbox, its tags and
// state and its versions; Composer under PHP.
import { setupStart, type ComponentInfo, type RelationLink, type SetupSnapshot, type VersionedTool } from "../ipc";
import { t } from "../i18n";
import { Checkbox, Codicon } from "../workbench";
import { ItemBar, ItemStatus, itemState, type ItemState } from "./setup";
import { composerOf, splitVersion, ToolInstallLine, ToolVersions, type KitVersions } from "../versions";
import { ByHandBox } from "./byhand";
import { LinkProblems, outsideOf } from "./relations";
import { sizeText } from "./model";
import { ComponentLogo } from "../shared/logos";
import { Badge, Tag } from "../shared/tags";

/** The projects that need `c`: blue when chosen, orange with a warning when not. */
export function ProjectsChip({ names, on }: { names: string[]; on: boolean }) {
  if (names.length === 0) return null;
  const list = names.join(", ");
  return on ? (
    <Tag
      tone="focus"
      className="ml-2"
      title={t("Serve ai progetti: {0}.", list)}
      icon={<Codicon name="folder-library" size={11} />}
    >
      <span className="sr-only">{t("serve ai progetti")} </span>
      {list}
    </Tag>
  ) : (
    <Tag
      tone="warning"
      className="ml-2"
      title={t("Serve ai progetti: {0}. Non è scelto: sceglilo perché funzionino.", list)}
      icon={<Codicon name="warning" size={11} />}
    >
      <span className="sr-only">{t("attenzione: non scelto, serve ai progetti")} </span>
      {list}
    </Tag>
  );
}

/** What `c` needs (installed with it) and, when chosen, what needs `c`. */
export function Dependencies({ c, list, chosen }: { c: ComponentInfo; list: ComponentInfo[]; chosen: Set<string> }) {
  const name = (id: string) => list.find((x) => x.id === id)?.name ?? id;
  const usedBy = list.filter((x) => chosen.has(x.id) && x.needs.includes(c.id)).map((x) => x.name);
  return (
    <>
      {c.needs.map((n) => (
        <Tag
          key={n}
          className="ml-2"
          title={t("Richiede {0}: si installa insieme.", name(n))}
          icon={<Codicon name="type-hierarchy-sub" size={11} />}
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
          icon={<Codicon name="references" size={11} />}
        >
          {t("per {0}", usedBy.join(", "))}
        </Tag>
      )}
    </>
  );
}

/** Composer: a tool of its own with its version, but one per version of
 * PHP and compatible with it, installed and updated with it; its checkbox
 * follows PHP's. */
export function ComposerRow({
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
    <div data-setting className="border-l-2 border-l-transparent py-2 pr-[14px] pl-3 hover:bg-vsc-list-hover/50">
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
          <Tag dashed className="ml-2" icon={<Codicon name="references" size={11} />}>
            {t("con {0}", php.name)}
          </Tag>
        </label>
        <span className="w-16 shrink-0" />
        <span className="w-40 shrink-0 truncate text-right text-[12px]">
          {state ? (
            <ItemStatus state={state} />
          ) : all.length > 1 ? null /* its versions are in the list below */ : kit ? (
            <span className="text-vsc-success">
              <Codicon name="check" className="mr-1" size={12} />
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

/** A component in its section: its checkbox, logo, name with its tags, size
 * and state, and its versions; PHP is followed by Composer's row. */
export function ComponentRow({
  c,
  list,
  chosen,
  forProjects,
  kit,
  setup,
  links,
  onChange,
}: {
  c: ComponentInfo;
  list: ComponentInfo[];
  chosen: Set<string>;
  /** Names of the projects that need it. */
  forProjects: string[];
  kit: KitVersions;
  setup: SetupSnapshot | null;
  links: RelationLink[];
  onChange: (on: boolean) => void;
}) {
  const running = setup?.running ?? false;
  const required = c.locked;
  const on = chosen.has(c.id);
  const inputId = `comp-${c.id}`;
  const unchosen = forProjects.length > 0 && !on;
  // The versions of what is installed, checked or not; of what is not, the
  // version to install instead of the latest.
  const versioned = kit.byId.get(c.id);
  const versions = versioned?.installed ? versioned : null;
  const installable =
    versioned && !versioned.installed && versioned.downloadable && c.available && c.unavailableReason === null
      ? versioned
      : null;
  // Its step in the preparation: waiting, downloading, failed.
  const state = itemState(setup, c.id, c.name);
  // A left bar marks what a project needs: emerald, orange when not chosen
  // (a 5% tint: the orange text on it stays at 4.5:1 in light).
  const mark = unchosen
    ? "border-l-vsc-orange bg-vsc-orange/5"
    : forProjects.length > 0
      ? "border-l-vsc-focus"
      : on !== (c.chosen || c.locked)
        ? "border-l-vsc-settings-modified"
        : "border-l-transparent";
  // PHP's version without its Composer, which has its row.
  const shown = composerOf(c.version) ? splitVersion(c.version).main : c.version;
  return (
    <>
      <div data-setting className={`border-l-2 py-2 pr-[14px] pl-3 hover:bg-vsc-list-hover/50 ${mark}`}>
        <div className="flex items-center gap-3">
          <Checkbox
            id={inputId}
            checked={on}
            disabled={required || running || c.unavailableReason !== null || (!c.available && !on)}
            onChange={(v) => {
              onChange(v);
            }}
            className={unchosen ? "rounded-sm outline-2 outline-offset-1 outline-vsc-orange" : ""}
          />
          <span className="flex w-5 shrink-0 justify-center">
            <ComponentLogo id={c.id} size={20} />
          </span>
          <label htmlFor={inputId} className="min-w-0 flex-1 cursor-pointer leading-[18px]">
            <span className={unchosen ? "font-semibold text-vsc-orange" : "font-semibold text-vsc-settings-header"}>
              {c.name}
            </span>
            <span className="ml-2 text-vsc-desc">{c.desc}</span>
            <ProjectsChip names={forProjects} on={on} />
            <Dependencies c={c} list={list} chosen={chosen} />
            <LinkProblems id={c.id} links={links} />
          </label>
          {c.x64Only && (
            <span
              className="text-[12px] text-vsc-desc"
              title={t("Su un Mac con Apple Silicon gira in emulazione x64 con Rosetta 2")}
            >
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
                <Codicon name="check" className="mr-1" size={12} />
                {shown ?? t("installato")}
              </span>
            ) : c.unavailableReason !== null ? (
              <span className="text-vsc-desc">{c.unavailableReason}</span>
            ) : (
              <span className="text-vsc-desc">{t("non installato")}</span>
            )}
          </span>
        </div>
        {state && <ItemBar state={state} label={c.name} className="mt-1.5 ml-[62px]" />}
        {state?.kind === "problem" && (
          <p className="mt-1 ml-[62px] text-[12px] text-vsc-error select-text">{state.text}</p>
        )}
        {state?.kind === "problem" && state.byHand && (
          <div className="ml-[62px]">
            <ByHandBox
              id={c.id}
              name={c.name}
              byHand={state.byHand}
              disabled={running}
              onRetry={() => setupStart([c.id], false)}
            />
          </div>
        )}
        {versions && (
          <div className="pl-[62px]">
            <ToolVersions tool={versions} kit={kit} setup={setup} outside={outsideOf(links, c.id)} />
          </div>
        )}
        {installable && !state && (
          <div className="pl-[62px]">
            <ToolInstallLine tool={installable} kit={kit} setup={setup} />
          </div>
        )}
      </div>
      {c.id === "php" && <ComposerRow php={c} tool={versioned} on={on} state={state} />}
    </>
  );
}
