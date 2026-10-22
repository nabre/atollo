// The choices the first-start page asks before anything is downloaded
// (D-137): the language, the theme, Atollo's automatic updates and, only where the
// network needs it, a proxy and the certificates of a firewall that
// inspects HTTPS. Saved as they change, as on the Settings page; nothing is
// installed until «Installa», so the downloads already use them.
import { ErrorLine, LoadError } from "../shared/common";
import { t } from "../i18n";
import { SettingsGroup } from "../workbench";
import { useSettingsAutosave } from "./autosave";
import { AutoUpdateSetting, LangSetting, NetworkSettings, ThemeSetting } from "./sections";

export function FirstRunSettings({
  platform,
  onSaved,
}: {
  platform?: string | undefined;
  onSaved?: (() => void) | undefined;
}) {
  const { view, s, loadError, error, status, set, net, retry } = useSettingsAutosave(onSaved);
  if (loadError) return <LoadError error={loadError} onRetry={retry} />;
  if (!s) return null;
  const start = t("Prima di iniziare");
  return (
    <>
      <SettingsGroup
        id="group-start"
        title={start}
        description={t("Scelte che valgono per tutto il kit: le ritrovi e le cambi in Impostazioni.")}
      >
        <LangSetting category={start} s={s} set={set} />
        <ThemeSetting category={start} s={s} set={set} />
        <AutoUpdateSetting category={start} s={s} set={set} />
        {view?.problem && (
          <ErrorLine
            error={t(
              "Le impostazioni in {0} non sono leggibili: uso quelle predefinite. Il file verrà riscritto alla prima modifica.",
              view.problem,
            )}
          />
        )}
        <ErrorLine error={error} />
        {status === "saving" && (
          <p role="status" className="text-vsc-desc">
            {t("Salvataggio…")}
          </p>
        )}
      </SettingsGroup>
      <NetworkSettings s={s} net={net} platform={platform} />
    </>
  );
}
