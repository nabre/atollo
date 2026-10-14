// Adding or changing an action of a project, saved in its panel.json
// (DECISIONS.md D-78): name, command, type, folder, address and environment
// variables. A detected action is corrected by saving one with its id.
import { useState, type SyntheticEvent } from "react";
import { VscClose, VscSave } from "react-icons/vsc";
import { projectActionSave, type ActionEdit, type ActionKind, type ProjectAction } from "./api";
import { t } from "./i18n";
import { ActionButton, ErrorLine, useAction } from "./pages";

/** `NOME=valore` lines to variables; lines without `=` are ignored. */
export function parseEnv(text: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of text.split(/\r?\n/)) {
    const i = line.indexOf("=");
    if (i <= 0) continue;
    out[line.slice(0, i).trim()] = line.slice(i + 1).trim();
  }
  return out;
}

function slug(label: string): string {
  return (
    label
      .toLowerCase()
      .normalize("NFD")
      .replace(/[̀-ͯ]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 40) || "azione"
  );
}

export function ActionForm({
  project,
  action,
  onDone,
  onCancel,
}: {
  project: string;
  /** null: a new action. */
  action: ProjectAction | null;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [label, setLabel] = useState(action?.label ?? "");
  const [command, setCommand] = useState(action?.command ?? "");
  const [kind, setKind] = useState<ActionKind>(action?.type ?? "command");
  const [folder, setFolder] = useState(action?.folder ?? "");
  const [url, setUrl] = useState(action?.url ?? "");
  const [env, setEnv] = useState(
    Object.entries(action?.env ?? {})
      .map(([k, v]) => `${k}=${v}`)
      .join("\n"),
  );
  const { busy, error, run } = useAction(onDone);
  const id = action?.id ?? slug(label);
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    const edit: ActionEdit = {
      id,
      label,
      command,
      kind,
      folder: folder.trim() || null,
      url: url.trim() || null,
      env: parseEnv(env),
    };
    run("save", () => projectActionSave(project, edit));
  };
  const field = "w-full input";
  return (
    <form
      onSubmit={submit}
      aria-label={action ? t("Modifica {0}", action.label) : t("Nuova azione")}
      className="space-y-2 border-t border-vsc-border p-3"
    >
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Nome")}</span>
          <input
            className={field}
            value={label}
            required
            onChange={(e) => {
              setLabel(e.target.value);
            }}
          />
        </label>
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Tipo")}</span>
          <select
            className={field}
            value={kind}
            onChange={(e) => {
              setKind(e.target.value as ActionKind);
            }}
          >
            <option value="web">{t("sito web")}</option>
            <option value="command">{t("comando")}</option>
            <option value="desktop">{t("app con finestra")}</option>
          </select>
        </label>
      </div>
      <label className="block space-y-1">
        <span className="text-vsc-desc">{t("Comando")}</span>
        <input
          className={`${field} font-mono`}
          value={command}
          required
          spellCheck={false}
          onChange={(e) => {
            setCommand(e.target.value);
          }}
        />
        <span className="block text-[12px] text-vsc-desc">
          {t("Parte con cmd.exe nella cartella indicata. {port} diventa la porta dell'azione.")}
        </span>
      </label>
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Cartella (dentro il progetto)")}</span>
          <input
            className={`${field} font-mono`}
            value={folder}
            placeholder="."
            onChange={(e) => {
              setFolder(e.target.value);
            }}
          />
        </label>
        {kind === "web" && (
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Indirizzo da aprire (facoltativo)")}</span>
            <input
              className={`${field} font-mono`}
              value={url}
              placeholder="http://localhost:{port}/"
              onChange={(e) => {
                setUrl(e.target.value);
              }}
            />
          </label>
        )}
      </div>
      <label className="block space-y-1">
        <span className="text-vsc-desc">{t("Variabili d'ambiente, una per riga: NOME=valore")}</span>
        <textarea
          className={`${field} font-mono`}
          rows={3}
          value={env}
          spellCheck={false}
          onChange={(e) => {
            setEnv(e.target.value);
          }}
        />
      </label>
      <p className="text-[12px] text-vsc-desc">
        {t("Si salva in panel.json nella cartella del progetto: va nel repository con il resto del codice.")}
      </p>
      <div className="flex gap-2">
        <ActionButton
          icon={VscSave}
          label={t("Salva")}
          primary
          submit
          busy={busy === "save"}
          onClick={() => undefined}
        />
        <ActionButton icon={VscClose} label={t("Annulla")} onClick={onCancel} />
      </div>
      <ErrorLine error={error} />
    </form>
  );
}
