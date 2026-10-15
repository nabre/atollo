// Settings page (data\app\settings.json, tauri/src/settings.rs), laid
// out as the Projects page: the side bar lists the sections (language,
// network, Gemini key, versions of Python, PHP and Rust, the app's updates,
// the GitHub account, where the kit is) and the tree of the new-project
// wizard's frameworks (frameworks/); each opens in a tab of the editor,
// Save is on the tab bar. The search on top shows every setting that
// matches. Replaces nabre/atollo's proxy.cmd.
import { useCallback, useEffect, useState, type ReactNode, type SyntheticEvent } from "react";
import type { IconType } from "react-icons";
import {
  VscAdd,
  VscBeaker,
  VscCloudDownload,
  VscExtensions,
  VscGithub,
  VscGlobe,
  VscHubot,
  VscInfo,
  VscNewFolder,
  VscPackage,
  VscSettings,
  VscVersions,
} from "react-icons/vsc";
import { editorRestart, settingsGet, settingsSet, type Settings, type SettingsView, type AppInfo } from "../api";
import { GithubAccount } from "../github";
import { BrandSymbol } from "../brand";
import { useAction } from "../hooks";
import { ErrorLine } from "../common";
import { addPiece, FrameworksTree, PieceGroup, pieceTab, TRY_TAB, TryGroup, useFrameworks } from "../frameworks";
import { t } from "../i18n";
import {
  Button,
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
import { KitInfo, Updates } from "./about";
import { AgentSettings, GeneralSettings, NetworkSettings, VersionSettings } from "./sections";

type SectionId = "general" | "network" | "agents" | "langs" | "update" | "github" | "kit";

function sections(): { id: SectionId; label: string; icon: IconType }[] {
  return [
    { id: "general", label: t("Generale"), icon: VscSettings },
    { id: "network", label: t("Rete"), icon: VscGlobe },
    { id: "agents", label: t("Agenti"), icon: VscHubot },
    { id: "langs", label: t("Versioni"), icon: VscVersions },
    { id: "update", label: t("Aggiornamenti"), icon: VscCloudDownload },
    { id: "github", label: t("Account GitHub"), icon: VscGithub },
    { id: "kit", label: t("Il kit"), icon: VscInfo },
  ];
}

type PaneId = "settings" | "frameworks";

export function SettingsPage({
  onSaved,
  editorRunning,
  info,
}: {
  onSaved: () => void;
  editorRunning: boolean;
  info: AppInfo | null;
}) {
  const [view, setView] = useState<SettingsView | null>(null);
  const [s, setS] = useState<Settings | null>(null);
  const [gemini, setGemini] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [search, setSearch] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [panes, setPanes] = useState<Set<PaneId>>(() => new Set(["settings", "frameworks"]));
  const tabs = useTabs(["general"]);
  const { busy, error, run } = useAction();
  const fw = useFrameworks();
  const loadFrameworks = fw.load;
  useEffect(loadFrameworks, [loadFrameworks]);

  const load = useCallback(() => {
    settingsGet().then(
      (v) => {
        setView(v);
        setS(v.settings);
        setGemini(null);
        setDirty(false);
      },
      (e: unknown) => {
        setLoadError(String(e));
      },
    );
  }, []);
  useEffect(load, [load]);

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
  const set = (patch: Partial<Settings>) => {
    if (!s) return;
    setSaved(false);
    setDirty(true);
    setS({ ...s, ...patch });
  };
  const net = (patch: Partial<Settings["network"]>) => {
    if (s) set({ network: { ...s.network, ...patch } });
  };
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("save", async () => {
      if (s && (dirty || gemini !== null)) await settingsSet(s, gemini);
      await fw.save();
      setSaved(true);
      load();
      onSaved();
    });
  };

  const all = sections();

  /** A section of the settings as its group (a tab, or a search result). */
  const group = (id: SectionId): ReactNode => {
    if (!s || !view) return null;
    switch (id) {
      case "general":
        return <GeneralSettings key={id} s={s} set={set} />;
      case "network":
        return <NetworkSettings key={id} s={s} net={net} />;
      case "agents":
        return (
          <AgentSettings
            key={id}
            view={view}
            gemini={gemini}
            onGemini={(key) => {
              setSaved(false);
              setDirty(true);
              setGemini(key);
            }}
            onRemoveKey={() => {
              setSaved(false);
              setGemini("");
            }}
          />
        );
      case "langs":
        return <VersionSettings key={id} s={s} set={set} />;
      case "update":
        return <Updates key={id} />;
      case "github":
        return (
          <SettingsGroup key={id} id="s-github" title={t("Account GitHub")}>
            <SettingItem title={t("Account GitHub del kit")} keywords="github git gh login">
              <GithubAccount />
            </SettingItem>
          </SettingsGroup>
        );
      case "kit":
        return info ? <KitInfo key={id} info={info} /> : null;
    }
  };

  /** What a tab shows: a section, a framework or the trial. */
  const content = (tab: string): ReactNode => {
    if (tab === TRY_TAB) return <TryGroup fw={fw} />;
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
      return { id, label: t("Prova una combinazione"), icon: <VscBeaker size={16} aria-hidden="true" /> };
    }
    const piece = fw.pieces.find((p) => pieceTab(p.id) === id);
    const Icon = piece?.kind === "addon" ? VscExtensions : VscPackage;
    return { id, label: piece?.name ?? id, icon: <Icon size={16} aria-hidden="true" /> };
  };

  const side = (
    <>
      <SideBarTitle title={t("Impostazioni")} />
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
                icon={VscNewFolder}
                label={t("Aggiungi una base")}
                onClick={() => {
                  open(pieceTab(addPiece(fw, "base")));
                }}
              />
              <IconButton
                icon={VscAdd}
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

  const unsaved = dirty || gemini !== null || fw.dirty;
  return (
    <ViewContainer id="settings" side={side}>
      <form onSubmit={submit} className="flex min-h-0 flex-1 flex-col">
        <EditorTabs
          tabs={tabs.open.map(tabOf)}
          active={search ? null : tabs.active}
          onSelect={open}
          onClose={tabs.close}
          actions={
            <>
              {saved && !unsaved && (
                <span role="status" className="px-1 text-vsc-success">
                  {t("Salvato.")}
                </span>
              )}
              <Button variant="primary" small submit busy={busy === "save"} disabled={!unsaved}>
                {t("Salva")}
              </Button>
            </>
          }
        />
        <div className="min-h-0 flex-1 overflow-y-auto pb-10">
          {(loadError ?? view?.problem ?? error ?? (saved && editorRunning)) && (
            <div className="max-w-[880px] space-y-2 px-[14px] pt-3">
              <ErrorLine error={loadError} />
              {view?.problem && (
                <ErrorLine
                  error={t(
                    "Le impostazioni in {0} non sono leggibili: uso quelle predefinite e non modifico il file.",
                    view.problem,
                  )}
                />
              )}
              <ErrorLine error={error} />
              {saved && editorRunning && (
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
      </form>
    </ViewContainer>
  );
}
