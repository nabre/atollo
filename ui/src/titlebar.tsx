// Title bar of the app window, drawn by the interface: the window has no
// Windows frame (tauri/src/main.rs, `decorations(false)`). The empty part
// drags the window and a double click maximizes it (`data-tauri-drag-region`);
// the three buttons are those of Windows, native buttons as VSCodium's
// window controls (no vscode-elements component draws them). Only the main window has the
// permissions they use (capabilities/main.json). Its measures are VSCodium's
// without the command center: 30px high, window controls 46px wide.
import { useEffect, useState } from "react";
import { VscChromeClose, VscChromeMaximize, VscChromeMinimize, VscChromeRestore } from "react-icons/vsc";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { browserPreview } from "./api";
import { t } from "./i18n";
import { BrandWordmark } from "./brand";
import { Badge } from "./tags";

/** The window, or null in the browser preview and in the tests. */
function current(): Window | null {
  if (browserPreview) return null;
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

function act(f: (w: Window) => Promise<void>) {
  const w = current();
  if (w) f(w).catch(() => undefined);
}

export function TitleBar({ title }: { title: string | undefined }) {
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const w = current();
    if (!w) return;
    let stop: (() => void) | null = null;
    let gone = false;
    const read = () => {
      w.isMaximized().then(
        (m) => {
          if (!gone) setMaximized(m);
        },
        () => undefined,
      );
    };
    read();
    w.onResized(read).then(
      (f) => {
        if (gone) f();
        else stop = f;
      },
      () => undefined,
    );
    return () => {
      gone = true;
      stop?.();
    };
  }, []);
  const button =
    "flex h-full w-vsc-window-control items-center justify-center text-vsc-titlebar-fg hover:bg-vsc-toolbar-hover focus-visible:bg-vsc-toolbar-hover";
  return (
    <div
      data-tauri-drag-region
      className="flex h-vsc-titlebar shrink-0 items-center border-b border-vsc-titlebar-border bg-vsc-titlebar text-vsc-titlebar-fg select-none"
    >
      <span data-tauri-drag-region className="flex items-center gap-vsc-size80 pl-vsc-size120 text-vsc-body-small">
        <BrandWordmark height={14} />
        {title && (
          <span data-tauri-drag-region className="text-vsc-desc">
            — {title}
          </span>
        )}
      </span>
      <span data-tauri-drag-region className="h-full flex-1" />
      {browserPreview && <Badge className="mr-vsc-size80">{t("Anteprima nel browser: dati di esempio")}</Badge>}
      <button
        type="button"
        aria-label={t("Riduci a icona")}
        title={t("Riduci a icona")}
        className={button}
        onClick={() => {
          act((w) => w.minimize());
        }}
      >
        <VscChromeMinimize size={16} aria-hidden="true" />
      </button>
      <button
        type="button"
        aria-label={maximized ? t("Ripristina") : t("Ingrandisci")}
        title={maximized ? t("Ripristina") : t("Ingrandisci")}
        className={button}
        onClick={() => {
          act((w) => w.toggleMaximize());
        }}
      >
        {maximized ? (
          <VscChromeRestore size={16} aria-hidden="true" />
        ) : (
          <VscChromeMaximize size={16} aria-hidden="true" />
        )}
      </button>
      <button
        type="button"
        aria-label={t("Chiudi")}
        title={t("Chiudi")}
        // VSCodium's close hover, a fixed color in its CSS: not a theme color.
        className="flex h-full w-vsc-window-control items-center justify-center text-vsc-titlebar-fg hover:bg-[rgba(232,17,35,0.9)] hover:text-white focus-visible:bg-[rgba(232,17,35,0.9)] focus-visible:text-white"
        onClick={() => {
          // Same path as the Windows button: the app asks first if something is open.
          act((w) => w.close());
        }}
      >
        <VscChromeClose size={16} aria-hidden="true" />
      </button>
    </div>
  );
}
