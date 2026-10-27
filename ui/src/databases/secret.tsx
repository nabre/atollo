// The password field the Databases page and the project page share: in a
// file of its own, so the project page does not bring the whole Databases
// page (loaded lazily, D-182) into the main chunk.

import { useState } from "react";
import { t } from "../i18n";
import { useCopy } from "../shared/hooks";
import { codicon, IconButton } from "../workbench";

/** A password: hidden until shown, copied without showing it. */
export function SecretField({ value, label }: { value: string; label: string }) {
  const [shown, setShown] = useState(false);
  const { copied, copy } = useCopy(1500);
  return (
    <span className="flex min-w-0 items-center gap-2">
      <code className="truncate font-mono text-vsc-fg">{shown ? value : "••••••••••••"}</code>
      <IconButton
        icon={shown ? codicon("eye-closed") : codicon("eye")}
        label={shown ? t("Nascondi {0}", label) : t("Mostra {0}", label)}
        title={shown ? t("Nascondi") : t("Mostra")}
        checked={shown}
        onClick={() => {
          setShown(!shown);
        }}
      />
      <IconButton
        icon={codicon("copy")}
        label={t("Copia {0}", label)}
        title={t("Copia")}
        onClick={() => {
          copy(value);
        }}
      />
      {copied && <span className="text-vsc-desc">{t("Copiata")}</span>}
    </span>
  );
}
