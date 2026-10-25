// The Gemini API key, Antigravity's sign-in without a Google account: on
// the agent's page of the AI page with the other sign-ins (D-165), no more
// in Settings. Saved apart from the settings and never shown; a key typed
// is saved when the field loses the focus, then the field empties.
import { useState } from "react";
import { settingsGet, settingsSet, type SettingsView } from "../ipc";
import { t } from "../i18n";
import { ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { Link, TextField } from "../workbench";

/** `key`: a new key, or "" to remove it. The settings go back as they are
 * on disk now, read again: the key is saved alone. */
function geminiSave(key: string): Promise<void> {
  return settingsGet().then((v) => settingsSet(v.settings, key));
}

export function GeminiKey({ onDone }: { onDone: () => void }) {
  const { data: view, setData: setView } = useLoad<SettingsView | null>(settingsGet, null, {
    onError: { fallback: null },
  });
  const { error, run } = useAction(onDone);
  const [typed, setTyped] = useState("");
  const saved = view?.geminiKeySet ?? false;
  const save = (key: string) => {
    setTyped("");
    run("gemini", () =>
      geminiSave(key).then(() => {
        setView((v) => (v ? { ...v, geminiKeySet: key.trim() !== "" } : v));
      }),
    );
  };
  return (
    <div className="flex flex-col gap-1.5">
      <TextField
        id="gemini-key"
        type="password"
        // No saved passwords for the key.
        autoComplete="off"
        label={t("Chiave API di Gemini (Antigravity senza account Google)")}
        value={typed}
        placeholder={saved ? t("salvata: scrivi per sostituirla") : t("nessuna")}
        onChange={setTyped}
        onBlur={() => {
          if (typed.trim() !== "") save(typed);
        }}
        className="w-full max-w-md"
      />
      <p className="text-[12px] text-vsc-desc">
        {t("Resta nel kit, in data/app/secrets, leggibile solo dal tuo utente, e arriva solo ai programmi del kit.")}
      </p>
      {saved && (
        <div>
          <Link
            onClick={() => {
              save("");
            }}
          >
            {t("Togli la chiave")}
          </Link>
        </div>
      )}
      <ErrorLine error={error} />
    </div>
  );
}
