// «Salva come base» (report v6/04, N-10, D-162): the project's files as a
// base of the new-project wizard, under «Le mie basi». What Git would commit
// (.gitignore respected), never .env; the top folders and files can be left
// out one by one.
import { useCallback, useState } from "react";
import { projectBasePreview, projectSaveBase, type BaseEntry } from "../api";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { formatBytes } from "../shared/format";
import { CheckboxLabel, codicon, ConfirmDialog, Notice, TextField } from "../workbench";

export function SaveBaseDialog({
  project,
  onClose,
  onSaved,
}: {
  project: string;
  onClose: () => void;
  /** The bases changed: the wizard reads them again. */
  onSaved?: () => void;
}) {
  const read = useCallback(() => projectBasePreview(project), [project]);
  const { data: entries, error: readError } = useLoad<BaseEntry[] | null>(read, null, { onError: "show" });
  const [name, setName] = useState(project);
  const [desc, setDesc] = useState("");
  const [skip, setSkip] = useState<Set<string>>(new Set());
  const [saved, setSaved] = useState(false);
  const { busy, error, run } = useAction(onSaved);
  const kept = (entries ?? []).filter((e) => !skip.has(e.name));
  const files = kept.reduce((n, e) => n + e.files, 0);
  const bytes = kept.reduce((n, e) => n + e.bytes, 0);
  return (
    <ConfirmDialog
      title={t("Salva {0} come base", project)}
      cancelLabel={saved ? t("Chiudi") : t("Annulla")}
      onCancel={onClose}
      actions={
        saved ? null : (
          <ActionButton
            icon={codicon("save")}
            label={t("Salva come base")}
            primary
            busy={busy === "save"}
            disabled={entries === null || files === 0 || name.trim() === "" || busy !== null}
            onClick={() => {
              run("save", async () => {
                await projectSaveBase(project, name.trim(), desc.trim(), [...skip]);
                setSaved(true);
              });
            }}
          />
        )
      }
      footer={<ErrorLine error={error ?? readError} />}
    >
      {saved ? (
        <Notice tone="success">
          {t("Salvata: la trovi in Nuovo progetto › Le mie basi. Con lo stesso nome la si sostituisce.")}
        </Notice>
      ) : (
        <div className="space-y-3">
          <p className="text-vsc-desc">
            {t(
              "I file che Git terrebbe (.gitignore rispettato; senza Git, tutto tranne node_modules, vendor, .venv e simili), mai .env: un nuovo progetto da questa base parte da una loro copia.",
            )}
          </p>
          <div className="grid gap-2 sm:grid-cols-2">
            <label className="space-y-1">
              <span className="text-vsc-desc">{t("Nome della base")}</span>
              <TextField value={name} onChange={setName} className="w-full" />
            </label>
            <label className="space-y-1">
              <span className="text-vsc-desc">{t("Descrizione")}</span>
              <TextField value={desc} onChange={setDesc} className="w-full" />
            </label>
          </div>
          {entries === null ? (
            <p className="text-vsc-desc">{t("Caricamento…")}</p>
          ) : entries.length === 0 ? (
            <p className="text-vsc-desc">{t("Nessun file da salvare.")}</p>
          ) : (
            <fieldset className="space-y-1">
              <legend className="mb-1 font-semibold">{t("Cosa copiare")}</legend>
              <ul className="m-0 max-h-56 list-none space-y-0.5 overflow-y-auto p-0">
                {entries.map((e) => (
                  <li key={e.name} className="flex items-center gap-2">
                    <CheckboxLabel
                      checked={!skip.has(e.name)}
                      onChange={(on) => {
                        setSkip((s) => {
                          const next = new Set(s);
                          if (on) next.delete(e.name);
                          else next.add(e.name);
                          return next;
                        });
                      }}
                    >
                      <span className="font-mono">{e.dir ? `${e.name}/` : e.name}</span>
                    </CheckboxLabel>
                    <span className="flex-1" />
                    <span className="text-[12px] text-vsc-desc">
                      {t("{0} file", String(e.files))} · {formatBytes(e.bytes)}
                    </span>
                  </li>
                ))}
              </ul>
              <p className="text-[12px] text-vsc-desc">
                {t("In tutto: {0} file, {1}", String(files), formatBytes(bytes))}
              </p>
            </fieldset>
          )}
        </div>
      )}
    </ConfirmDialog>
  );
}
