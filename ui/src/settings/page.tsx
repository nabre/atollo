// Settings page (data\app\settings.json, tauri/src/settings.rs), laid
// out as the Projects page: the side bar lists the sections (language,
// network, Gemini key, versions of Python, PHP and Rust, the app's updates,
// the GitHub account, where the kit is) and the tree of the new-project
// wizard's frameworks (frameworks/); each opens in a tab of the editor.
// Every change is saved and applied as it is made, with no Save button
// (D-137): the tab bar says when. The search on top shows every setting
// that matches. Replaces nabre/dev-kit's proxy.cmd.
import { useEffect, useState, type ReactNode } from "react";
import type { IconType } from "react-icons";
import { editorRestart, type AppInfo } from "../api";
import { GithubAccount, GitIdentitySetting } from "../github";
import type { GuideTopicId } from "../guide";
import { BrandSymbol } from "../brand";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import {
  addPiece,
  FrameworksTree,
  IntegrationGroup,
  parseIntegrationTab,
  PieceGroup,
  pieceTab,
  TRY_TAB,
  TryGroup,
  useFrameworks,
} from "../frameworks";
import { t } from "../i18n";
import {
  Codicon,
  codicon,
  EditorTabs,
  IconButton,
  Link,
  List,
  ListRow,
  Notice,
  PaneStack,
  SearchBox,
  SettingItem,
  SettingsGroup,
  SettingsSearch,
  SideBarTitle,
  useTabs,
  ViewContainer,
  ViewPane,
  Watermark,
  type Tab,
} from "../workbench";
import { KitInfo, Privacy, Updates } from "./about";
import { useSettingsAutosave } from "./autosave";
import { AgentSettings, GeneralSettings, NetworkSettings } from "./sections";

/** The wait after an edit of the frameworks before saving it. */
const FRAMEWORKS_DELAY = 800;

type SectionId = "general" | "network" | "agents" | "update" | "github" | "privacy" | "kit";

function sections(): { id: SectionId; label: string; icon: IconType }[] {
  return [
    { id: "general", label: t("Generale"), icon: codicon("settings") },
    { id: "network", label: t("Rete"), icon: codicon("globe") },
    { id: "agents", label: t("Agenti"), icon: codicon("hubot") },
    { id: "update", label: t("Aggiornamenti"), icon: codicon("cloud-download") },
    { id: "github", label: t("Account GitHub"), icon: codicon("github") },
    { id: "privacy", label: t("Privacy"), icon: codicon("shield") },
    { id: "kit", label: t("Il kit"), icon: codicon("info") },
  ];
}

type PaneId = "settings" | "frameworks";

