// Rust's compiler on Windows, on Rust's page in Tools (was in Settings ›
// Versioni until D-137): GNU (the default, everything in the kit) or MSVC
// (needs Visual Studio Build Tools on the PC), from the next install or
// update of Rust. Elsewhere rustup has one host: nothing to choose.
import { rustToolchainSet, settingsGet, type Settings } from "../api";
import { ErrorLine } from "../shared/common";
import { useAction, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { Select } from "../workbench";

export function RustToolchain({ platform }: { platform: string | undefined }) {
  const { data, setData } = useLoad<Settings["rustToolchain"] | undefined>(
    () => settingsGet().then((v) => v.settings.rustToolchain),
    undefined,
    { onError: "keep" },
  );
  const { error, run } = useAction();
  if (!platform?.startsWith("win32") || data === undefined) return null;
  return (
    <div className="mt-4 max-w-[420px] space-y-1">
      <p className="font-semibold">{t("Compilatore di Rust")}</p>
      <p className="text-vsc-desc">{t("Vale dalla prossima installazione o aggiornamento di Rust.")}</p>
      <Select
        label={t("Compilatore di Rust")}
        // "gnu" written by hand is the default.
        value={data === "msvc" ? "msvc" : ""}
        onChange={(v) => {
          const next = v === "msvc" ? "msvc" : null;
          run("rust", async () => {
            await rustToolchainSet(next);
            setData(next);
          });
        }}
        options={[
          { value: "", label: t("GNU (predefinito, tutto nel kit)") },
          { value: "msvc", label: t("MSVC (richiede Visual Studio Build Tools nel PC)") },
        ]}
        className="w-full"
      />
      <ErrorLine error={error} />
    </div>
  );
}
