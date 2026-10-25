// The packages kept to reinstall the conda environments (R, Ruby, MongoDB
// and SQLite where they come from conda-forge, make of C/C++) without the
// network after the kit moves (DECISIONS.md D-118): the setting, the space
// they take and «Libera». Only on a system where some component cannot
// follow the kit.
import { condaKept, condaKeptFree, type CondaKept, type Settings } from "../ipc";
import { t } from "../i18n";
import { megabytes } from "../shared/format";
import { ActionButton, ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { CheckboxLabel, codicon, SettingItem } from "../workbench";

export function CondaSettings({
  category,
  s,
  set,
}: {
  category: string;
  s: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  const { data, setData } = useLoad<CondaKept | null>(condaKept, null, { onError: "keep" });
  const { busy, error, run } = useAction();
  if (!data?.applies) return null;
  const title = t("Pacchetti per reinstallare senza rete");
  return (
    <SettingItem
      category={category}
      title={title}
      description={t(
        "Su questo sistema alcuni componenti (R, Ruby, MongoDB, SQLite, make) non seguono il kit quando cambia cartella e si reinstallano: con i loro pacchetti nel kit la reinstallazione non scarica nulla.",
      )}
      keywords="conda micromamba offline"
      modified={!s.keepCondaPackages}
    >
      <CheckboxLabel
        checked={s.keepCondaPackages}
        onChange={(v) => {
          set({ keepCondaPackages: v });
        }}
      >
        {t("Conserva i pacchetti nel kit")}
      </CheckboxLabel>
      {data.bytes > 0 && (
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <span className="flex-1">{t("Copia per reinstallare senza rete: {0} MB", megabytes(data.bytes))}</span>
          <ActionButton
            icon={codicon("trash")}
            label={t("Libera")}
            small
            busy={busy === "free"}
            onClick={() => {
              run("free", () => condaKeptFree().then(setData));
            }}
          />
        </div>
      )}
      <ErrorLine error={error} />
    </SettingItem>
  );
}