export function SettingsPage({
  onSaved,
  editorRunning,
  info,
  onGuide,
}: {
  onSaved: () => void;
  editorRunning: boolean;
  info: AppInfo | null;
  onGuide?: (topic: GuideTopicId) => void;
}) {
  const {
    view,
    s,
    loadError,
    error: saveError,
    status,
    needsRestart,
    set,
    net,
    setGemini,
  } = useSettingsAutosave(onSaved);
  const [search, setSearch] = useState("");
  const [panes, setPanes] = useState<Set<PaneId>>(() => new Set(["settings", "frameworks"]));
  const tabs = useTabs(["general"]);
  const { busy, error, run } = useAction();
  const fw = useFrameworks();
  const loadFrameworks = fw.load;
  useEffect(loadFrameworks, [loadFrameworks]);

  // The frameworks saved a moment after the last edit.
  const saveFrameworks = fw.save;
  useEffect(() => {
    if (!fw.dirty) return;
    const timer = window.setTimeout(() => {
      run("frameworks", saveFrameworks);
    }, FRAMEWORKS_DELAY);
    return () => {
      window.clearTimeout(timer);
    };
  }, [fw.dirty, saveFrameworks, run]);

  const toggle = (id: PaneId) => {
    setPanes((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const open = (tab: string) => {
    tabs.show(tab);
    setSearch("");
  };
  const all = sections();

  /** A section of the settings as its group (a tab, or a search result). */
  const group = (id: SectionId): ReactNode => {
    if (!s || !view) return null;
    switch (id) {
      case "general":
        return <GeneralSettings key={id} s={s} set={set} platform={info?.platform} />;
      case "network":
        return <NetworkSettings key={id} s={s} net={net} platform={info?.platform} />;
      case "agents":
        return <AgentSettings key={id} view={view} onGemini={setGemini} />;
      case "update":
        return <Updates key={id} />;
      case "github":
        return (
          <SettingsGroup key={id} id="s-github" title={t("Account GitHub")}>
            <SettingItem title={t("Account GitHub del kit")} keywords="github git gh login">
              <GithubAccount />
            </SettingItem>
            <GitIdentitySetting />
          </SettingsGroup>
        );
      case "privacy":
        return <Privacy key={id} />;
      case "kit":
        return info ? <KitInfo key={id} info={info} /> : null;
    }
  };

  /** What a tab shows: a section, a framework or the trial. */
  const content = (tab: string): ReactNode => {
    if (tab === TRY_TAB) return <TryGroup fw={fw} />;
    const integration = parseIntegrationTab(tab);
    if (integration) {
      return (
        <IntegrationGroup
          key={tab}
          fw={fw}
          addon={integration.addon}
          base={integration.base}
          onOpen={open}
          onRemoved={() => {
            tabs.close(tab);
          }}
        />
      );
    }
    if (tab.startsWith("fw:")) {
      return (
        <PieceGroup
          key={tab}
          fw={fw}
          id={tab.slice(3)}
          onRemoved={() => {
            tabs.close(tab);
          }}
        />
      );
    }
    return all.some((x) => x.id === tab) ? group(tab as SectionId) : null;
  };
  const tabOf = (id: string): Tab => {
    const section = all.find((x) => x.id === id);
    if (section) {
      const Icon = section.icon;
      return { id, label: section.label, icon: <Icon size={16} aria-hidden="true" /> };
    }
    if (id === TRY_TAB) {
      return { id, label: t("Prova una combinazione"), icon: <Codicon name="beaker" size={16} /> };
    }
    const integration = parseIntegrationTab(id);
    if (integration) {
      const addon = fw.pieces.find((p) => p.id === integration.addon);
      const base = fw.pieces.find((p) => p.id === integration.base);
      return {
        id,
        label: addon?.name ?? integration.addon,
        description: base?.name ?? integration.base,
        icon: <Codicon name="extensions" size={16} />,
      };
    }
    const piece = fw.pieces.find((p) => pieceTab(p.id) === id);
    const Icon = piece?.kind === "addon" ? codicon("extensions") : codicon("package");
    return { id, label: piece?.name ?? id, icon: <Icon size={16} aria-hidden="true" /> };
  };

  const side = (
    <>
      <SideBarTitle
        title={t("Impostazioni")}
        actions={
          onGuide ? (
            <IconButton
              icon={codicon("book")}
              label={t("Apri la guida di questa pagina")}
              onClick={() => {
                onGuide(tabs.active === "github" ? "github" : tabs.active === "network" ? "network" : "settings");
              }}
            />
          ) : undefined
        }
      />
      <div className="px-3 pb-1.5">
        <SearchBox value={search} onChange={setSearch} placeholder={t("Cerca nelle impostazioni")} />
      </div>
      <PaneStack>
        <ViewPane
          first
          title={t("Impostazioni")}
          expanded={panes.has("settings")}
          onToggle={() => {
            toggle("settings");
          }}
        >
          <List label={t("Impostazioni")}>
            {all.map(({ id, label, icon: Icon }) => (
              <ListRow
                key={id}
                icon={<Icon size={16} aria-hidden="true" />}
                label={label}
                selected={!search && tabs.active === id}
                current="page"
                onSelect={() => {
                  open(id);
                }}
              />
            ))}
          </List>
        </ViewPane>
        <ViewPane
          fill
          title={t("Framework")}
          expanded={panes.has("frameworks")}
          onToggle={() => {
            toggle("frameworks");
          }}
          badge={fw.pieces.length > 0 ? fw.pieces.length : undefined}
          actions={
            <>
              <IconButton
                icon={codicon("new-folder")}
                label={t("Aggiungi una base")}
                onClick={() => {
                  open(pieceTab(addPiece(fw, "base")));
                }}
              />
              <IconButton
                icon={codicon("add")}
                label={t("Aggiungi un'aggiunta")}
                onClick={() => {
                  open(pieceTab(addPiece(fw, "addon")));
                }}
              />
            </>
          }
        >
          <FrameworksTree fw={fw} selected={search ? null : tabs.active} onSelect={open} />
        </ViewPane>
      </PaneStack>
    </>
  );

  const saving = status === "saving" || busy === "frameworks" || fw.dirty;
  return (
    <ViewContainer id="settings" side={side}>
      <div className="flex min-h-0 flex-1 flex-col">
        <EditorTabs
          tabs={tabs.open.map(tabOf)}
          active={search ? null : tabs.active}
          onSelect={open}
          onClose={tabs.close}
          actions={
            <span role="status" className={`px-2 ${saving ? "text-vsc-desc" : "text-vsc-success"}`}>
              {saving ? t("Salvataggio…") : status === "saved" ? t("Salvato.") : ""}
            </span>
          }
        />
        {/* The settings' tree, its rows 24px in from the sides as in VSCodium. */}
        <div className="min-h-0 flex-1 overflow-y-auto px-vsc-size240 pb-vsc-size400">
          {(loadError ?? view?.problem ?? saveError ?? error ?? (needsRestart && editorRunning)) && (
            <div className="space-y-2 px-[14px] pt-3">
              <ErrorLine error={loadError} />
              {view?.problem && (
                <ErrorLine
                  error={t(
                    "Le impostazioni in {0} non sono leggibili: uso quelle predefinite. Il file verrà riscritto alla prima modifica.",
                    view.problem,
                  )}
                />
              )}
              <ErrorLine error={saveError} />
              <ErrorLine error={error} />
              {needsRestart && editorRunning && (
                <Notice tone="info">
                  {t("Riavvia l'editor perché i suoi terminali usino le nuove impostazioni.")}{" "}
                  <Link
                    onClick={() => {
                      run("restart", editorRestart);
                    }}
                  >
                    {t("Riavvia l'editor")}
                  </Link>
                </Notice>
              )}
            </div>
          )}
          {!s && !loadError && <p className="p-5 text-vsc-desc">{t("Caricamento…")}</p>}
          <SettingsSearch search={search}>
            {search ? (
              <>
                {all.map((x) => group(x.id))}
                {fw.pieces.map((p) => (
                  <PieceGroup
                    key={p.id}
                    fw={fw}
                    id={p.id}
                    onRemoved={() => {
                      tabs.close(pieceTab(p.id));
                    }}
                  />
                ))}
                <TryGroup fw={fw} />
              </>
            ) : tabs.active ? (
              content(tabs.active)
            ) : (
              <Watermark>
                <BrandSymbol size={96} />
                <p className="text-vsc-desc">{t("Scegli un'impostazione nella barra laterale.")}</p>
              </Watermark>
            )}
          </SettingsSearch>
        </div>
      </div>
    </ViewContainer>
  );
}
