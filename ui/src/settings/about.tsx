// The sections of Settings about NEXKORA itself: its updates, where the kit
// is.
import { updateCheck, updateStatus, type UpdateStatus, type AppInfo } from "../api";
import { BrandSymbol } from "../brand";
import { useAction, useLoad } from "../hooks";
import { ErrorLine } from "../common";
import { t } from "../i18n";
import { Button, Properties, SettingItem, SettingsGroup } from "../workbench";

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
  facts.splice(3, 0, [t("Cartella temporanea dei programmi"), info.systemTemp ?? "data\\tmp"]);
  if (info.diskFs !== null) {
    const kind = { fixed: t("fisso"), removable: t("rimovibile"), remote: t("di rete"), other: t("altro") };
    facts.splice(3, 0, [
      t("Disco del kit"),
      info.diskKind === null ? info.diskFs : `${info.diskFs}, ${kind[info.diskKind]}`,
    ]);
  }
  const rows = facts.map(([k, v]): [string, React.ReactNode] => [
    k,
    <span className="font-mono text-[12px] break-all select-text">{v}</span>,
  ]);
  return (
    <SettingsGroup id="s-kit" title={t("Il kit")}>
      <SettingItem
        title="NEXKORA"
        description={t("Ambiente di sviluppo portabile: linguaggi, agenti ed editor in una cartella.")}
        keywords={rows.map(([k]) => k).join(" ")}
      >
        <div className="flex items-start gap-4">
          <BrandSymbol size={40} />
          <Properties rows={rows} />
        </div>
      </SettingItem>
    </SettingsGroup>
  );
}

export function Updates() {
  const { data: st, setData: setSt } = useLoad<UpdateStatus | null>(updateStatus, null, { onError: "keep" });
  const { busy, error, run } = useAction();
  if (!st) return null;
  let text: string;
  if (st.staged) text = t("La versione {0} è pronta: sostituisce questa alla chiusura di NEXKORA.", st.staged);
  else if (st.error) text = t("Controllo non riuscito: {0}", st.error);
  else if (st.latest && st.latest === st.current) text = t("NEXKORA {0} è la versione più recente.", st.current);
  else text = t("Versione installata: {0}", st.current);
  return (
    <SettingsGroup id="s-update" title={t("Aggiornamenti")}>
      <SettingItem title={t("Aggiornamenti di NEXKORA")} description={text}>
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
