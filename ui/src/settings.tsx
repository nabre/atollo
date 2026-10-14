// Settings page (data\app\settings.json, src-tauri/src/settings.rs) as
// VSCodium's Settings editor: search, table of contents, one item per
// setting. Language, network (proxy, certificates, local network), Gemini
// key, versions of Python, PHP and Rust, the app's updates, the GitHub
// account and where the kit is. Replaces nabre/atollo's proxy.cmd.
import { useCallback, useEffect, useState, type SyntheticEvent } from "react";
import { VscSettingsGear } from "react-icons/vsc";
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
import { LANGS, t } from "./i18n";
import {
  Button,
  CheckboxLabel,
  Link,
  Notice,
  Properties,
  SettingItem,
  SettingsEditor,
  SettingsGroup,
  SingleEditor,
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
  const { busy, error, run } = useAction();

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

  const title = t("Impostazioni");
  const icon = <VscSettingsGear size={16} aria-hidden="true" />;
  if (loadError) {
    return (
      <SingleEditor title={title} icon={icon}>
        <div className="p-5">
          <ErrorLine error={loadError} />
        </div>
      </SingleEditor>
    );
  }
  if (!s || !view) {
    return (
      <SingleEditor title={title} icon={icon}>
        <p className="p-5 text-vsc-desc">{t("Caricamento…")}</p>
      </SingleEditor>
    );
  }

  const set = (patch: Partial<Settings>) => {
    setSaved(false);
    setDirty(true);
    setS({ ...s, ...patch });
  };
  const net = (patch: Partial<Settings["network"]>) => {
    set({ network: { ...s.network, ...patch } });
  };
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    run("save", async () => {
      await settingsSet(s, gemini);
      setSaved(true);
      load();
      onSaved();
    });
  };

  const general = t("Generale");
  const network = t("Rete");
  const agents = t("Agenti");
  const versions = t("Versioni");
  const toc = [
    { id: "s-general", label: general },
    { id: "s-network", label: network },
    { id: "s-agents", label: agents },
    { id: "s-langs", label: versions },
    { id: "s-update", label: t("Aggiornamenti") },
    { id: "s-github", label: t("Account GitHub") },
    { id: "s-kit", label: t("Il kit") },
  ];

  return (
    <SingleEditor title={title} icon={icon}>
      <form onSubmit={submit} className="flex min-h-0 flex-1 flex-col">
        <SettingsEditor
          toc={toc}
          search={search}
          onSearch={setSearch}
          placeholder={t("Cerca nelle impostazioni")}
          actions={
            <>
              {saved && (
                <span role="status" className="text-vsc-success">
                  {t("Salvato.")}
                </span>
              )}
              <Button variant="primary" submit busy={busy === "save"} disabled={!dirty && gemini === null}>
                {t("Salva")}
              </Button>
            </>
          }
        >
          {(view.problem ?? error ?? (saved && editorRunning)) && (
            <div className="max-w-[880px] space-y-2 px-[14px] pt-3">
              {view.problem && (
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

          <SettingsGroup id="s-general" title={general}>
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

          <SettingsGroup
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

          <SettingsGroup id="s-agents" title={agents}>
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

          <SettingsGroup
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

          <Updates />

          <SettingsGroup id="s-github" title={t("Account GitHub")}>
            <SettingItem title={t("Account GitHub del kit")} keywords="github git gh login">
              <GithubAccount />
            </SettingItem>
          </SettingsGroup>

          {info && <KitInfo info={info} />}
        </SettingsEditor>
      </form>
    </SingleEditor>
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
