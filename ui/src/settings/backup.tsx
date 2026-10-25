// «Copia del kit» (report v6/04, N-12, D-163): the README's backup rule done
// by the app. Export: projects\ and data\ without what is downloaded again,
// the databases stopped for a coherent copy. Import: the copy is checked and
// applied at the next start; what it replaces is kept aside.
import { useState } from "react";
import {
  appQuit,
  folderOpen,
  kitExport,
  kitImport,
  kitImportCancel,
  kitImportPending,
  type KitCopy,
  type KitExported,
} from "../ipc";
import { ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { locale, t } from "../i18n";
import { formatLargeBytes } from "../shared/format";
import { Button, Notice, SettingItem } from "../workbench";

export function KitBackup() {
  const { data: pending, setData: setPending } = useLoad<boolean>(kitImportPending, false, { onError: "keep" });
  const [exported, setExported] = useState<KitExported | null>(null);
  const [staged, setStaged] = useState<KitCopy | null>(null);
  const { busy, error, run } = useAction();
  const doExport = (zip: boolean) => {
    run(zip ? "zip" : "folder", async () => {
      const made = await kitExport(zip);
      if (made) setExported(made);
    });
  };
  const doImport = (zip: boolean) => {
    run(zip ? "import-zip" : "import-folder", async () => {
      const copy = await kitImport(zip);
      if (copy) {
        setStaged(copy);
        setPending(true);
      }
    });
  };
  return (
    <SettingItem
      title={t("Copia del kit")}
      description={t(
        "Progetti e dati (impostazioni, database, agenti) in una cartella o in uno zip, senza i programmi, le cache e i modelli locali, che si riscaricano. I database si fermano per la copia. La copia contiene le password del kit: tienila al sicuro.",
      )}
      keywords="backup copia esporta importa zip export import restore ripristina"
    >
      <div className="flex flex-wrap gap-2">
        <Button
          busy={busy === "folder"}
          disabled={busy !== null}
          onClick={() => {
            doExport(false);
          }}
        >
          {t("Esporta in una cartella…")}
        </Button>
        <Button
          busy={busy === "zip"}
          disabled={busy !== null}
          onClick={() => {
            doExport(true);
          }}
        >
          {t("Esporta in uno zip…")}
        </Button>
        <Button
          busy={busy === "import-folder"}
          disabled={busy !== null}
          onClick={() => {
            doImport(false);
          }}
        >
          {t("Importa da una cartella…")}
        </Button>
        <Button
          busy={busy === "import-zip"}
          disabled={busy !== null}
          onClick={() => {
            doImport(true);
          }}
        >
          {t("Importa da uno zip…")}
        </Button>
      </div>
      {exported && (
        <p className="mt-2">
          {t("Copia fatta: {0} file, {1}.", String(exported.files), formatLargeBytes(exported.bytes))}{" "}
          <span className="font-mono text-[12px] break-all select-text">{exported.path}</span>
        </p>
      )}
      {pending && (
        <Notice tone="warning" className="mt-2">
          <div className="space-y-2">
            <p>
              {staged
                ? t(
                    "Copia pronta: {0} file, {1}, fatta il {2}.",
                    String(staged.files),
                    formatLargeBytes(staged.bytes),
                    new Date(staged.created * 1000).toLocaleString(locale()),
                  )
                : t("Una copia aspetta di essere importata.")}{" "}
              {t(
                "Al prossimo avvio prende il posto dei progetti e dei dati di questo kit; quelli di adesso vanno nella cartella .atollo-before-import, non si cancellano. I programmi che mancano si riscaricano.",
              )}
            </p>
            <div className="flex flex-wrap gap-2">
              <Button
                variant="primary"
                disabled={busy !== null}
                onClick={() => {
                  run("quit", () => appQuit());
                }}
              >
                {t("Chiudi Atollo per importarla")}
              </Button>
              <Button
                disabled={busy !== null}
                onClick={() => {
                  run("cancel", async () => {
                    await kitImportCancel();
                    setPending(false);
                    setStaged(null);
                  });
                }}
              >
                {t("Annulla l'importazione")}
              </Button>
              <Button
                disabled={busy !== null}
                onClick={() => {
                  run("open", () => folderOpen("kit"));
                }}
              >
                {t("Apri la cartella del kit")}
              </Button>
            </div>
            <p className="text-[12px] text-vsc-desc">{t("Poi riapri Atollo: la copia si applica all'avvio.")}</p>
          </div>
        </Notice>
      )}
      <ErrorLine error={error} />
    </SettingItem>
  );
}
