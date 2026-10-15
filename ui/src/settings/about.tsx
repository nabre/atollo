// The sections of Settings about Atollo itself: its updates, where the kit
// is.
import { useEffect, useState } from "react";
import { updateCheck, updateStatus, type UpdateStatus, type AppInfo } from "../api";
import { BrandSymbol } from "../brand";
import { useAction } from "../hooks";
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
      </SettingItem>
    </SettingsGroup>
  );
}

export function Updates() {
  const [st, setSt] = useState<UpdateStatus | null>(null);
  const { busy, error, run } = useAction();
  useEffect(() => {
    updateStatus().then(setSt, () => undefined);
  }, []);
  if (!st) return null;
  let text: string;
  if (st.staged) text = t("La versione {0} è pronta: sostituisce questa alla chiusura di Atollo.", st.staged);
  else if (st.error) text = t("Controllo non riuscito: {0}", st.error);
  else if (st.latest && st.latest === st.current) text = t("Atollo {0} è la versione più recente.", st.current);
  else text = t("Versione installata: {0}", st.current);
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
