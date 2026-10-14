// Copies of a project's database (DECISIONS.md D-78): export into
// data\db\backup\<project>, restore one of them (a file from elsewhere is
// copied into that folder first: the app has no file picker, D-15).
import { useCallback, useEffect, useState } from "react";
import { VscCheck, VscCloudUpload, VscDiscard, VscFolderOpened } from "react-icons/vsc";
import { folderOpen, projectBackups, projectDbExport, projectDbRestore, type Backup } from "./api";
import { locale, t } from "./i18n";
import { ActionButton, ErrorLine, useAction } from "./pages";

function size(bytes: number): string {
  const kb = bytes / 1024;
  return kb < 1024
    ? `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 0 }).format(Math.max(1, kb))} KB`
    : `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(kb / 1024)} MB`;
}

export function DatabaseCopies({ project, engine }: { project: string; engine: Backup["engine"] }) {
  const [list, setList] = useState<Backup[]>([]);
  const [confirm, setConfirm] = useState<string | null>(null);
  const [done, setDone] = useState<string | null>(null);
  const load = useCallback(() => {
    projectBackups(project).then(
      (all) => {
        setList(all.filter((b) => b.engine === engine));
      },
      () => undefined,
    );
  }, [project, engine]);
  useEffect(load, [load]);
  const { busy, error, run } = useAction(load);
  return (
    <div className="space-y-2 border-t border-vsc-border px-3 py-2">
      <div className="flex flex-wrap items-center gap-2">
        <span className="min-w-0 flex-1 font-semibold">{t("Copie")}</span>
        <ActionButton
          icon={VscCloudUpload}
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
          icon={VscFolderOpened}
          label={t("Apri la cartella delle copie")}
          onClick={() => {
            void folderOpen("backup", project);
          }}
        />
      </div>
      {done && (
        <p role="status" className="flex items-center gap-1 text-vsc-success">
          <VscCheck size={14} aria-hidden="true" />
          {done}
        </p>
      )}
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
                <>
                  <span className="text-vsc-warning">{t("I dati attuali vengono sostituiti.")}</span>
                  <ActionButton
                    icon={VscDiscard}
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
                    icon={VscDiscard}
                    label={t("Annulla")}
                    onClick={() => {
                      setConfirm(null);
                    }}
                  />
                </>
              ) : (
                <ActionButton
                  icon={VscDiscard}
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
