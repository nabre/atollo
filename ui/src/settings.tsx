// Settings page (data\app\settings.json, src-tauri/src/settings.rs): language,
// network (proxy, certificates, local network), Gemini key, versions of
// Python, PHP and Rust, and the app's updates. Replaces nabre/atollo's
// proxy.cmd edited by hand.
import { useCallback, useEffect, useState, type SyntheticEvent, type ReactNode } from "react";
import { VscCheck, VscCloudDownload, VscDebugRestart, VscSave } from "react-icons/vsc";
import {
  editorRestart,
  settingsGet,
  settingsSet,
  updateCheck,
  updateStatus,
  type Settings,
  type SettingsView,
  type UpdateStatus,
} from "./api";
import { ActionButton, ErrorLine, useAction } from "./pages";
import { LANGS, t } from "./i18n";

const empty = (s: string) => (s.trim() === "" ? null : s.trim());

export function SettingsPage({ onSaved, editorRunning }: { onSaved: () => void; editorRunning: boolean }) {
  const [view, setView] = useState<SettingsView | null>(null);
  const [s, setS] = useState<Settings | null>(null);
  const [gemini, setGemini] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const { busy, error, run } = useAction();

  const load = useCallback(() => {
    settingsGet().then(
      (v) => {
        setView(v);
        setS(v.settings);
        setGemini(null);
      },
      (e: unknown) => {
        setLoadError(String(e));
      },
    );
  }, []);
  useEffect(load, [load]);

  if (loadError) return <ErrorLine error={loadError} />;
  if (!s || !view) return <p className="text-vsc-desc">{t("Caricamento…")}</p>;

  const net = (patch: Partial<Settings["network"]>) => {
    setSaved(false);
    setS({ ...s, network: { ...s.network, ...patch } });
  };
  const set = (patch: Partial<Settings>) => {
    setSaved(false);
    setS({ ...s, ...patch });
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

  return (
    <form onSubmit={submit} className="max-w-3xl space-y-6">
      {view.problem && (
        <ErrorLine
          error={t(
            "Le impostazioni in {0} non sono leggibili: uso quelle predefinite e non modifico il file.",
            view.problem,
          )}
        />
      )}
      <Section id="s-general" title={t("Generale")}>
        <Field label={t("Lingua")}>
          <select
            value={s.lang ?? ""}
            onChange={(e) => {
              set({ lang: (empty(e.target.value) as Settings["lang"]) ?? null });
            }}
            className="input"
          >
            <option value="">{t("Lingua di Windows")}</option>
            {LANGS.map((l) => (
              <option key={l.id} value={l.id}>
                {l.name}
              </option>
            ))}
          </select>
        </Field>
        <Check
          checked={s.autoUpdate}
          onChange={(v) => {
            set({ autoUpdate: v });
          }}
          label={t("Scarica da solo le nuove versioni di Atollo (sostituiscono l'exe alla chiusura)")}
        />
      </Section>

      <Section id="s-network" title={t("Rete")}>
        <p className="text-vsc-desc">
          {t(
            "Solo se la rete passa da un proxy o ispeziona il traffico HTTPS: valgono per download, agenti e strumenti del kit.",
          )}
        </p>
        <Field label={t("Proxy HTTPS")}>
          <input
            value={s.network.httpsProxy ?? ""}
            placeholder="http://proxy.azienda.it:8080"
            onChange={(e) => {
              net({ httpsProxy: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Field label={t("Proxy HTTP")}>
          <input
            value={s.network.httpProxy ?? ""}
            onChange={(e) => {
              net({ httpProxy: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Field label={t("Senza proxy per")}>
          <input
            value={s.network.noProxy ?? ""}
            placeholder="localhost,127.0.0.1,.azienda.it"
            onChange={(e) => {
              net({ noProxy: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Field label={t("Certificati aggiuntivi (file PEM)")}>
          <input
            value={s.network.extraCaCerts ?? ""}
            placeholder="E:\certificati\azienda.pem"
            onChange={(e) => {
              net({ extraCaCerts: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Check
          checked={s.network.allowLan}
          onChange={(v) => {
            net({ allowLan: v });
          }}
          label={t("Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete")}
        />
      </Section>

      <Section id="s-agents" title={t("Agenti")}>
        <div className="flex flex-col gap-1">
          <label htmlFor="gemini-key" className="text-[12px] text-vsc-desc">
            {t("Chiave API di Gemini (Antigravity senza account Google)")}
          </label>
          <div className="flex flex-wrap items-center gap-2">
            <input
              id="gemini-key"
              type="password"
              autoComplete="off"
              value={gemini ?? ""}
              placeholder={view.geminiKeySet ? t("salvata: scrivi per sostituirla") : t("nessuna")}
              onChange={(e) => {
                setSaved(false);
                setGemini(e.target.value === "" ? null : e.target.value);
              }}
              className="input"
            />
            {view.geminiKeySet && (
              <button
                type="button"
                className="text-vsc-link underline"
                onClick={() => {
                  setSaved(false);
                  setGemini("");
                }}
              >
                {t("Togli la chiave")}
              </button>
            )}
          </div>
          {gemini === "" && <p className="text-vsc-warning">{t("La chiave verrà tolta al salvataggio.")}</p>}
        </div>
      </Section>

      <Section id="s-langs" title={t("Versioni")}>
        <p className="text-vsc-desc">
          {t("Valgono dalla prossima installazione o aggiornamento in Linguaggi e strumenti.")}
        </p>
        <Field label={t("Python (vuoto = quella del kit)")}>
          <input
            value={s.pythonVersion ?? ""}
            placeholder="3.13"
            onChange={(e) => {
              set({ pythonVersion: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Field label={t("PHP (vuoto = la più recente)")}>
          <input
            value={s.phpVersion ?? ""}
            placeholder="8.4"
            onChange={(e) => {
              set({ phpVersion: empty(e.target.value) });
            }}
            className="input"
          />
        </Field>
        <Field label={t("Compilatore di Rust")}>
          <select
            value={s.rustToolchain ?? ""}
            onChange={(e) => {
              set({ rustToolchain: (empty(e.target.value) as Settings["rustToolchain"]) ?? null });
            }}
            className="input"
          >
            <option value="">{t("GNU (predefinito, tutto nel kit)")}</option>
            <option value="gnu">GNU</option>
            <option value="msvc">{t("MSVC (richiede Visual Studio Build Tools nel PC)")}</option>
          </select>
        </Field>
      </Section>

      <div className="flex flex-wrap items-center gap-3">
        <ActionButton
          icon={VscSave}
          label={t("Salva")}
          primary
          submit
          busy={busy === "save"}
          onClick={() => undefined}
        />
        {saved && (
          <span role="status" className="flex items-center gap-1 text-vsc-success">
            <VscCheck size={16} aria-hidden="true" />
            {editorRunning
              ? t("Salvato. Riavvia l'editor perché i suoi terminali usino le nuove impostazioni.")
              : t("Salvato.")}
          </span>
        )}
        {saved && editorRunning && (
          <ActionButton
            icon={VscDebugRestart}
            label={t("Riavvia l'editor")}
            busy={busy === "restart"}
            onClick={() => {
              run("restart", editorRestart);
            }}
          />
        )}
      </div>
      <ErrorLine error={error} />
      <Updates />
    </form>
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
    <Section id="s-update" title={t("Aggiornamenti di Atollo")}>
      <p role="status">{text}</p>
      <ActionButton
        icon={VscCloudDownload}
        label={t("Cerca aggiornamenti")}
        busy={busy === "check" || st.checking}
        onClick={() => {
          run("check", async () => {
            setSt(await updateCheck());
          });
        }}
      />
      <ErrorLine error={error} />
    </Section>
  );
}

function Section({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="space-y-3">
      <h2 id={id} className="border-b border-vsc-border pb-1 text-[13px] font-semibold">
        {title}
      </h2>
      {children}
    </section>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-[12px] text-vsc-desc">{label}</span>
      {children}
    </label>
  );
}

function Check({ checked, onChange, label }: { checked: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <label className="flex items-start gap-2">
      <input
        type="checkbox"
        className="mt-0.5"
        checked={checked}
        onChange={(e) => {
          onChange(e.target.checked);
        }}
      />
      <span>{label}</span>
    </label>
  );
}
