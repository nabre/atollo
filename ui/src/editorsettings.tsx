// A project's editor settings, in its .vscode\settings.json (DECISIONS.md
// D-78): format on save, indentation, folders hidden in the editor.
import { useCallback, useEffect, useState, type SyntheticEvent } from "react";
import { VscSave } from "react-icons/vsc";
import { projectEditorSettings, projectEditorSettingsSet, type EditorSettingsView } from "./api";
import { t } from "./i18n";
import { ActionButton, ErrorLine } from "./common";
import { useAction } from "./hooks";

type Tri = "default" | "yes" | "no";
const toTri = (v: boolean | null): Tri => (v === null ? "default" : v ? "yes" : "no");
const fromTri = (v: Tri): boolean | null => (v === "default" ? null : v === "yes");

export function EditorSettingsBox({ project }: { project: string }) {
  const [view, setView] = useState<EditorSettingsView | null>(null);
  const [format, setFormat] = useState<Tri>("default");
  const [indent, setIndent] = useState("default");
  const [hidden, setHidden] = useState("");
  const [saved, setSaved] = useState(false);
  const load = useCallback(() => {
    projectEditorSettings(project).then(
      (v) => {
        setView(v);
        setFormat(toTri(v.settings.formatOnSave));
        setIndent(
          v.settings.insertSpaces === false
            ? "tab"
            : v.settings.tabSize !== null
              ? String(v.settings.tabSize)
              : "default",
        );
        setHidden(v.settings.hidden.join("\n"));
      },
      () => undefined,
    );
  }, [project]);
  useEffect(load, [load]);
  const { busy, error, run } = useAction(() => {
    setSaved(true);
    load();
  });
  if (!view) return null;
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    setSaved(false);
    run("save", () =>
      projectEditorSettingsSet(project, {
        formatOnSave: fromTri(format),
        tabSize: indent === "default" || indent === "tab" ? null : Number(indent),
        insertSpaces: indent === "default" ? null : indent !== "tab",
        hidden: hidden
          .split(/\r?\n/)
          .map((h) => h.trim())
          .filter(Boolean),
      }),
    );
  };
  const field = "input";
  return (
    <section aria-labelledby="editor-settings" className="border-t border-vsc-border">
      <h2 id="editor-settings" className="flex h-8 items-center px-3 text-[12px] font-semibold text-vsc-sidebar-title">
        {t("Impostazioni dell'editor per questo progetto")}
      </h2>
      {view.readOnly ? (
        <p className="px-3 py-2 text-vsc-desc">
          {t(
            ".vscode\\settings.json ha commenti o non è JSON semplice: NEXKORA non lo riscrive. Modificalo nell'editor.",
          )}
        </p>
      ) : (
        <form onSubmit={submit} className="space-y-2 px-3 py-2">
          <div className="flex flex-wrap gap-4">
            <label className="flex items-center gap-2">
              {t("Formatta al salvataggio")}
              <select
                className={field}
                value={format}
                onChange={(e) => {
                  setFormat(e.target.value as Tri);
                }}
              >
                <option value="default">{t("come l'editor")}</option>
                <option value="yes">{t("sì")}</option>
                <option value="no">{t("no")}</option>
              </select>
            </label>
            <label className="flex items-center gap-2">
              {t("Rientro")}
              <select
                className={field}
                value={indent}
                onChange={(e) => {
                  setIndent(e.target.value);
                }}
              >
                <option value="default">{t("come l'editor")}</option>
                <option value="2">{t("2 spazi")}</option>
                <option value="4">{t("4 spazi")}</option>
                <option value="tab">{t("tabulazione")}</option>
              </select>
            </label>
          </div>
          <label className="block space-y-1">
            <span className="text-vsc-desc">{t("Cartelle nascoste nell'editor, una per riga (es. **/vendor)")}</span>
            <textarea
              className={`${field} w-full font-mono`}
              rows={3}
              value={hidden}
              spellCheck={false}
              onChange={(e) => {
                setHidden(e.target.value);
              }}
            />
          </label>
          <p className="text-[12px] text-vsc-desc">
            {t("Si salvano in .vscode\\settings.json nel progetto; le altre impostazioni del file restano.")}
          </p>
          <div className="flex items-center gap-2">
            <ActionButton
              icon={VscSave}
              label={t("Salva")}
              primary
              submit
              busy={busy === "save"}
              onClick={() => undefined}
            />
            {saved && <span className="text-vsc-success">{t("Salvato.")}</span>}
          </div>
          <ErrorLine error={error} />
        </form>
      )}
    </section>
  );
}
