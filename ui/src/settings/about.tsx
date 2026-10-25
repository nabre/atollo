// The sections of Settings about Atollo itself: its updates, where the kit
// is, what it sends over the network (docs/PRIVACY.md).
import { useState } from "react";
import {
  diagnosticsExport,
  folderOpen,
  issueOpen,
  kitHealth,
  privacyOpen,
  updateCheck,
  updateStatus,
  type UpdateStatus,
  type AppInfo,
  type HealthFinding,
} from "../ipc";
import { BrandSymbol } from "../shared/brand";
import { useAction, useLoad } from "../shared/hooks";
import { ErrorLine } from "../shared/common";
import { t } from "../i18n";
import { updateText } from "../shell/update";
import { KitBackup } from "./backup";
import { Button, Dot, Link, Properties, SettingItem, SettingsGroup } from "../workbench";

/** Where the kit is (was on the Home, D-76). */
export function KitInfo({ info }: { info: AppInfo }) {
  const facts: [string, string][] = [
    [t("Cartella del kit"), info.root],
    [t("Progetti"), info.projects],
    [t("Dati e programmi"), info.data],
    [t("Piattaforma"), info.platform],
    ["WebView2", info.webview],
    [t("Versione"), info.version],
  ];
  facts.splice(3, 0, [
    t("Cartella temporanea dei programmi"),
    info.systemTemp ?? (info.platform.startsWith("win32") ? "data\\tmp" : "data/tmp"),
  ]);
  if (info.diskFs !== null) {
    const kind = { fixed: t("fisso"), removable: t("rimovibile"), remote: t("di rete"), other: t("altro") };
    const disk: string[] = [info.diskFs];
    if (info.diskKind !== null) disk.push(kind[info.diskKind]);
    if (info.diskNoexec) disk.push("noexec");
    facts.splice(3, 0, [t("Disco del kit"), disk.join(", ")]);
  }
  const rows = facts.map(([k, v]): [string, React.ReactNode] => [
    k,
    <span className="font-mono text-[12px] break-all select-text">{v}</span>,
  ]);
  return (
    <SettingsGroup id="s-kit" title={t("Il kit")}>
      <SettingItem
        title="Atollo"
        description={t("Ambiente di sviluppo portabile: linguaggi, agenti ed editor in una cartella.")}
        keywords={rows.map(([k]) => k).join(" ")}
      >
        <div className="flex items-start gap-4">
          <BrandSymbol size={40} />
          <Properties rows={rows} />
        </div>
        {info.diskNote ? <p className="mt-2 text-vsc-warning">{info.diskNote}</p> : null}
      </SettingItem>
      <KitHealth />
      <KitBackup />
      <ProblemReport />
    </SettingsGroup>
  );
}

const HEALTH_TONE: Record<HealthFinding["level"], "ok" | "warn" | "err"> = { ok: "ok", warn: "warn", err: "err" };

/** «Salute del kit» (report v6/04, N-3): what of the PC and of the drive
 * may stop the kit, each with what to do. */
function KitHealth() {
  const { data, error, setData } = useLoad<HealthFinding[] | null>(kitHealth, null, { onError: "show" });
  const { busy, error: again, run } = useAction();
  return (
    <SettingItem
      title={t("Salute del kit")}
      description={t(
        "Disco, spazio libero, percorso del kit, file resi privati da un altro account, librerie di sistema.",
      )}
      keywords="salute disco spazio fat32 exfat noexec permessi librerie health disk space permissions libraries"
    >
      {data === null ? (
        <p className="text-vsc-desc">{error ? null : t("Controllo in corso…")}</p>
      ) : (
        <ul className="m-0 list-none space-y-1.5 p-0">
          {data.map((f) => (
            <li key={f.id} className="flex items-start gap-2">
              <span className="pt-1">
                <Dot tone={HEALTH_TONE[f.level]} />
              </span>
              <span className="min-w-0">
                <span className="whitespace-pre-line">{f.text}</span>
                {f.fix && (
                  <span className="block font-mono text-[12px] break-all text-vsc-desc select-text">{f.fix}</span>
                )}
              </span>
            </li>
          ))}
        </ul>
      )}
      <div className="mt-2">
        <Button
          busy={busy === "health"}
          onClick={() => {
            run("health", () => kitHealth().then(setData));
          }}
        >
          {t("Controlla di nuovo")}
        </Button>
      </div>
      <ErrorLine error={error ?? again} />
    </SettingItem>
  );
}

