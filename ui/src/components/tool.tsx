// The page of one tool, as the page of a project: a header with what it is,
// its state and the button that chooses it, a navigation bar, one section
// at a time (its versions, its details, the projects that use it). The
// choice is applied with the bar under the page (./applybar.tsx).
import { useState } from "react";
import { VscCheck, VscLinkExternal, VscWarning } from "react-icons/vsc";
import { setupStart, urlOpen, type ComponentInfo, type RelationLink, type SetupSnapshot } from "../api";
import { t } from "../i18n";
import { ByHandBox } from "../byhand";
import { ComponentLogo } from "../logos";
import { LinkProblems, outsideOf } from "../relations";
import { ItemBar, ItemStatus, itemState } from "../setup";
import { Tag } from "../tags";
import { ToolInstallLine, ToolVersions, splitVersion, composerOf, type KitVersions } from "../versions";
import {
  Button,
  Dot,
  EditorBody,
  EditorHeader,
  EditorSection,
  IconButton,
  Link,
  NavBar,
  Notice,
  type NavItem,
} from "../workbench";
import { groups, sizeText, xcodeMissing } from "./model";
import { ComposerRow, Dependencies } from "./rows";
import type { ComponentsState } from "./state";

type Section = "versions" | "details" | "projects";

export function ToolPage({
  c,
  state,
  kit,
  setup,
  links,
  onOpen,
}: {
  c: ComponentInfo;
  state: ComponentsState;
  kit: KitVersions;
  setup: SetupSnapshot | null;
  links: RelationLink[];
  /** Opens the page of another tool (what it needs, what needs it). */
  onOpen: (id: string) => void;
}) {
  const list = state.list ?? [];
  const { chosen, choose, neededBy, projects } = state;
  const [section, setSection] = useState<Section>(c.versioned ? "versions" : "details");
  const running = setup?.running ?? false;
  const on = chosen.has(c.id);
  const pending = on !== (c.chosen || c.locked);
  const forProjects = neededBy.get(c.id) ?? [];
  const item = itemState(setup, c.id, c.name);
  const versioned = kit.byId.get(c.id);
  const installable =
    versioned &&
    !versioned.installed &&
    versioned.downloadable &&
    c.available &&
    !c.noArm64 &&
    c.unavailableReason === null
      ? versioned
      : null;
  const group = groups().find((g) => g.has(c));
  const name = (id: string) => list.find((x) => x.id === id)?.name ?? id;
  const usedBy = list.filter((x) => chosen.has(x.id) && x.needs.includes(c.id));
  const shown = composerOf(c.version) ? splitVersion(c.version).main : c.version;
  // Projects that use it, with the version each chose (versioned tools).
  const lines = versioned?.projects ?? {};
  const users = projects.filter((p) => p.requires.includes(c.id) || Object.values(lines).flat().includes(p.id));

  const items: NavItem<Section>[] = [
    ...(c.versioned ? [{ id: "versions" as const, label: t("Versioni") }] : []),
    { id: "details", label: t("Dettagli") },
    {
      id: "projects",
      label: t("Progetti"),
      mark:
        forProjects.length > 0 && !on ? (
          <VscWarning size={12} className="text-vsc-warning" aria-label={t("Serve a progetti e non è scelto")} />
        ) : users.length > 0 ? (
          <span className="text-[11px] text-vsc-desc">{users.length}</span>
        ) : undefined,
    },
  ];

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <EditorHeader
        icon={<ComponentLogo id={c.id} size={56} />}
        title={c.name}
        meta={[
          group ? <span key="group">{group.title}</span> : null,
          item ? (
            <ItemStatus key="state" state={item} />
          ) : c.installed ? (
            <span key="state" className="flex items-center gap-1 text-vsc-success">
              <VscCheck size={14} aria-hidden="true" />
              {shown ?? t("installato")}
            </span>
          ) : c.bootstrapStaged ? (
            <span
              key="state"
              className="flex items-center gap-1 text-vsc-success"
              title={t("Gli archivi sono già nel kit: l'installazione termina al primo avvio su quel sistema")}
            >
              <VscCheck size={14} aria-hidden="true" />
              {shown ? `${shown} (${t("pre-installato")})` : t("pre-installato")}
            </span>
          ) : c.noArm64 ? (
            <span key="state" className="text-vsc-desc">
              {t("non disponibile su PC ARM64")}
            </span>
          ) : c.unavailableReason !== null ? (
            <span key="state" className="text-vsc-desc">
              {c.unavailableReason}
            </span>
          ) : c.needsWindows11 ? (
            <span key="state" className="text-vsc-desc">
              {t("richiede Windows 11 su ARM64")}
            </span>
          ) : (
            <span key="state" className="text-vsc-desc">
              {t("non installato")}
            </span>
          ),
          <span key="size" className="text-vsc-desc">
            {sizeText(c.mb)}
          </span>,
          c.x64Only ? (
            <span key="x64" className="text-vsc-desc" title={t("Su Windows ARM64 gira in emulazione x64")}>
              x64
            </span>
          ) : null,
          pending ? (
            <span key="pending" className="flex items-center gap-1 text-vsc-settings-modified">
              <Dot tone="warn" />
              {on ? t("da installare con «Applica»") : t("da rimuovere con «Applica»")}
            </span>
          ) : null,
        ]}
        description={c.desc}
        actions={
          <>
            {c.locked ? (
              <Button disabled title={t("Sempre installato")}>
                {t("Obbligatorio")}
              </Button>
            ) : on ? (
              <Button
                disabled={running}
                onClick={() => {
                  choose(c.id, false);
                }}
              >
                {c.installed || c.bootstrapStaged ? t("Rimuovi") : t("Non installare")}
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={running || !c.available}
                onClick={() => {
                  choose(c.id, true);
                }}
              >
                {c.installed || c.bootstrapStaged ? t("Mantieni") : t("Installa")}
              </Button>
            )}
            <IconButton
              icon={VscLinkExternal}
              label={t("Sito ufficiale")}
              title={c.home}
              onClick={() => {
                void urlOpen(c.home);
              }}
            />
          </>
        }
      />
      {item && (
        <div className="space-y-1 px-5 pb-2">
          <ItemBar state={item} label={c.name} />
          {item.kind === "problem" && <p className="text-[12px] text-vsc-error select-text">{item.text}</p>}
          {item.kind === "problem" && item.byHand && (
            <ByHandBox
              id={c.id}
              name={c.name}
              byHand={item.byHand}
              disabled={running}
              onRetry={() => setupStart([c.id], false, state.target)}
            />
          )}
        </div>
      )}
      {xcodeMissing(c) && (
        <div className="px-5 pb-2">
          <Notice tone="warning">
            {t("I programmi che usano i framework Apple richiedono Xcode Command Line Tools")}{" "}
            <span className="text-vsc-desc">{t("Si installano con xcode-select --install, da amministratore.")}</span>
          </Notice>
        </div>
      )}
      <NavBar label={t("Sezioni dello strumento")} items={items} active={section} onSelect={setSection} />
      <EditorBody>
        {section === "versions" && (
          <EditorSection id="versions" title={t("Versioni")}>
            {versioned?.installed ? (
              <ToolVersions tool={versioned} kit={kit} setup={setup} outside={outsideOf(links, c.id)} />
            ) : installable ? (
              <ToolInstallLine tool={installable} kit={kit} setup={setup} />
            ) : (
              <p className="text-vsc-desc">{t("Si installa all'ultima versione.")}</p>
            )}
            {c.id === "php" && (
              <div className="mt-3">
                <ComposerRow php={c} tool={versioned} on={on} state={item} />
              </div>
            )}
          </EditorSection>
        )}
        {section === "details" && (
          <EditorSection id="details" title={t("Dettagli")}>
            <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-1.5">
              <dt className="text-vsc-desc">{t("Tipo")}</dt>
              <dd>{group?.title ?? c.group}</dd>
              <dt className="text-vsc-desc">{t("Versione")}</dt>
              <dd className="font-mono select-text">
                {c.version ?? t("non installato")}
                {!c.versioned && <span className="ml-2 font-sans text-vsc-desc">{t("sempre l'ultima")}</span>}
              </dd>
              <dt className="text-vsc-desc">{t("Dimensione")}</dt>
              <dd>{sizeText(c.mb)}</dd>
              <dt className="text-vsc-desc">{t("Richiede")}</dt>
              <dd>
                {c.needs.length === 0 ? (
                  <span className="text-vsc-desc">{t("niente")}</span>
                ) : (
                  c.needs.map((n, i) => (
                    <span key={n}>
                      {i > 0 && ", "}
                      <Link
                        onClick={() => {
                          onOpen(n);
                        }}
                      >
                        {name(n)}
                      </Link>
                    </span>
                  ))
                )}
              </dd>
              <dt className="text-vsc-desc">{t("Serve a")}</dt>
              <dd>
                {usedBy.length === 0 ? (
                  <span className="text-vsc-desc">{t("niente")}</span>
                ) : (
                  usedBy.map((x, i) => (
                    <span key={x.id}>
                      {i > 0 && ", "}
                      <Link
                        onClick={() => {
                          onOpen(x.id);
                        }}
                      >
                        {x.name}
                      </Link>
                    </span>
                  ))
                )}
              </dd>
              <dt className="text-vsc-desc">{t("Sito")}</dt>
              <dd>
                <Link
                  onClick={() => {
                    void urlOpen(c.home);
                  }}
                >
                  {c.home}
                </Link>
              </dd>
            </dl>
            <div className="mt-3 flex flex-wrap gap-y-1">
              <Dependencies c={c} list={list} chosen={chosen} />
              <LinkProblems id={c.id} links={links} />
            </div>
          </EditorSection>
        )}
        {section === "projects" && (
          <EditorSection id="projects" title={t("Progetti")}>
            {forProjects.length > 0 && !on && (
              <Notice tone="warning" className="mb-3">
                {t("Serve ai progetti: {0}. Non è scelto: sceglilo perché funzionino.", forProjects.join(", "))}
              </Notice>
            )}
            {users.length === 0 ? (
              <p className="text-vsc-desc">{t("Nessun progetto lo usa.")}</p>
            ) : (
              <ul className="m-0 list-none space-y-1 p-0">
                {users.map((p) => {
                  const line = Object.entries(lines).find(([, ids]) => ids.includes(p.id))?.[0];
                  return (
                    <li key={p.id} className="flex items-center gap-2">
                      <span>{p.name}</span>
                      {line && <Tag>{line}</Tag>}
                      {p.requires.includes(c.id) && (
                        <Tag tone={on ? "focus" : "warning"}>{t("richiesto dal suo codice")}</Tag>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}
          </EditorSection>
        )}
      </EditorBody>
    </div>
  );
}
