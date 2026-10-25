// The form of an action of Avvio (D-78): a new one, or one changed, written
// in the project's panel.json. A detected action saved with its own id is
// corrected by it (the one in panel.json takes its place).
import { useState, type SyntheticEvent } from "react";
import { projectActionSave, type ActionEdit, type ActionKind, type ProjectAction, type ProjectInfo } from "../ipc";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction } from "../shared/hooks";
import { t } from "../i18n";
import { codicon, Select, TextArea, TextField } from "../workbench";

/** The id of a new action from its name, as `valid_id` (detect/overrides.rs)
 * accepts it: lowercase letters, digits and `-_:.`, a letter or a digit
 * first, 40 at most; `-2`, `-3`… when the project has it already. */
export function actionId(name: string, taken: string[]): string {
  const base =
    name
      .normalize("NFD")
      .replace(/[̀-ͯ]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9_:.-]+/g, "-")
      .replace(/^[^a-z0-9]+/, "")
      .slice(0, 40)
      .replace(/-+$/, "") || "azione";
  if (!taken.includes(base)) return base;
  for (let n = 2; ; n++) {
    const suffix = `-${String(n)}`;
    const id = `${base.slice(0, 40 - suffix.length).replace(/-+$/, "")}${suffix}`;
    if (!taken.includes(id)) return id;
  }
}

/** `NAME=value`, one per line, as the env of an action; or the number of
 * the first line without a name before `=`. */
function parseEnv(text: string): { env: Record<string, string> } | { bad: number } {
  const env: Record<string, string> = {};
  const lines = text.split(/\r?\n/);
  for (const [i, line] of lines.entries()) {
    if (line.trim() === "") continue;
    const at = line.indexOf("=");
    const key = at < 0 ? "" : line.slice(0, at).trim();
    if (key === "") return { bad: i + 1 };
    env[key] = line.slice(at + 1);
  }
  return { env };
}

/** «Nuova azione» (no `action`) or «Modifica» of `action`. */
export function ActionForm({
  p,
  action,
  onSaved,
  onCancel,
}: {
  p: ProjectInfo;
  action?: ProjectAction | undefined;
  onSaved: (id: string) => void;
  onCancel: () => void;
}) {
  const [label, setLabel] = useState(action?.label ?? "");
  const [kind, setKind] = useState<ActionKind>(action?.type ?? "web");
  const [command, setCommand] = useState(action?.command ?? "");
  const [folder, setFolder] = useState(action?.folder ?? "");
  const [url, setUrl] = useState(action?.url ?? "");
  const [envText, setEnvText] = useState(
    Object.entries(action?.env ?? {})
      .map(([k, v]) => `${k}=${v}`)
      .join("\n"),
  );
  const [invalid, setInvalid] = useState<string | null>(null);
  const id =
    action?.id ??
    actionId(
      label,
      p.actions.map((a) => a.id),
    );
  const { busy, error, run } = useAction(() => {
    onSaved(id);
  });
  // Name and command are needed: Salva waits for them, as the MCP form.
  const complete = label.trim() !== "" && command.trim() !== "";
  const submit = (e: SyntheticEvent) => {
    e.preventDefault();
    if (!complete) return;
    const parsed = parseEnv(envText);
    if ("bad" in parsed) {
      setInvalid(t("Riga {0} delle variabili: scrivi NOME=valore.", parsed.bad));
      return;
    }
    setInvalid(null);
    const edit: ActionEdit = {
      id,
      label: label.trim(),
      command: command.trim(),
      kind,
      folder: folder.trim() === "" ? null : folder.trim(),
      url: kind === "web" && url.trim() !== "" ? url.trim() : null,
      env: parsed.env,
    };
    run("save", () => projectActionSave(p.id, edit));
  };
  return (
    <form
      onSubmit={submit}
      aria-label={action ? t("Modifica {0}", action.label) : t("Nuova azione")}
      className="space-y-2 rounded border border-vsc-border px-3 py-2"
    >
      {action && !action.custom && (
        <p className="text-[12px] text-vsc-desc">
          {t("Azione rilevata: salvata, la tua versione in panel.json prende il suo posto.")}
        </p>
      )}
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Nome")}</span>
          <TextField label={t("Nome")} value={label} onChange={setLabel} autoFocus className="w-full" />
        </label>
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Tipo")}</span>
          <Select<ActionKind>
            label={t("Tipo")}
            value={kind}
            onChange={setKind}
            options={[
              { value: "web", label: t("Sito o server web") },
              { value: "desktop", label: t("App con finestra") },
              { value: "command", label: t("Comando") },
            ]}
            className="w-full"
          />
        </label>
      </div>
      <label className="block space-y-1">
        <span className="text-vsc-desc">{t("Comando")}</span>
        <TextField
          mono
          label={t("Comando")}
          placeholder="npm run dev -- --port {port}"
          value={command}
          onChange={setCommand}
          className="w-full"
        />
        <span className="block text-[12px] text-vsc-desc">{t("{port} nel comando diventa la porta dell'azione.")}</span>
      </label>
      <div className="grid gap-2 sm:grid-cols-2">
        <label className="space-y-1">
          <span className="text-vsc-desc">{t("Cartella")}</span>
          <TextField
            mono
            label={t("Cartella")}
            placeholder={t("facoltativa, relativa al progetto")}
            value={folder}
            onChange={setFolder}
            className="w-full"
          />
        </label>
        {kind === "web" && (
          <label className="space-y-1">
            <span className="text-vsc-desc">{t("Indirizzo")}</span>
            <TextField
              mono
              label={t("Indirizzo")}
              placeholder="http://localhost:{port}"
              value={url}
              onChange={setUrl}
              className="w-full"
            />
          </label>
        )}
      </div>
      <label className="block space-y-1">
        <span className="text-vsc-desc">{t("Variabili")}</span>
        <TextArea
          mono
          rows={2}
          spellCheck={false}
          label={t("Variabili")}
          placeholder={t("facoltative, NOME=valore una per riga")}
          value={envText}
          onChange={setEnvText}
          className="w-full"
        />
      </label>
      {!action && label.trim() !== "" && <p className="text-[12px] text-vsc-desc">{t("Id: {0}", id)}</p>}
      <div className="flex gap-2">
        <ActionButton
          icon={codicon("save")}
          label={t("Salva")}
          primary
          submit
          disabled={!complete}
          busy={busy === "save"}
          onClick={() => undefined}
        />
        <ActionButton icon={codicon("close")} label={t("Annulla")} onClick={onCancel} />
      </div>
      <ErrorLine error={invalid ?? error} />
    </form>
  );
}
