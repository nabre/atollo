// Copies of a project's database (DECISIONS.md D-78): export into
// data\projects\<project>\private\backups, restore one of them (a file from elsewhere is
// copied into that folder first: the app has no file picker, D-15).
import { useCallback, useState } from "react";
import { folderOpen, projectBackups, projectDbExport, projectDbRestore, type Backup } from "../api";
import { locale, t } from "../i18n";
import { ActionButton, ErrorLine } from "../common";
import { useAction, useLoad } from "../hooks";
import { codicon, InlineConfirm, Notice } from "../workbench";

function size(bytes: number): string {
  const kb = bytes / 1024;
  return kb < 1024
    ? `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 0 }).format(Math.max(1, kb))} KB`
    : `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(kb / 1024)} MB`;
}

export function DatabaseCopies({ project, engine }: { project: string; engine: Backup["engine"] }) {
  const [confirm, setConfirm] = useState<string | null>(null);
  const [done, setDone] = useState<string | null>(null);
  const read = useCallback(
    () => projectBackups(project).then((all) => all.filter((b) => b.engine === engine)),
    [project, engine],
  );
  const { data: list, reload: load } = useLoad<Backup[]>(read, [], { onError: "keep" });
  const { busy, error, run } = useAction(load);
  return (
    <div className="space-y-2 border-t border-vsc-border px-3 py-2">
      <div className="flex flex-wrap items-center gap-2">
        <span className="min-w-0 flex-1 font-semibold">{t("Copie")}</span>
        <ActionButton
          icon={codicon("cloud-upload")}
          label={t("Esporta")}
          busy={busy === "export"}
          onClick={() => {
            setDone(null);
            run("export", async () => {
              const file = await projectDbExport(project, engine);
              setDone(t("Copia salvata: {0}", file));
            });
          }}
        />
        <ActionButton
          icon={codicon("folder-opened")}
          label={t("Apri la cartella delle copie")}
          onClick={() => {
            void folderOpen("backup", project);
          }}
        />
      </div>
      {done && <Notice tone="success">{done}</Notice>}
      {list.length === 0 ? (
        <p className="text-[12px] text-vsc-desc">
          {t("Nessuna copia. Per ripristinare un file da fuori, copialo nella cartella delle copie.")}
        </p>
      ) : (
        <ul className="divide-y divide-vsc-border border-t border-vsc-border">
          {list.map((b) => (
            <li key={b.file} className="flex flex-wrap items-center gap-2 px-2 py-1">
              <span className="min-w-0 flex-1 truncate font-mono text-[12px]">{b.file}</span>
              <span className="text-[12px] text-vsc-desc">{size(b.size)}</span>
              {confirm === b.file ? (
                <InlineConfirm
                  row
                  question={t("Ripristinare {0}?", b.file)}
                  textClassName="text-vsc-warning"
                  onCancel={() => {
                    setConfirm(null);
                  }}
                  actions={
                    <>
                      <ActionButton
                        icon={codicon("discard")}
                        label={t("Ripristina")}
                        primary
                        busy={busy === `restore:${b.file}`}
                        onClick={() => {
                          setConfirm(null);
                          setDone(null);
                          run(`restore:${b.file}`, async () => {
                            await projectDbRestore(project, b.file);
                            setDone(t("Ripristinato da {0}", b.file));
                          });
                        }}
                      />
                      <ActionButton
                        icon={codicon("discard")}
                        label={t("Annulla")}
                        onClick={() => {
                          setConfirm(null);
                        }}
                      />
                    </>
                  }
                >
                  {t("I dati attuali vengono sostituiti.")}
                </InlineConfirm>
              ) : (
                <ActionButton
                  icon={codicon("discard")}
                  label={t("Ripristina…")}
                  onClick={() => {
                    setConfirm(b.file);
                  }}
                />
              )}
            </li>
          ))}
        </ul>
      )}
      <ErrorLine error={error} />
    </div>
  );
}
