// The sections of Settings that are fields: general, network, agents,
// versions of the languages.
import { LANGS, t } from "../i18n";
import type { Settings, SettingsView } from "../api";
import { CheckboxLabel, Link, SettingItem, SettingsGroup } from "../workbench";

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

/** Generale: the language, the terminal, the automatic updates. */
export function GeneralSettings({ s, set }: { s: Settings; set: (patch: Partial<Settings>) => void }) {
  const general = t("Generale");
  return (
    <SettingsGroup id="s-general" title={general}>
      <SettingItem
        category={general}
        title={t("Lingua")}
        description={t("La lingua di NEXKORA.")}
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
          {t("Scarica da solo le nuove versioni di NEXKORA (sostituiscono l'exe alla chiusura)")}
        </CheckboxLabel>
      </SettingItem>
    </SettingsGroup>
  );
}

/** Rete: proxy, certificates, the local network. */
export function NetworkSettings({ s, net }: { s: Settings; net: (patch: Partial<Settings["network"]>) => void }) {
  const network = t("Rete");
  return (
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
  );
}

/** Agenti: the Gemini key, saved apart and never shown. */
export function AgentSettings({
  view,
  gemini,
  onGemini,
  onRemoveKey,
}: {
  view: SettingsView;
  /** The key typed (null: unchanged, "": to remove). */
  gemini: string | null;
  onGemini: (key: string | null) => void;
  onRemoveKey: () => void;
}) {
  const agents = t("Agenti");
  return (
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
            onGemini(e.target.value === "" ? null : e.target.value);
          }}
          className="input w-full"
        />
        {view.geminiKeySet && (
          <div className="mt-1.5">
            <Link onClick={onRemoveKey}>{t("Togli la chiave")}</Link>
          </div>
        )}
        {gemini === "" && <p className="mt-1 text-vsc-warning">{t("La chiave verrà tolta al salvataggio.")}</p>}
      </SettingItem>
    </SettingsGroup>
  );
}

/** Versioni: the lines of Python and PHP, Rust's compiler. */
export function VersionSettings({ s, set }: { s: Settings; set: (patch: Partial<Settings>) => void }) {
  const versions = t("Versioni");
  return (
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
  );
}
