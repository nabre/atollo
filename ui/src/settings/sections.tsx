// The sections of Settings that are fields: general, network, agents.
// The lines of the languages are chosen in Tools and per project, Rust's
// compiler on Rust's page in Tools (D-137). Each change is saved as it is made (D-137);
// the first-start page shows some of them too (setup.tsx).
import { useEffect, useRef, useState } from "react";
import { LANGS, t } from "../i18n";
import type { Settings, SettingsView } from "../api";
import { CheckboxLabel, Link, Select, SettingItem, SettingsGroup, TextField } from "../workbench";
import { CondaSettings } from "./conda";

const empty = (s: string) => (s.trim() === "" ? null : s.trim());

/** `platform` of AppInfo (`win32-x64`, `darwin-arm64`, `linux-x64`…);
 * unknown counts as Windows. */
const isWindows = (platform: string | undefined) => !platform || platform.startsWith("win32");

/** An example PEM file in this system's style. */
const pemExample = (platform: string | undefined) =>
  isWindows(platform) ? "E:\\certs\\company.pem" : "/etc/ssl/certs/company.pem";

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
      <TextField
        id={id}
        label={title}
        value={value ?? ""}
        placeholder={placeholder}
        onChange={(v) => {
          onChange(empty(v));
        }}
        className="w-full"
      />
    </SettingItem>
  );
}

/** The language: the system's, or one of the four. */
export function LangSetting({
  category,
  s,
  set,
}: {
  category: string;
  s: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  return (
    <SettingItem
      category={category}
      title={t("Lingua")}
      description={t("La lingua di Atollo.")}
      modified={s.lang !== null}
    >
      <Select
        label={t("Lingua")}
        value={s.lang ?? ""}
        onChange={(v) => {
          set({ lang: (empty(v) as Settings["lang"]) ?? null });
        }}
        options={[{ value: "", label: t("Lingua del sistema") }, ...LANGS.map((l) => ({ value: l.id, label: l.name }))]}
        className="w-full"
      />
    </SettingItem>
  );
}

/** Light or dark: the system's, or one always, for every window (D-137). */
export function ThemeSetting({
  category,
  s,
  set,
}: {
  category: string;
  s: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  return (
    <SettingItem
      category={category}
      title={t("Tema")}
      description={t("Chiaro o scuro per Atollo, l'editor e i terminali.")}
      keywords="theme dark light"
      modified={s.theme !== "system"}
    >
      <Select
        label={t("Tema")}
        value={s.theme}
        onChange={(v) => {
          set({ theme: v });
        }}
        options={[
          { value: "system", label: t("Come il sistema") },
          { value: "light", label: t("Chiaro") },
          { value: "dark", label: t("Scuro") },
        ]}
        className="w-full"
      />
    </SettingItem>
  );
}

/** The automatic updates of Atollo: a request to GitHub (PRIVACY.md). */
export function AutoUpdateSetting({
  category,
  s,
  set,
}: {
  category: string;
  s: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  return (
    <SettingItem category={category} title={t("Aggiornamento automatico")} modified={!s.autoUpdate} inline>
      <CheckboxLabel
        checked={s.autoUpdate}
        onChange={(v) => {
          set({ autoUpdate: v });
        }}
      >
        {t("Scarica da solo le nuove versioni di Atollo (sostituiscono l'exe alla chiusura)")}
      </CheckboxLabel>
    </SettingItem>
  );
}

/** Generale: the language, the theme, the programs' temporary files, the
 * automatic updates, the kept conda packages (D-118). */
export function GeneralSettings({
  s,
  set,
  platform,
}: {
  s: Settings;
  set: (patch: Partial<Settings>) => void;
  platform?: string | undefined;
}) {
  const general = t("Generale");
  return (
    <SettingsGroup id="s-general" title={general}>
      <LangSetting category={general} s={s} set={set} />
      <ThemeSetting category={general} s={s} set={set} />
      <SettingItem
        category={general}
        title={t("Temporanei dei programmi")}
        description={
          isWindows(platform)
            ? t(
                "Automatico usa la cartella temporanea di Windows se il kit è su una chiavetta, su un disco di rete o su FAT32/exFAT, altrimenti la cartella data\\tmp del kit. Nel sistema i file restano sul PC finché chiudi l'app.",
              )
            : t(
                "Automatico usa la cartella temporanea del sistema se il kit è su una chiavetta, su un disco di rete o su FAT32/exFAT, altrimenti la cartella data/tmp del kit. Nel sistema i file restano sul computer finché chiudi l'app.",
              )
        }
        modified={s.programTemp !== "auto"}
      >
        <Select
          label={t("Temporanei dei programmi")}
          value={s.programTemp}
          onChange={(v) => {
            set({ programTemp: v });
          }}
          options={[
            { value: "auto", label: t("Automatico") },
            { value: "kit", label: t("Nel kit") },
            { value: "system", label: t("Nel sistema") },
          ]}
          className="w-full"
        />
      </SettingItem>
      <AutoUpdateSetting category={general} s={s} set={set} />
      <CondaSettings category={general} s={s} set={set} />
    </SettingsGroup>
  );
}

/** Rete: proxy, certificates, the local network. */
export function NetworkSettings({
  s,
  net,
  platform,
}: {
  s: Settings;
  net: (patch: Partial<Settings["network"]>) => void;
  platform?: string | undefined;
}) {
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
        placeholder="http://proxy.example.com:8080"
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
        placeholder="localhost,127.0.0.1,.example.com"
        onChange={(v) => {
          net({ noProxy: v });
        }}
      />
      <TextSetting
        category={network}
        title={t("Certificati aggiuntivi (file PEM)")}
        value={s.network.extraCaCerts}
        placeholder={pemExample(platform)}
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

/** Agenti: the Gemini key, saved apart and never shown. A key typed is
 * saved when the field loses the focus, then the field empties. */
export function AgentSettings({
  view,
  onGemini,
}: {
  view: SettingsView;
  /** A new key, or "" to remove it. */
  onGemini: (key: string) => void;
}) {
  const agents = t("Agenti");
  const [typed, setTyped] = useState("");
  const commit = () => {
    if (typed.trim() === "") return;
    onGemini(typed);
    setTyped("");
  };
  // A key typed and not committed is saved when the section closes.
  const left = useRef({ typed, onGemini });
  useEffect(() => {
    left.current = { typed, onGemini };
  }, [typed, onGemini]);
  useEffect(
    () => () => {
      const { typed: key, onGemini: save } = left.current;
      if (key.trim() !== "") save(key);
    },
    [],
  );
  return (
    <SettingsGroup id="s-agents" title={agents}>
      <SettingItem
        category={agents}
        title={t("Chiave API di Gemini (Antigravity senza account Google)")}
        description={t(
          "Resta nel kit, in data/app/secrets, leggibile solo dal tuo utente, e arriva solo ai programmi del kit.",
        )}
        modified={view.geminiKeySet}
      >
        <TextField
          id="gemini-key"
          type="password"
          // No saved passwords for the key.
          autoComplete="off"
          label={t("Chiave API di Gemini (Antigravity senza account Google)")}
          value={typed}
          placeholder={view.geminiKeySet ? t("salvata: scrivi per sostituirla") : t("nessuna")}
          onChange={setTyped}
          onBlur={commit}
          className="w-full"
        />
        {view.geminiKeySet && (
          <div className="mt-1.5">
            <Link
              onClick={() => {
                setTyped("");
                onGemini("");
              }}
            >
              {t("Togli la chiave")}
            </Link>
          </div>
        )}
      </SettingItem>
    </SettingsGroup>
  );
}
