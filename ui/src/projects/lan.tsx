// A project's site from the phone (report v6/04, N-6, D-158): with «Rete
// locale» on, the site's address with this PC's IP and its QR, made by the
// app (tauri/src/net/lan.rs), never by a service on the internet.
import { useCallback, useState } from "react";
import { lanLink, type LanLink, type Qr } from "../api";
import { useCopy, useLoad } from "../shared/hooks";
import { t } from "../i18n";
import { codicon, IconButton } from "../workbench";

/** The modules as one SVG path, with the quiet zone of 4 modules the
 * readers need; dark on white whatever the theme. */
export function QrCode({ qr, size = 160, label }: { qr: Qr; size?: number; label: string }) {
  const quiet = 4;
  const side = qr.width + quiet * 2;
  let d = "";
  for (let y = 0; y < qr.width; y++) {
    for (let x = 0; x < qr.width; x++) {
      if (qr.modules[y * qr.width + x] === "1") d += `M${String(x + quiet)} ${String(y + quiet)}h1v1h-1z`;
    }
  }
  return (
    <svg
      role="img"
      aria-label={label}
      width={size}
      height={size}
      viewBox={`0 0 ${String(side)} ${String(side)}`}
      shapeRendering="crispEdges"
      className="rounded"
    >
      <rect width={side} height={side} fill="#ffffff" />
      <path d={d} fill="#000000" />
    </svg>
  );
}

/** The site's address on the local network (none when the site is not
 * running or «Rete locale» is off). */
export function useLanLink(url: string | null): LanLink | null {
  const load = useCallback(() => (url === null ? Promise.resolve(null) : lanLink(url)), [url]);
  const { data } = useLoad<LanLink | null>(load, null, { onError: "keep" });
  return url === null ? null : data;
}

/** «Dal telefono»: the address, copy, and the QR on demand. */
export function LanAddress({ link }: { link: LanLink }) {
  const [shown, setShown] = useState(false);
  const { copied, copy } = useCopy(1500);
  return (
    <span className="block space-y-2">
      <span className="flex min-w-0 flex-wrap items-center gap-2">
        <code className="font-mono text-[12px] break-all select-text">{link.url}</code>
        <IconButton
          icon={codicon("copy")}
          label={t("Copia l'indirizzo per il telefono")}
          title={t("Copia")}
          onClick={() => {
            copy(link.url);
          }}
        />
        <IconButton
          icon={codicon("device-mobile")}
          label={shown ? t("Nascondi il QR") : t("Mostra il QR")}
          title={shown ? t("Nascondi il QR") : t("Mostra il QR")}
          checked={shown}
          onClick={() => {
            setShown(!shown);
          }}
        />
        {copied && <span className="text-vsc-desc">{t("Copiato")}</span>}
      </span>
      {shown && (
        <span className="block space-y-1">
          <QrCode qr={link.qr} label={t("QR di {0}", link.url)} />
          <span className="block text-[12px] text-vsc-desc">
            {t(
              "Inquadralo con la fotocamera del telefono, collegato alla stessa rete di questo PC. Se non si apre, il firewall di Windows può chiedere il permesso al primo avvio del sito: consentilo per le reti private.",
            )}
          </span>
        </span>
      )}
    </span>
  );
}