/** «Segnala un problema» and «Esporta diagnostica» (report v6/04, N-1 and
 * N-2): the zip to attach is made on the PC, its secrets masked; the
 * issue opens on GitHub in the browser, and nothing is sent until the
 * user creates it there. */
function ProblemReport() {
  const { busy, error, run } = useAction();
  const [zip, setZip] = useState<string | null>(null);
  return (
    <SettingItem
      title={t("Segnalare un problema")}
      description={t(
        "Lo zip della diagnostica raccoglie versione, sistema, disco, ambiente, verifica dei programmi, componenti, porte e la parte finale dei log, con chiavi, token e password del kit tolti e la cartella del tuo profilo come ~. Resta sul PC: allegalo tu alla segnalazione, dopo averlo guardato.",
      )}
      keywords="diagnostica segnala problema bug issue github log zip diagnostics report"
    >
      <div className="flex flex-wrap items-center gap-2">
        <Button
          busy={busy === "zip"}
          onClick={() => {
            setZip(null);
            run("zip", () => diagnosticsExport().then(setZip));
          }}
        >
          {t("Esporta diagnostica")}
        </Button>
        <Button
          onClick={() => {
            run("issue", issueOpen);
          }}
        >
          {t("Segnala un problema")}
        </Button>
      </div>
      {zip && (
        <p className="mt-2 flex flex-wrap items-center gap-2">
          <span className="font-mono text-[12px] break-all select-text">{zip}</span>
          <Link
            onClick={() => {
              run("folder", () => folderOpen("diagnostics"));
            }}
          >
            {t("Apri la cartella")}
          </Link>
        </p>
      )}
      <ErrorLine error={error} />
    </SettingItem>
  );
}

export function Updates() {
  const { data: st, setData: setSt } = useLoad<UpdateStatus | null>(updateStatus, null, { onError: "keep" });
  const { busy, error, run } = useAction();
  if (!st) return null;
  const text = updateText(st);
  return (
    <SettingsGroup id="s-update" title={t("Aggiornamenti")}>
      <SettingItem title={t("Aggiornamenti di Atollo")} description={text}>
        <Button
          busy={busy === "check" || st.checking}
          onClick={() => {
            run("check", async () => {
              setSt(await updateCheck());
            });
          }}
        >
          {t("Cerca aggiornamenti")}
        </Button>
        <ErrorLine error={error} />
      </SettingItem>
    </SettingsGroup>
  );
}

/** Privacy: what Atollo sends and keeps, in short; the policy itself is
 * docs/PRIVACY.md, in the app's language. */
export function Privacy() {
  const { error, run } = useAction();
  return (
    <SettingsGroup id="s-privacy" title={t("Privacy")}>
      <SettingItem
        title={t("Cosa fa Atollo con i tuoi dati")}
        keywords="privacy policy telemetry gdpr"
        description={t(
          "Atollo non ha telemetria e non raccoglie dati: nessun account, nessuna statistica d'uso. Impostazioni, chiavi e accessi restano nella cartella del kit; le poche eccezioni sono nell'informativa.",
        )}
      >
        <ul className="list-disc space-y-1 pl-5">
          <li>{t("Cerca gli aggiornamenti di Atollo su GitHub, se l'aggiornamento automatico è attivo.")}</li>
          <li>{t("Scarica linguaggi e strumenti dalle loro fonti ufficiali quando li installi o aggiorni.")}</li>
          <li>
            {t(
              "Legge dalle stesse fonti le versioni disponibili (una volta al giorno, anche questo si spegne con ATOLLO_NO_UPDATE) e prende le estensioni dell'editor da Open VSX.",
            )}
          </li>
          <li>
            {t(
              "Con la pagina AI aperta, ogni 2 minuti legge i limiti del piano di ogni agente collegato, mandandone il token solo al suo fornitore; con l'account GitHub collegato ne usa il token con api.github.com.",
            )}
          </li>
          <li>
            {t(
              "Gli agenti AI, GitHub e i registri dei pacchetti parlano direttamente con i loro servizi, con le loro regole sulla privacy.",
            )}
          </li>
        </ul>
        <div className="mt-2">
          <Link
            onClick={() => {
              run("privacy", privacyOpen);
            }}
          >
            {t("Leggi l'informativa sulla privacy")}
          </Link>
        </div>
        <ErrorLine error={error} />
      </SettingItem>
    </SettingsGroup>
  );
}
