// Settings page (datapp\settings.json, src-tauri/src/settings.rs), laid
// out as the Projects page: the side bar lists the sections (language,
// network, Gemini key, versions of Python, PHP and Rust, the app's updates,
// the GitHub account, where the kit is) and the tree of the new-project
// wizard's frameworks (frameworks.tsx); each opens in a tab of the editor,
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
import {
  editorRestart,
  settingsGet,
  settingsSet,
  updateCheck,
  updateStatus,
  type Settings,
  type SettingsView,
  type UpdateStatus,
  type AppInfo,
} from "./api";
import { GithubAccount } from "./github";
import { BrandSymbol } from "./brand";
import { ErrorLine, useAction } from "./pages";
import { addPiece, FrameworksTree, PieceGroup, pieceTab, TRY_TAB, TryGroup, useFrameworks } from "./frameworks";
import { LANGS, t } from "./i18n";
import {
  Button,
  CheckboxLabel,
  EditorTabs,
  IconButton,
  Link,
  List,
  ListRow,
  Notice,
  PaneStack,
  Properties,
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
} from "./workbench";

const empty = (s: string) => (s.trim() === "" ? null : s.trim());

/** A text setting: its title is the field's label. */
function TextSetting({
  category,
  title,
  description,
  value,
  placeholder,
  onChange,
}: {
  category: string;
  title: string;
  description?: string;
  value: string | null;
  placeholder?: string;
  onChange: (v: string | null) => void;
}) {
  const id = `set-${category}-${title}`.replace(/\W+/g, "-");
  return (
    <SettingItem category={category} title={title} description={description} modified={value !== null}>
      <input
        id={id}
        aria-label={title}
        value={value ?? ""}
        placeholder={placeholder}
        onChange={(e) => {
          onChange(empty(e.target.value));
        }}
        className="input w-full"
      />
    </SettingItem>
  );
}

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
  const general = t("Generale");
  const network = t("Rete");
  const agents = t("Agenti");
  const versions = t("Versioni");

  /** A section of the settings as its group (a tab, or a search result). */
  const group = (id: SectionId): ReactNode => {
    if (!s || !view) return null;
    switch (id) {
      case "general":
        return (
          <SettingsGroup key={id} id="s-general" title={general}>
            <SettingItem
              category={general}
              title={t("Lingua")}
              description={t("La lingua di Atollo.")}
              modified={s.lang !== null}
            >
              <select
                aria-label={t("Lingua")}
                value={s.lang ?? ""}
                onChange={(e) => {
                  set({ lang: (empty(e.target.value) as Settings["lang"]) ?? null });
                }}
                className="input w-full"
              >
                <option value="">{t("Lingua di Windows")}</option>
                {LANGS.map((l) => (
                  <option key={l.id} value={l.id}>
                    {l.name}
                  </option>
                ))}
              </select>
            </SettingItem>
            <SettingItem
              category={general}
              title={t("Terminale")}
              description={t("Dove si aprono le console del kit.")}
              modified={s.terminal !== null}
            >
              <select
                aria-label={t("Terminale")}
                value={s.terminal ?? ""}
                onChange={(e) => {
                  set({ terminal: (empty(e.target.value) as Settings["terminal"]) ?? null });
                }}
                className="input w-full"
              >
                <option value="">{t("Windows Terminal del kit (predefinito)")}</option>
                <option value="classic">{t("Console classica di Windows")}</option>
              </select>
            </SettingItem>
            <SettingItem category={general} title={t("Aggiornamento automatico")} modified={!s.autoUpdate} inline>
              <CheckboxLabel
                checked={s.autoUpdate}
                onChange={(v) => {
                  set({ autoUpdate: v });
                }}
              >
                {t("Scarica da solo le nuove versioni di Atollo (sostituiscono l'exe alla chiusura)")}
              </CheckboxLabel>
            </SettingItem>
          </SettingsGroup>
        );
      case "network":
        return (
          <SettingsGroup
            key={id}
            id="s-network"
            title={network}
            description={t(
              "Solo se la rete passa da un proxy o ispeziona il traffico HTTPS: valgono per download, agenti e strumenti del kit.",
            )}
          >
            <TextSetting
              category={network}
              title={t("Proxy HTTPS")}
              value={s.network.httpsProxy}
              placeholder="http://proxy.azienda.it:8080"
              onChange={(v) => {
                net({ httpsProxy: v });
              }}
            />
            <TextSetting
              category={network}
              title={t("Proxy HTTP")}
              value={s.network.httpProxy}
              onChange={(v) => {
                net({ httpProxy: v });
              }}
            />
            <TextSetting
              category={network}
              title={t("Senza proxy per")}
              value={s.network.noProxy}
              placeholder="localhost,127.0.0.1,.azienda.it"
              onChange={(v) => {
                net({ noProxy: v });
              }}
            />
            <TextSetting
              category={network}
              title={t("Certificati aggiuntivi (file PEM)")}
              value={s.network.extraCaCerts}
              placeholder="E:\certificati\azienda.pem"
              onChange={(v) => {
                net({ extraCaCerts: v });
              }}
            />
            <SettingItem category={network} title={t("Rete locale")} modified={s.network.allowLan} inline>
              <CheckboxLabel
                checked={s.network.allowLan}
                onChange={(v) => {
                  net({ allowLan: v });
                }}
              >
                {t("Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete")}
              </CheckboxLabel>
            </SettingItem>
          </SettingsGroup>
        );
      case "agents":
        return (
          <SettingsGroup key={id} id="s-agents" title={agents}>
            <SettingItem
              category={agents}
              title={t("Chiave API di Gemini (Antigravity senza account Google)")}
              modified={view.geminiKeySet}
            >
              <input
                id="gemini-key"
                type="password"
                autoComplete="off"
                aria-label={t("Chiave API di Gemini (Antigravity senza account Google)")}
                value={gemini ?? ""}
                placeholder={view.geminiKeySet ? t("salvata: scrivi per sostituirla") : t("nessuna")}
                onChange={(e) => {
                  setSaved(false);
                  setDirty(true);
                  setGemini(e.target.value === "" ? null : e.target.value);
                }}
                className="input w-full"
              />
              {view.geminiKeySet && (
                <div className="mt-1.5">
                  <Link
                    onClick={() => {
                      setSaved(false);
                      setGemini("");
                    }}
                  >
                    {t("Togli la chiave")}
                  </Link>
                </div>
              )}
              {gemini === "" && <p className="mt-1 text-vsc-warning">{t("La chiave verrà tolta al salvataggio.")}</p>}
            </SettingItem>
          </SettingsGroup>
        );
      case "langs":
        return (
          <SettingsGroup
            key={id}
            id="s-langs"
            title={versions}
            description={t("Valgono dalla prossima installazione o aggiornamento in Strumenti.")}
          >
            <TextSetting
              category={versions}
              title={t("Python (vuoto = quella del kit)")}
              value={s.pythonVersion}
              placeholder="3.13"
              onChange={(v) => {
                set({ pythonVersion: v });
              }}
            />
            <TextSetting
              category={versions}
              title={t("PHP (vuoto = la più recente)")}
              value={s.phpVersion}
              placeholder="8.4"
              onChange={(v) => {
                set({ phpVersion: v });
              }}
            />
            <SettingItem category={versions} title={t("Compilatore di Rust")} modified={s.rustToolchain !== null}>
              <select
                aria-label={t("Compilatore di Rust")}
                value={s.rustToolchain ?? ""}
                onChange={(e) => {
                  set({ rustToolchain: (empty(e.target.value) as Settings["rustToolchain"]) ?? null });
                }}
                className="input w-full"
              >
                <option value="">{t("GNU (predefinito, tutto nel kit)")}</option>
                <option value="gnu">GNU</option>
                <option value="msvc">{t("MSVC (richiede Visual Studio Build Tools nel PC)")}</option>
              </select>
            </SettingItem>
          </SettingsGroup>
        );
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

/** Where the kit is (was on the Home, D-76). */
function KitInfo({ info }: { info: AppInfo }) {
  const facts: [string, string][] = [
    [t("Cartella del kit"), info.root],
    [t("Progetti"), info.projects],
    [t("Dati e programmi"), info.data],
    [t("Piattaforma"), info.platform],
    ["WebView2", info.webview],
    [t("Versione"), info.version],
  ];
  const rows = facts.map(([k, v]): [string, React.ReactNode] => [
    k,
    <span className="font-mono text-[12px] break-all select-text">{v}</span>,
  ]);
  return (
    <SettingsGroup id="s-kit" title={t("Il kit")}>
      <SettingItem
        title="Atollo"
        description={t("Ambiente di sviluppo portabile: linguaggi, agenti ed editor in una cartella.")}
        keywords={rows.map(([k]) => k).join(" ")}
      >
        <div className="flex items-start gap-4">
          <BrandSymbol size={40} />
          <Properties rows={rows} />
        </div>
      </SettingItem>
    </SettingsGroup>
  );
}

function Updates() {
  const [st, setSt] = useState<UpdateStatus | null>(null);
  const { busy, error, run } = useAction();
  useEffect(() => {
    updateStatus().then(setSt, () => undefined);
  }, []);
  if (!st) return null;
  let text: string;
  if (st.staged) text = t("La versione {0} è pronta: sostituisce questa alla chiusura di Atollo.", st.staged);
  else if (st.error) text = t("Controllo non riuscito: {0}", st.error);
  else if (st.latest && st.latest === st.current) text = t("Atollo {0} è la versione più recente.", st.current);
  else text = t("Versione installata: {0}", st.current);
  return (
    <SettingsGroup id="s-update" title={t("Aggiornamenti")}>
      <SettingItem title={t("Aggiornamenti di Atollo")} description={text}>
        <Button
          busy={busy === "check" || st.checking}
          onClick={() => {
            run("check", async () => {
              setSt(await updateCheck());
            });
          }}
        >
          {t("Cerca aggiornamenti")}
        </Button>
        <ErrorLine error={error} />
      </SettingItem>
    </SettingsGroup>
  );
}
