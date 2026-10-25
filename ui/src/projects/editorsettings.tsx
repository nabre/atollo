// A project's editor settings, in its .vscode\settings.json (DECISIONS.md
// D-78): format on save, indentation, folders hidden in the editor.
import { useCallback, useState, type SyntheticEvent } from "react";
import { projectEditorSettings, projectEditorSettingsSet, type EditorSettings, type EditorSettingsView } from "../ipc";
import { t } from "../i18n";
import { ActionButton, ErrorLine, LoadError, useRetryFocus } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { codicon, Select, TextArea } from "../workbench";

type Tri = "default" | "yes" | "no";
const toTri = (v: boolean | null): Tri => (v === null ? "default" : v ? "yes" : "no");
const fromTri = (v: Tri): boolean | null => (v === "default" ? null : v === "yes");

export function EditorSettingsBox({ project }: { project: string }) {
  const read = useCallback(() => projectEditorSettings(project), [project]);
  const {
    data: view,
    error: loadErr,
    reload,
    retry,
  } = useLoad<EditorSettingsView | null>(read, null, { onError: "show" });
  // The project whose settings were just saved: the form, made again from
  // the file read after, says so and keeps the focus on Salva.
  const [savedFor, setSavedFor] = useState<string | null>(null);
  const saved = savedFor === project;
  const { busy, error, run } = useAction(() => {
    setSavedFor(project);
    reload();
  });
  const { target: heading, onRetry } = useRetryFocus<HTMLHeadingElement>(retry);
  const loadError = <LoadError error={loadErr} onRetry={onRetry} className="px-3 py-2" />;
  if (!view) return loadError;
  return (
    <section aria-labelledby="editor-settings" className="border-t border-vsc-border">
      <h2
        id="editor-settings"
        ref={heading}
        tabIndex={-1}
        className="flex h-8 items-center px-3 text-[12px] font-semibold text-vsc-sidebar-title"
      >
        {t("Impostazioni dell'editor per questo progetto")}
      </h2>
      {view.readOnly ? (
        <p className="px-3 py-2 text-vsc-desc">
          {t(
            ".vscode\\settings.json ha commenti o non è JSON semplice: Atollo non lo riscrive. Modificalo nell'editor.",
          )}
        </p>
      ) : (
        <SettingsForm
          // A new key when the file changes: the fields start from it.
          key={`${project}\n${JSON.stringify(view.settings)}`}
          settings={view.settings}
          busy={busy === "save"}
          saved={saved}
          error={error}
          onSave={(settings) => {
            setSavedFor(null);
            run("save", () => projectEditorSettingsSet(project, settings));
          }}
        />
      )}
      {loadError}
    </section>
  );
}

function SettingsForm({
  settings,
  busy,
  saved,
  error,
  onSave,
}: {
  settings: EditorSettings;
  busy: boolean;
  saved: boolean;
  error: string | null;
  onSave: (settings: EditorSettings) => void;
}) {
  const [format, setFormat] = useState<Tri>(toTri(settings.formatOnSave));
  const [indent, setIndent] = useState(
    settings.insertSpaces === false ? "tab" : settings.tabSize !== null ? String(settings.tabSize) : "default",
  );
  const [hidden, setHidden] = useState(settings.hidden.join("\n"));
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    onSave({
      formatOnSave: fromTri(format),
      tabSize: indent === "default" || indent === "tab" ? null : Number(indent),
      insertSpaces: indent === "default" ? null : indent !== "tab",
      hidden: hidden
        .split(/\r?\n/)
        .map((h) => h.trim())
        .filter(Boolean),
    });
  };
  return (
    <form onSubmit={submit} className="space-y-2 px-3 py-2">
      <div className="flex flex-wrap gap-4">
        <label className="flex items-center gap-2">
          {t("Formatta al salvataggio")}
          <Select
            value={format}
            onChange={setFormat}
            options={[
              { value: "default", label: t("come l'editor") },
              { value: "yes", label: t("sì") },
              { value: "no", label: t("no") },
            ]}
            className="w-44"
          />
        </label>
        <label className="flex items-center gap-2">
          {t("Rientro")}
          <Select
            value={indent}
            onChange={setIndent}
            options={[
              { value: "default", label: t("come l'editor") },
              { value: "2", label: t("2 spazi") },
              { value: "4", label: t("4 spazi") },
              { value: "tab", label: t("tabulazione") },
            ]}
            className="w-44"
          />
        </label>
      </div>
      <label className="block space-y-1">
        <span className="text-vsc-desc">{t("Cartelle nascoste nell'editor, una per riga (es. **/vendor)")}</span>
        <TextArea mono rows={3} value={hidden} spellCheck={false} onChange={setHidden} className="w-full" />
      </label>
      <p className="text-[12px] text-vsc-desc">
        {t("Si salvano in .vscode\\settings.json nel progetto; le altre impostazioni del file restano.")}
      </p>
      <div className="flex items-center gap-2">
        <ActionButton
          icon={codicon("save")}
          label={t("Salva")}
          primary
          submit
          busy={busy}
          // Made again after a save (a new key): the focus stays on it.
          autoFocus={saved}
          onClick={() => undefined}
        />
        {saved && <span className="text-vsc-success">{t("Salvato.")}</span>}
      </div>
      <ErrorLine error={error} />
    </form>
  );
}
