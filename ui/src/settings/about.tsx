// The sections of Settings about Atollo itself: its updates, where the kit
// is, what it sends over the network (docs/PRIVACY.md).
import { privacyOpen, updateCheck, updateStatus, type UpdateStatus, type AppInfo } from "../api";
import { BrandSymbol } from "../brand";
import { useAction, useLoad } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { updateText } from "../update";
import { Button, Link, Properties, SettingItem, SettingsGroup } from "../workbench";

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
    </SettingsGroup>
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
              "Con la pagina Agenti aperta, ogni 2 minuti legge i limiti del piano di ogni agente collegato, mandandone il token solo al suo fornitore; con l'account GitHub collegato ne usa il token con api.github.com.",
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
